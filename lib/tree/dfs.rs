use crate::{
    // prelude::*,
    mate::*,
    gen::*,
    eval::{EvalFn, Eval},
};
use super::*;
use std::time::{Instant, Duration};
use std::borrow::Cow;
use scoped_threadpool::Pool;

pub fn dfs_schedule<F: EvalFn, C: for<'b> Fn(&TreeNode<'b>) -> bool + Copy + Send>(
    game: &Game,
    depth: usize,
    max_duration: Option<Duration>,
    eval_fn: F,
    pool_size: usize,
    max_pool_size: usize,
    n_threads: u32,
    condition: C,
) -> Option<(EvalNode, Eval)> {
    let start = Instant::now();

    let mut tasks = Tasks::new(game, pool_size, max_pool_size, max_duration);

    if !tasks.fill_pool() {
        return None
    }

    let mut pool = Pool::new(n_threads);

    pool.scoped(|scope| {
        for task in &mut tasks {
            let executor = DFSExecutor::new(
                game,
                task.0,
                task.1,
                max_duration,
                eval_fn,
                condition,
                depth,
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
pub fn dfs_bl_schedule<F: EvalFn>(
    game: &Game,
    depth: usize,
    max_branches: usize,
    max_duration: Option<Duration>,
    eval_fn: F,
    pool_size: usize,
    max_pool_size: usize,
    n_threads: u32,
) -> Option<(EvalNode, Eval)> {
    dfs_schedule(
        game,
        depth,
        max_duration,
        eval_fn,
        pool_size,
        max_pool_size,
        n_threads,
        move |node| node.branches <= max_branches,
    )
}

pub fn dfs<'a, F: EvalFn, C: for<'b> Fn(&TreeNode<'b>) -> bool + Copy>(
    game: &'a Game,
    node: TreeNode<'a>,
    depth: usize,
    max_duration: Option<Duration>,
    eval_fn: F,
    condition: C,
) -> Option<(EvalNode, Eval)> {
    dfs_rec(
        game,
        node,
        depth,
        f32::NEG_INFINITY,
        f32::INFINITY,
        max_duration.unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1)),
        eval_fn,
        condition,
    )
}


pub fn dfs_bl<'a, F: EvalFn>(
    game: &'a Game,
    node: TreeNode<'a>,
    depth: usize,
    max_branches: usize,
    max_duration: Option<Duration>,
    eval_fn: F,
) -> Option<(EvalNode, Eval)> {
    dfs_rec(
        game,
        node,
        depth,
        f32::NEG_INFINITY,
        f32::INFINITY,
        max_duration.unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1)),
        eval_fn,
        move |node| node.branches <= max_branches,
    )
}

fn dfs_rec<'a, F: EvalFn, C: for<'b> Fn(&TreeNode<'b>) -> bool + Copy>(
    game: &'a Game,
    node: TreeNode<'a>,
    depth: usize,
    mut alpha: Eval,
    beta: Eval,
    max_duration: Duration,
    eval_fn: F,
    condition: C,
) -> Option<(EvalNode, Eval)> {
    if max_duration == Duration::new(0, 0) {
        return None
    }

    let start = Instant::now();

    match is_mate(game, &node.partial_game, Some(max_duration)) {
        Mate::Checkmate => {
            Some((node.into(), f32::NEG_INFINITY))
        }
        Mate::Stalemate => {
            Some((node.into(), 0.0))
        }
        Mate::TimeoutCheckmate | Mate::TimeoutStalemate | Mate::Error => {
            None
        }
        Mate::None(ms, pos, iter) => {
            if depth == 0 {
                let score = eval_fn.eval(game, &node)?;
                // score is expected to return higher for the current player
                Some((node.into(), score))
            } else {
                let mut best_node: Option<EvalNode> = None;
                let mut best_score: Eval = f32::NEG_INFINITY;

                let mut iter = match iter {
                    None => GenLegalMovesetIter::new(game, Cow::Borrowed(&node.partial_game), Some(max_duration)),
                    Some(i) => i,
                };

                let initial_node = vec![(ms, pos)];

                for (child_ms, child_pos) in initial_node.into_iter().chain(&mut iter) {
                    if start.elapsed() >= max_duration {
                        return None
                    }

                    let child_node = TreeNode::extend(&node, child_ms, child_pos);

                    if condition(&child_node) {
                        let (child_best, child_score) = dfs_rec(
                            game,
                            child_node,
                            depth - 1,
                            -beta,
                            -alpha,
                            max_duration.checked_sub(start.elapsed())?,
                            eval_fn,
                            condition
                        )?;

                        if -child_score > best_score {
                            best_score = -child_score;
                            best_node = Some(child_best);
                        }

                        if best_score > alpha {
                            alpha = best_score;
                        }

                        if alpha >= beta || alpha == f32::INFINITY {
                            break
                        }
                    }
                }

                if iter.timed_out() {
                    return None
                }

                match best_node {
                    Some(node) => {
                        Some((node.into(), best_score))
                    }
                    None => {
                        Some((node.into(), best_score))
                    }
                }
            }
        }
    }
}

