use crate::{
    // prelude::*,
    mate::*,
    gen::*,
    eval::{EvalFn, Eval},
    check::is_in_check,
    goals::branch::*,
};
use super::*;
use std::time::{Instant, Duration};
use std::borrow::Cow;
use scoped_threadpool::Pool;

#[cfg(feature = "countnodes")]
lazy_static! {
    pub static ref NODES: std::sync::Mutex<u64> = std::sync::Mutex::new(0);
    pub static ref SIGMA: std::sync::Mutex<Duration> = std::sync::Mutex::new(Duration::new(0, 0));
}

const APPROX_MIN_NODES: usize = 16;
const PRUNE_VALUE: Eval = Eval::NEG_INFINITY;

pub fn dfs_schedule<F: EvalFn, G: Goal>(
    game: &Game,
    depth: usize,
    eval_fn: F,
    options: TasksOptions<G>,
    approx: bool,
) -> Option<(EvalNode, Eval)> {
    let start = Instant::now();

    let mut tasks = Tasks::new(game, options);

    if !tasks.fill_pool(depth) {
        return None
    }

    let mut pool = Pool::new(options.n_threads);

    pool.scoped(|scope| {
        for task in &mut tasks {
            let executor = DFSExecutor::new(
                game,
                task.0,
                task.1,
                options.max_duration,
                eval_fn,
                options.goal,
                depth,
                approx,
            );
            scope.execute(move || {
                executor.execute(start);
            });
        }
    });

    // println!("{:?}", tasks.tree);
    tasks.update_tree();
    // println!("{:?}", tasks.tree);
    tasks.best_move()
}


// TODO: actually make this threaded
pub fn dfs_bl_schedule<F: EvalFn, G: Goal>(
    game: &Game,
    depth: usize,
    max_branches: usize,
    eval_fn: F,
    options: TasksOptions<G>,
    approx: bool,
) -> Option<(EvalNode, Eval)> {
    let g = options.goal.or(MaxBranching::new(&game.info, max_branches));
    dfs_schedule(
        game,
        depth,
        eval_fn,
        options.goal(g),
        approx,
    )
}

pub fn dfs<'a, F: EvalFn, G: Goal>(
    game: &'a Game,
    node: TreeNode<'a>,
    depth: usize,
    max_duration: Option<Duration>,
    eval_fn: F,
    goal: G,
    approx: bool,
) -> Option<(EvalNode, Eval)> {
    #[cfg(feature = "countnodes")]
    let start = Instant::now();

    let res = dfs_rec(
        game,
        node,
        depth,
        f32::NEG_INFINITY,
        f32::INFINITY,
        max_duration.unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1)),
        eval_fn,
        goal,
        approx,
    )?;

    #[cfg(feature = "countnodes")]
    {
        let sigma = start.elapsed();
        if let Ok(mut guard_nodes) = NODES.lock() {
            if let Ok(mut guard_sigma) = SIGMA.lock() {
                *guard_nodes += res.2;
                *guard_sigma += sigma;
            }
        }
    }

    Some((res.0, res.1))
}


pub fn dfs_bl<'a, F: EvalFn, G: Goal>(
    game: &'a Game,
    node: TreeNode<'a>,
    depth: usize,
    max_branches: usize,
    max_duration: Option<Duration>,
    eval_fn: F,
    goal: G,
    approx: bool,
) -> Option<(EvalNode, Eval)> {
    #[cfg(feature = "countnodes")]
    let start = Instant::now();

    let res = dfs_rec(
        game,
        node,
        depth,
        f32::NEG_INFINITY,
        f32::INFINITY,
        max_duration.unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1)),
        eval_fn,
        goal.or(MaxBranching::new(&game.info, max_branches)),
        approx,
    )?;

    #[cfg(feature = "countnodes")]
    {
        let sigma = start.elapsed();
        if let Ok(mut guard_nodes) = NODES.lock() {
            if let Ok(mut guard_sigma) = SIGMA.lock() {
                *guard_nodes += res.2;
                *guard_sigma += sigma;
            }
        }
    }

    Some((res.0, res.1))
}

