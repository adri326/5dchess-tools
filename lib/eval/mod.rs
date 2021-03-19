use crate::prelude::*;
use crate::tree::TreeNode;

pub mod wdl;

pub mod value;
pub use value::PieceValues;

pub mod king_safety;
pub use king_safety::KingSafety;

pub mod timeline_advantage;
pub use timeline_advantage::TimelineAdvantage;

pub mod pawn_progression;
pub use pawn_progression::PawnProgression;

pub mod deepen;
pub use deepen::Deepen;

pub type Eval = f32;

// TODO: move to prelude?

pub trait EvalFn: Copy + Send {
    fn eval(&self, game: &Game, node: &TreeNode) -> Option<Eval>;

    fn add<F: EvalFn>(self, other: F) -> SumFn<Self, F> {
        SumFn(self, other)
    }
}

pub trait EvalBoardFn: Copy + Send {
    fn eval_board(&self, game: &Game, node: &TreeNode, board: &Board) -> Option<Eval>;

    fn add<F: EvalBoardFn>(self, other: F) -> SumBoardFn<Self, F> {
        SumBoardFn(self, other)
    }

    fn into_eval(self) -> EvalBoardFnToEvalFn<Self> {
        EvalBoardFnToEvalFn(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvalBoardFnToEvalFn<F: EvalBoardFn>(pub F);

impl<F: EvalBoardFn> EvalFn for EvalBoardFnToEvalFn<F> {
    fn eval(&self, game: &Game, node: &TreeNode) -> Option<Eval> {
        let partial_game = &node.partial_game;
        let mut sum: Eval = 0.0;

        for board in partial_game.own_boards(game).chain(partial_game.opponent_boards(game)) {
            sum += self.0.eval_board(game, node, board)?;
        }

        Some(sum)
    }
}

impl<F: EvalBoardFn> From<F> for EvalBoardFnToEvalFn<F> {
    fn from(f: F) -> Self {
        Self(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoEvalFn;

impl NoEvalFn {
    pub fn new() -> Self {
        Self
    }
}

impl EvalFn for NoEvalFn {
    fn eval(&self, _game: &Game, _node: &TreeNode) -> Option<Eval> {
        Some(0.0)
    }
}

impl EvalBoardFn for NoEvalFn {
    fn eval_board(&self, _game: &Game, _node: &TreeNode, _board: &Board) -> Option<Eval> {
        Some(0.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SumFn<A: EvalFn, B: EvalFn>(pub A, pub B);

impl<F: EvalFn, G: EvalFn> EvalFn for SumFn<F, G> {
    #[inline]
    fn eval(&self, game: &Game, node: &TreeNode) -> Option<Eval> {
        Some(self.0.eval(game, node)? + self.1.eval(game, node)?)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SumBoardFn<A: EvalBoardFn, B: EvalBoardFn>(pub A, pub B);

impl<F: EvalBoardFn, G: EvalBoardFn> EvalBoardFn for SumBoardFn<F, G> {
    #[inline]
    fn eval_board(&self, game: &Game, node: &TreeNode, board: &Board) -> Option<Eval> {
        Some(self.0.eval_board(game, node, board)? + self.1.eval_board(game, node, board)?)
    }
}
