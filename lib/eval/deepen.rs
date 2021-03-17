use super::*;
use crate::mate::*;
use crate::gen::moveset::*;
use std::borrow::Cow;
use std::time::{Instant, Duration};

// TODO: add goals?

#[derive(Clone, Copy)]
pub struct Deepen<E: EvalFn, I: EvalFn> {
    pub depth: usize,
    pub breadth_wide: usize,
    pub breadth_narrow: usize,

    pub win_value: Eval,
    pub draw_value: Eval,
    pub eval: E,
    pub none_mult: Eval,
    pub intermediary_eval: I,

    pub timeout_win_value: Eval,
    pub timeout_draw_value: Eval,

    pub timeout_default: Option<Eval>,

    pub max_time: Duration,
    pub node_max_time: Duration,
}

impl Default for Deepen<NoEvalFn, NoEvalFn> {
    fn default() -> Self {
        Self {
            depth: 2,
            breadth_wide: 15,
            breadth_narrow: 5,

            win_value: 5.0,
            draw_value: 0.0,
            eval: NoEvalFn(),
            none_mult: 0.1,
            intermediary_eval: NoEvalFn(),

            timeout_win_value: 3.0, // Not as valuable as a clear win
            timeout_draw_value: -0.05, // Slight disadvantage because of the computational complexity

            timeout_default: Some(-0.05),

            max_time: Duration::new(0, 100_000),
            node_max_time: Duration::new(0, 10_000),
        }
    }
}

macro_rules! prop {
    ( $name:tt: $type:ty ) => {
        pub fn $name(mut self, value: $type) -> Self {
            self.$name = value;
            self
        }
    }
}

impl<E: EvalFn, I: EvalFn> Deepen<E, I> {
    prop!(depth: usize);
    prop!(breadth_wide: usize);
    prop!(breadth_narrow: usize);
    prop!(win_value: Eval);
    prop!(draw_value: Eval);
    prop!(none_mult: Eval);
    prop!(timeout_win_value: Eval);
    prop!(timeout_draw_value: Eval);
    prop!(timeout_default: Option<Eval>);
    prop!(max_time: Duration);
    prop!(node_max_time: Duration);

    #[inline]
    pub fn eval<E2: EvalFn>(self, value: E2) -> Deepen<E2, I> {
        Deepen::from((self, value, self.intermediary_eval))
    }

    #[inline]
    pub fn intermediary_value<I2: EvalFn>(self, value: I2) -> Deepen<E, I2> {
        Deepen::from((self, self.eval, value))
    }
}

impl<E: EvalFn, I: EvalFn> EvalFn for Deepen<E, I> {
    fn eval<'a>(&self, game: &'a Game, node: &'a TreeNode) -> Option<Eval> {
        if let Some((res, _)) = deepen(game, node, self.depth, &self, Eval::NEG_INFINITY, Eval::INFINITY, Instant::now() + self.max_time) {
            Some(res)
        } else {
            self.timeout_default
        }
    }
}

impl<E: EvalFn, E2: EvalFn, I: EvalFn, I2: EvalFn> From<(Deepen<E, I>, E2, I2)> for Deepen<E2, I2> {
    fn from((other, eval, intermediary_eval): (Deepen<E, I>, E2, I2)) -> Self {
        Self {
            depth: other.depth,
            breadth_wide: other.breadth_wide,
            breadth_narrow: other.breadth_narrow,
            win_value: other.win_value,
            draw_value: other.draw_value,
            none_mult: other.none_mult,
            timeout_win_value: other.timeout_win_value,
            timeout_draw_value: other.timeout_draw_value,
            timeout_default: other.timeout_default,
            max_time: other.max_time,
            node_max_time: other.node_max_time,

            eval: eval,
            intermediary_eval: intermediary_eval,
        }
    }
}

fn deepen<E: EvalFn, I: EvalFn>(game: &Game, node: &TreeNode, depth: usize, settings: &Deepen<E, I>, mut alpha: Eval, beta: Eval, deadline: Instant) -> Option<(Eval, Eval)> {
    deadline.checked_duration_since(Instant::now())?;

    match is_mate(game, &node.partial_game, Some(settings.node_max_time)) {
        Mate::None(ms, pos, iter) => {
            if depth == 0 {
                let res = settings.eval.eval(game, node)?;
                Some((res, res))
            } else {
                let iter = match iter {
                    None => GenLegalMovesetIter::new(game, Cow::Borrowed(&node.partial_game), Some(settings.node_max_time)),
                    Some(i) => i,
                };

                let child_node = TreeNode::extend(node, ms, pos);

                let mut pool = Vec::with_capacity(settings.breadth_wide);
                let eval = settings.intermediary_eval.eval(game, &child_node);
                pool.push((child_node, eval));

                for (ms, pos) in iter.take(settings.breadth_wide - 1) {
                    let child_node = TreeNode::extend(node, ms, pos);
                    let eval = settings.intermediary_eval.eval(game, &child_node);
                    pool.push((child_node, eval));
                }

                pool.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

                let mut best_res: Option<Eval> = None;
                let mut best_score: Option<Eval> = None;

                for (child_node, _int_eval) in pool.into_iter().take(settings.breadth_narrow) {
                    deadline.checked_duration_since(Instant::now())?;
                    let (res, score) = deepen(game, &child_node, depth - 1, settings, -beta, -alpha, deadline)?;
                    match best_score {
                        None => {
                            best_score = Some(-score);
                            best_res = Some(-res);
                        }
                        Some(bs) => {
                            if -score > bs {
                                best_score = Some(-score);
                                best_res = Some(-res);
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

                best_res.map(|b| (b, best_score.unwrap()))
            }
        }
        Mate::Checkmate => Some((-settings.win_value, Eval::NEG_INFINITY)),
        Mate::Stalemate => Some((settings.draw_value, 0.0)),
        Mate::TimeoutCheckmate => Some((-settings.timeout_win_value, Eval::NEG_INFINITY)),
        Mate::TimeoutStalemate => Some((settings.timeout_draw_value, 0.0)),
        Mate::Error => None
    }
}
