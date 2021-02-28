use crate::{
    prelude::*,
    mate::*,
    gen::*,
};
use super::*;
use std::time::{Instant, Duration};
use std::borrow::Cow;

type Eval = f32;

pub fn dfs<'a>(
    game: &'a Game,
    node: TreeNode<'a>,
    depth: usize,
    mut alpha: Eval,
    beta: Eval,
    max_duration: Duration,
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
        Mate::None(ms) => {
            if depth == 0 {
                let score = evaluate_position(game, &node);
                // score is expected to return higher for the current player
                Some((node.into(), score))
            } else {
                let mut best_node: Option<EvalNode> = None;
                let mut best_score: Eval = f32::NEG_INFINITY;

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

                    let (child_best, child_score) = dfs(game, child_node, depth - 1, -beta, -alpha, max_duration - start.elapsed())?;

                    if -child_score > best_score {
                        best_score = -child_score;
                        best_node = Some(child_best);
                    }

                    if best_score > alpha {
                        alpha = best_score;
                    }

                    if alpha >= beta {
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

fn evaluate_position(game: &Game, node: &TreeNode) -> Eval {
    0.0
}
