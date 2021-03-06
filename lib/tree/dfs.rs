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
use std::sync::Mutex;

pub fn dfs_schedule<F: EvalFn, C: for<'b> Fn(&TreeNode<'b>) -> bool + Copy + Send>(
    game: &Game,
    depth: usize,
    max_duration: Option<Duration>,
    eval_fn: F,
    pool_size: usize,
    n_threads: u32,
    condition: C,
) -> Option<(EvalNode, Eval)> {
    let start = Instant::now();

    let tasks = Mutex::new(Tasks::new(game, pool_size, max_duration));
    let mut pool = Pool::new(n_threads);

    {
        let tasks = &tasks;
        pool.scoped(move |scope| {
            for _n in 0..n_threads {
                scope.execute(move || {
                    loop {
                        let elem = {
                            let mut guard = tasks.lock().unwrap();
                            guard.next()
                        };

                        let (task, handle) = match elem {
                            Some(t) => t,
                            None => break
                        };

                        if max_duration.map(|d| d <= start.elapsed()).unwrap_or(false) {
                            return
                        }

                        let (node, value) = if task.path.len() > depth {
                            let score = match eval_fn.eval(&game, &task) {
                                Some(score) => score,
                                None => {
                                    return
                                }
                            };

                            (task.into(), score)
                        } else {
                            let depth = depth - task.path.len();
                            // println!("@{}: {:?} {:?}", _n, task.path, start.elapsed());
                            match dfs(&game, task, depth, max_duration.map(|d| d - start.elapsed()), eval_fn, condition) {
                                Some(res) => res,
                                None => {
                                    return
                                }
                            }
                        };

                        handle.report(value);

                        if value == f32::INFINITY && node.path.len() == 1 {
                            // TODO: block tasks to indirectly stop the other threads?
                            return
                        }
                    }
                });
            }

            scope.join_all()
        });
    }

    let mut tasks = tasks.into_inner().unwrap();

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
    n_threads: u32,
) -> Option<(EvalNode, Eval)> {
    dfs_schedule(
        game,
        depth,
        max_duration,
        eval_fn,
        pool_size,
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
