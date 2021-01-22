use crate::prelude::*;
use super::*;

// This is a sample strategy
pub struct NoCastling;

impl<'a, B> Strategy<'a, B> for NoCastling
where
    B: Clone + AsRef<Board> + 'a,
    for<'b> &'b B: GenMoves<'b, B>,
{
    type From = Move;
    type To = bool;

    fn apply<'b>(mv: Move, _game: &'b Game, _partial_game: &'b PartialGame<'b, B>) -> Option<bool> {
        Some(mv.kind != MoveKind::Castle)
    }
}
