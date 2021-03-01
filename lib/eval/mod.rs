use crate::prelude::*;
use crate::tree::TreeNode;

pub mod wdl;

pub trait EvalFn : Copy {
    type Output: PartialOrd + std::ops::Neg<Output=Self::Output> + Copy + std::fmt::Debug;

    const MIN: Self::Output;
    const DRAW: Self::Output;

    fn eval<'a>(&self, game: &'a Game, node: &'a TreeNode) -> Option<Self::Output>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoEvalFn();

impl NoEvalFn {
    pub fn new() -> Self {
        Self()
    }
}

impl EvalFn for NoEvalFn {
    type Output = f32;

    const MIN: Self::Output = Self::Output::NEG_INFINITY;
    const DRAW: Self::Output = 0.0;

    fn eval<'a>(&self, _game: &'a Game, _node: &'a TreeNode) -> Option<Self::Output> {
        Some(0.0)
    }
}
