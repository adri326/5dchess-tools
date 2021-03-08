use crate::prelude::*;
use crate::tree::TreeNode;

pub mod wdl;

pub mod value;
pub use value::PieceValues;

pub mod king_safety;
pub use king_safety::KingSafety;

pub mod timeline_advantage;
pub use timeline_advantage::TimelineAdvantage;

pub type Eval = f32;

pub trait EvalFn : Copy + Send {
    fn eval<'a>(&self, game: &'a Game, node: &'a TreeNode) -> Option<Eval>;

    fn add<'a, F: EvalFn>(self, other: F) -> SumFn<Self, F> {
        SumFn(self, other)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoEvalFn();

impl NoEvalFn {
    pub fn new() -> Self {
        Self()
    }
}

impl EvalFn for NoEvalFn {
    fn eval<'a>(&self, _game: &'a Game, _node: &'a TreeNode) -> Option<Eval> {
        Some(0.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SumFn<A: EvalFn, B: EvalFn>(pub A, pub B);

impl<F: EvalFn, G: EvalFn> EvalFn for SumFn<F, G> {
    fn eval<'a>(&self, game: &'a Game, node: &'a TreeNode) -> Option<Eval> {
        Some(self.0.eval(game, node)? + self.1.eval(game, node)?)
    }
}
