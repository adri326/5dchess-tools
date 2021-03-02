use crate::prelude::*;
use crate::tree::TreeNode;

pub mod wdl;

pub type Eval = f32;

pub trait EvalFn : Copy {
    fn eval<'a>(&self, game: &'a Game, node: &'a TreeNode) -> Option<Eval>;
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
