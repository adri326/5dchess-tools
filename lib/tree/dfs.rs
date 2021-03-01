use crate::{
    prelude::*,
    mate::*,
    gen::*,
    eval::EvalFn,
};
use super::*;
use std::time::{Instant, Duration};
use std::borrow::Cow;
use std::sync::Arc;

// TODO: actually make this threaded
pub fn dfs_schedule<F: EvalFn>(
    game: Arc<Game>,
    depth: usize,
    max_duration: Option<Duration>,
    eval_fn: F,
    pool_size: usize,
) -> Option<(EvalNode, F::Output)> {
    let mut best_node: Option<EvalNode> = None;
    let mut best_score: F::Output = F::MIN;
    let start = Instant::now();

    for task in Tasks::new(Arc::clone(&game), pool_size, max_duration) {
        let invert_score = task.path.len() % 2 != 0;
        let (node, value) = if task.path.len() > depth {
            let score = eval_fn.eval(&game, &task)?;

            (task.into(), score)
        } else {
            let depth = depth - task.path.len();
            dfs(&game, task, depth, max_duration, eval_fn)?
        };

        let value = if invert_score { -value } else { value };

        if value > best_score {
            best_node = Some(node);
            best_score = value;

            if best_score == -F::MIN {
                break
            }
        }
    }

    match best_node {
        None => None,
        Some(x) => Some((x, best_score)),
    }
}

pub fn dfs<'a, F: EvalFn>(
    game: &'a Game,
    node: TreeNode<'a>,
    depth: usize,
    max_duration: Option<Duration>,
    eval_fn: F,
) -> Option<(EvalNode, F::Output)> {
    dfs_rec(
        game,
        node,
        depth,
        F::MIN,
        -F::MIN,
        max_duration.unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1)),
        eval_fn
    )
}

fn dfs_rec<'a, F: EvalFn>(
    game: &'a Game,
    node: TreeNode<'a>,
    depth: usize,
    mut alpha: F::Output,
    beta: F::Output,
    max_duration: Duration,
    eval_fn: F,
) -> Option<(EvalNode, F::Output)> {
    if max_duration == Duration::new(0, 0) {
        return None
    }

    let start = Instant::now();

    match is_mate(game, &node.partial_game, Some(max_duration)) {
        Mate::Checkmate => {
            Some((node.into(), F::MIN))
        }
        Mate::Stalemate => {
            Some((node.into(), F::DRAW))
        }
        Mate::TimeoutCheckmate | Mate::TimeoutStalemate | Mate::Error => {
            None
        }
        Mate::None(_ms) => {
            if depth == 0 {
                let score = eval_fn.eval(game, &node)?;
                // score is expected to return higher for the current player
                Some((node.into(), score))
            } else {
                let mut best_node: Option<EvalNode> = None;
                let mut best_score: F::Output = F::MIN;

                let mut iter = GenLegalMovesetIter::new(game, Cow::Borrowed(&node.partial_game), Some(max_duration));

                for (child_ms, child_pos) in &mut iter {
                    if start.elapsed() > max_duration {
                        return None
                    }

                    let mut child_path = node.path.clone();
                    child_path.push(child_ms);

                    let child_node = TreeNode {
                        partial_game: child_pos,
                        path: child_path,
                    };

                    let (child_best, child_score) = dfs_rec(game, child_node, depth - 1, -beta, -alpha, max_duration - start.elapsed(), eval_fn)?;

                    if -child_score > best_score {
                        best_score = -child_score;
                        best_node = Some(child_best);
                    }

                    if best_score > alpha {
                        alpha = best_score;
                    }

                    if alpha >= beta || alpha == -F::MIN {
                        break
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