// == IDDFS ==

pub fn iddfs<'a, F: EvalFn, C: for<'b> Fn(&TreeNode<'b>) -> bool + Copy>(
    game: &'a Game,
    node: TreeNode<'a>,
    max_duration: Option<Duration>,
    eval_fn: F,
    condition: C,
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
            condition
        ) {
            best = Some(best_node);
            depth += 1;
        } else {
            break
        }
    }

    best
}

pub fn iddfs_bl<'a, F: EvalFn>(
    game: &'a Game,
    node: TreeNode<'a>,
    max_branches: usize,
    max_duration: Option<Duration>,
    eval_fn: F,
) -> Option<(EvalNode, Eval)> {
    iddfs(
        game,
        node,
        max_duration,
        eval_fn,
        move |node| node.branches <= max_branches,
    )
}

pub fn iddfs_schedule<'a, F: EvalFn, C: for<'b> Fn(&TreeNode<'b>) -> bool + Copy + Send>(
    game: &'a Game,
    max_duration: Option<Duration>,
    eval_fn: F,
    pool_size: usize,
    max_pool_size: usize,
    n_threads: u32,
    condition: C,
) -> Option<(EvalNode, Eval)> {
    let start = Instant::now();

    let mut tasks = Tasks::new(game, pool_size, max_pool_size, max_duration);
    if !tasks.fill_pool() {
        return None
    }
    let mut pool = Pool::new(n_threads);
    let mut best = None;

    let mut depth: usize = 0;

    while max_duration.map(|d| start.elapsed() < d).unwrap_or(true) {
        pool.scoped(|scope| {
            for task in &mut tasks {
                let executor = DFSExecutor::new(
                    game,
                    task.0,
                    task.1,
                    max_duration,
                    eval_fn,
                    condition,
                    depth,
                );
                scope.execute(move || {
                    if max_duration.map(|d| d <= start.elapsed()).unwrap_or(false) {
                        return
                    }
                    executor.execute(start);
                });
            }
        });

        tasks.update_tree();
        if let Some(b) = tasks.best_move() {
            let v = b.1;
            best = Some(b);
            if v.is_infinite() {
                break
            }
        } else {
            break
        }
        tasks.reset(true, true);

        depth += 1;
    }

    best
}

pub fn iddfs_bl_schedule<'a, F: EvalFn>(
    game: &'a Game,
    max_branches: usize,
    max_duration: Option<Duration>,
    eval_fn: F,
    pool_size: usize,
    max_pool_size: usize,
    n_threads: u32,
) -> Option<(EvalNode, Eval)> {
    iddfs_schedule(
        game,
        max_duration,
        eval_fn,
        pool_size,
        max_pool_size,
        n_threads,
        move |node| node.branches <= max_branches,
    )
}


struct DFSExecutor<'a, F, C>
where
    F: EvalFn,
    C: for<'b> Fn(&TreeNode<'b>) -> bool + Copy
{
    game: &'a Game,
    task: TreeNode<'a>,
    handle: schedule::TreeHandle,
    max_duration: Option<Duration>,
    eval_fn: F,
    condition: C,
    depth: usize,
}

impl<'a, F, C> DFSExecutor<'a, F, C>
where
    F: EvalFn,
    C: for<'b> Fn(&TreeNode<'b>) -> bool + Copy
 {
    fn new(
        game: &'a Game,
        task: TreeNode<'a>,
        handle: schedule::TreeHandle,
        max_duration: Option<Duration>,
        eval_fn: F,
        condition: C,
        depth: usize,
    ) -> Self {
        Self {
            game,
            task,
            handle,
            max_duration,
            eval_fn,
            condition,
            depth
        }
    }

    fn execute(self, start: Instant) -> Option<Eval> {
        let (_node, value) = if self.task.path.len() > self.depth {
            let score = match self.eval_fn.eval(self.game, &self.task) {
                Some(score) => score,
                None => {
                    return None
                }
            };

            (self.task.into(), score)
        } else {
            let depth = self.depth - self.task.path.len();

            match dfs(
                self.game,
                self.task,
                depth,
                self.max_duration.map(|d| d.checked_sub(start.elapsed()).unwrap_or(Duration::new(0, 0))),
                self.eval_fn,
                self.condition
            ) {
                Some(res) => res,
                None => {
                    return None
                }
            }
        };

        self.handle.report(value);

        Some(value)
    }
}
