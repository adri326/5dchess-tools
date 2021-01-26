use crate::prelude::*;
use super::*;

/// This is a sample strategy which forbids castling moves
#[derive(Clone, Copy)]
pub struct NoCastling;

impl NoCastling {
    pub fn new() -> Self {
        Self
    }
}

impl<'a, B> Strategy<'a, B> for NoCastling
where
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
{
    type From = Move;
    type To = bool;

    fn apply(&self, mv: Move, _game: &'a Game, _partial_game: &'a PartialGame<'a, B>) -> Option<bool> {
        Some(mv.kind != MoveKind::Castle)
    }
}

/// Strategy that forbids time travel (when a piece travels to a past board).
/// Jumps will still be allowed and branching moves emerging from jumps to just-played boards might happen.
/// If you wish to entirely forbid branching moves, then use this with the [`NoBranching` goal](../goals/misc.rs#no-branching).
#[derive(Clone, Copy)]
pub struct NoTimeTravel;

impl NoTimeTravel {
    pub fn new() -> Self {
        Self
    }
}

impl<'a, B> Strategy<'a, B> for NoTimeTravel
where
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
{
    type From = Move;
    type To = bool;

    fn apply(&self, mv: Move, _game: &'a Game, partial_game: &'a PartialGame<'a, B>) -> Option<bool> {
        if mv.from.1.non_physical() == mv.to.1.non_physical() {
            Some(true)
        } else {
            partial_game.is_last_board(mv.to.1.non_physical())
        }
    }
}