fn dfs_rec<'a, F: EvalFn, G: Goal>(
    game: &'a Game,
    node: TreeNode<'a>,
    depth: usize,
    mut alpha: Eval,
    beta: Eval,
    max_duration: Duration,
    eval_fn: F,
    goal: G,
    approx: bool,
) -> Option<(EvalNode, Eval, u64)> {
    if max_duration == Duration::new(0, 0) {
        return None
    }

    let start = Instant::now();

    match is_mate(game, &node.partial_game, Some(max_duration)) {
        Mate::Checkmate => {
            Some((node.into(), f32::NEG_INFINITY, 1))
        }
        Mate::Stalemate => {
            Some((node.into(), 0.0, 1))
        }
        Mate::TimeoutCheckmate | Mate::TimeoutStalemate | Mate::Error => {
            None
        }
        Mate::None(ms, pos, iter) => {
            if depth == 0 {
                let score = eval_fn.eval(game, &node)?;
                // score is expected to return higher for the current player
                Some((node.into(), score, 1))
            } else {
                let mut best_node: Option<EvalNode> = None;
                let mut best_score: Option<Eval> = None;

                let mut iter = match iter {
                    None => GenLegalMovesetIter::new(game, Cow::Borrowed(&node.partial_game), Some(max_duration)),
                    Some(i) => i,
                };

                let initial_node = vec![(ms, pos)];
                let mut yielded: usize = 0;
                let mut nodes = 1;

                for (child_ms, child_pos) in initial_node.into_iter().chain(&mut iter) {
                    if approx && yielded >= APPROX_MIN_NODES {
                        if child_ms.moves().iter().find(|mv|
                            mv.from.1.t() > node.partial_game.info.present
                            || !node.partial_game.info.is_active(mv.from.1.l())
                        ).is_some() {
                            // println!("Pruned @ {:?}", child_ms);
                            break
                        }
                    }
                    yielded += 1;
                    if start.elapsed() >= max_duration {
                        return None
                    }

                    let child_node = TreeNode::extend(&node, child_ms, child_pos);

                    let (child_best, child_score, child_nodes) = match goal.verify(&child_node.path, game, &child_node.partial_game, Some(depth)) {
                        GoalResult::Win => (child_node.into(), Eval::INFINITY, 1),
                        GoalResult::Loss => (child_node.into(), Eval::NEG_INFINITY, 1),
                        GoalResult::Error => return None,
                        GoalResult::Score(s) => (child_node.into(), s, 1),
                        GoalResult::Ignore => continue,
                        GoalResult::Continue => {
                             dfs_rec(
                                game,
                                child_node,
                                depth - 1,
                                -beta,
                                -alpha,
                                max_duration.checked_sub(start.elapsed())?,
                                eval_fn,
                                goal,
                                approx,
                            )?
                        }
                    };

                    nodes += child_nodes;

                    match best_score {
                        None => {
                            best_score = Some(-child_score);
                            best_node = Some(child_best);
                        }
                        Some(b) => {
                            if -child_score > b {
                                best_score = Some(-child_score);
                                best_node = Some(child_best);
                            }
                        }
                    }

                    if best_score.unwrap() > alpha {
                        alpha = best_score.unwrap();
                    }

                    if alpha >= beta || alpha == f32::INFINITY {
                        break
                    }
                }

                if iter.timed_out() {
                    return None
                }

                match best_node {
                    Some(node) => {
                        Some((node.into(), best_score.unwrap(), nodes))
                    }
                    None => {
                        let score = if is_in_check(game, &node.partial_game)?.0 {
                            Eval::NEG_INFINITY
                        } else {
                            PRUNE_VALUE
                        };
                        Some((node.into(), score, nodes))
                    }
                }
            }
        }
    }
}

// == IDDFS ==

pub fn iddfs<'a, F: EvalFn, G: Goal>(
    game: &'a Game,
    node: TreeNode<'a>,
    max_duration: Option<Duration>,
    eval_fn: F,
    goal: G,
    approx: bool,
) -> Option<(EvalNode, Eval)> {
    let mut best = None;
    let mut depth = 0;
    let start = Instant::now();

    loop {
        if let Some(max_duration) = max_duration {
            if start.elapsed() >= max_duration {
                break
            }
        }

        if let Some(best_node) = dfs(
            game,
            node.clone(),
            depth,
            max_duration.map(|d| d.checked_sub(start.elapsed())).flatten(),
            eval_fn,
            goal,
            approx,
        ) {
            best = Some(best_node);
            depth += 1;
        } else {
            break
        }
    }

    best
}

pub fn iddfs_bl<'a, F: EvalFn, G: Goal>(
    game: &'a Game,
    node: TreeNode<'a>,
    max_branches: usize,
    max_duration: Option<Duration>,
    eval_fn: F,
    goal: G,
    approx: bool,
) -> Option<(EvalNode, Eval)> {
    iddfs(
        game,
        node,
        max_duration,
        eval_fn,
        goal.or(MaxBranching::new(&game.info, max_branches)),
        approx,
    )
}

