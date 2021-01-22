use crate::prelude::*;
use super::*;

// This is a sample strategy
pub struct NoCastling;

impl<'a, B> Strategy<'a, B> for NoCastling
where
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
{
    type From = Move;
    type To = bool;

    fn apply(mv: Move, _game: &'a Game, _partial_game: &'a PartialGame<'a, B>) -> Option<bool> {
        Some(mv.kind != MoveKind::Castle)
    }
}
