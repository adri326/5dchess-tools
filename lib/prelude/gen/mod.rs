//! Movement generation utilities
use super::*;

pub mod piece;
pub use piece::PiecePosition;

pub trait GenMoves<'a, I: Iterator<Item = Move> + 'a, B: Clone + AsRef<Board> + 'a> {
    fn generate_moves(&'a self, game: &'a Game, partial_game: &'a PartialGame<B>) -> Option<I>;
    fn validate_move(&'a self, game: &Game, partial_game: &PartialGame<B>, mv: &Move) -> bool;
}