pub fn iddfs_schedule<'a, F: EvalFn, G: Goal>(
    game: &'a Game,
    eval_fn: F,
    options: TasksOptions<G>,
    approx: bool,
) -> Option<(EvalNode, Eval)> {
    let start = Instant::now();
    let max_duration = options.max_duration;

    let mut tasks = Tasks::new(game, options);
    if !tasks.fill_pool(1) {
        return None
    }
    let mut pool = Pool::new(options.n_threads);
    let mut best = None;

    let mut depth: usize = 1;

    while max_duration.map(|d| start.elapsed() < d).unwrap_or(true) {
        pool.scoped(|scope| {
            for task in &mut tasks {
                let executor = DFSExecutor::new(
                    game,
                    task.0,
                    task.1,
                    max_duration,
                    eval_fn,
                    options.goal,
                    depth,
                    approx,
                );
                scope.execute(move || {
                    if max_duration.map(|d| d <= start.elapsed()).unwrap_or(false) {
                        return
                    }
                    executor.execute(start);
                });
            }
        });

        if max_duration.map(|d| d <= start.elapsed()).unwrap_or(false) {
            break
        }

        // println!("{{Depth {} complete!}}", depth);

        tasks.update_tree();
        if let Some(b) = tasks.best_move() {
            let v = b.1;

            if cfg!(feature = "iddfs_log") {
                print!("{{d={}, score={:7.2}, path=", depth, b.1);
                for ms in b.0.path.iter(){
                    print!("{}", ms);
                }
                println!("}}");
            }

            if v.is_infinite() || tasks.root_eval()?.is_infinite() {
                if tasks.root_eval()? == Eval::INFINITY {
                    best = Some(b);
                } else {
                    if let Some((mv, _)) = best {
                        best = Some((mv, v));
                    } else {
                        best = Some(b);
                    }
                }
                break
            }

            best = Some(b);
        } else {
            // println!("{:?}", tasks);
            break
        }
        tasks.reset(true, false, depth + 1);

        depth += 1;
    }

    best
}

pub fn iddfs_bl_schedule<'a, F: EvalFn, G: Goal>(
    game: &'a Game,
    max_branches: usize,
    eval_fn: F,
    options: TasksOptions<G>,
    approx: bool,
) -> Option<(EvalNode, Eval)> {
    let g = options.goal.or(MaxBranching::new(&game.info, max_branches));
    iddfs_schedule(
        game,
        eval_fn,
        options.goal(g),
        approx,
    )
}


struct DFSExecutor<'a, F, G>
where
    F: EvalFn,
    G: Goal,
{
    game: &'a Game,
    task: TreeNode<'a>,
    handle: schedule::TreeHandle,
    max_duration: Option<Duration>,
    eval_fn: F,
    goal: G,
    depth: usize,
    approx: bool,
}

impl<'a, F, G> DFSExecutor<'a, F, G>
where
    F: EvalFn,
    G: Goal,
 {
    fn new(
        game: &'a Game,
        task: TreeNode<'a>,
        handle: schedule::TreeHandle,
        max_duration: Option<Duration>,
        eval_fn: F,
        goal: G,
        depth: usize,
        approx: bool,
    ) -> Self {
        Self {
            game,
            task,
            handle,
            max_duration,
            eval_fn,
            goal,
            depth,
            approx,
        }
    }

    fn execute(self, start: Instant) -> Option<Eval> {
        let (_node, value) = if self.task.path.len() > self.depth {
            match self.goal.verify(&self.task.path, &self.game, &self.task.partial_game, Some(self.task.path.len() + self.depth)) {
                GoalResult::Win => (self.task.into(), Eval::INFINITY),
                GoalResult::Loss => (self.task.into(), Eval::NEG_INFINITY),
                GoalResult::Score(s) => (self.task.into(), s),
                GoalResult::Error => return None,
                GoalResult::Ignore => return None,
                GoalResult::Continue => {
                    let score = match self.eval_fn.eval(self.game, &self.task) {
                        Some(score) => score,
                        None => {
                            return None
                        }
                    };

                    (self.task.into(), score)
                }
            }
        } else {
            let depth = self.depth - self.task.path.len();

            dfs(
                self.game,
                self.task,
                depth,
                self.max_duration.map(|d| d.checked_sub(start.elapsed()).unwrap_or(Duration::new(0, 0))),
                self.eval_fn,
                self.goal,
                self.approx,
            )?
        };

        self.handle.report(value);

        Some(value)
    }
}
