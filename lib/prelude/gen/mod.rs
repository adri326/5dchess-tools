/*! # Movement generation utilities

    This module allows you to generate an iterator of the possible moves from pieces or boards.
    If you are making your own board type, then you should consider implementing the `GenMoves` trait.

    ## Example

    ```
    let position = Coords(0, 0, 1, 0);
    let piece = PiecePosition::new(game.get(position).piece().unwrap(), position);

    // This loop will now print all of the moves that the `c1`-knight can make
    for mv in piece.generate_moves(game, &no_partial_game(game)).unwrap() {
        println!("{:?}", mv);
    }

    // This loop will print all of the moves that white can make as their first move
    for mv in game.get_board((0, 0)).unwrap().generate_moves(game, &no_partial_game(game)).unwrap {
        println!("{:?}", mv);
    }
    ```
*/
use super::*;

pub mod piece;
pub use piece::PiecePosition;

pub mod board;

pub mod cache;
pub use cache::CacheMoves;

pub trait GenMoves<'a, B: Clone + AsRef<Board> + 'a>: Sized {
    type Iter: Iterator<Item = Move>;

    /**
        Returns the iterator that yields all of the moves.
    **/
    fn generate_moves(self, game: &'a Game, partial_game: &'a PartialGame<'a, B>) -> Option<Self::Iter>;

    /**
        Returns true if `mv` is a valid move. The default implementation traverses the iterator yielded by `generate_moves` and checks
        that there is a matching move.

        You should consider implementing your own `validate_move` if you can.
    **/
    fn validate_move(self, game: &'a Game, partial_game: &'a PartialGame<B>, mv: &Move) -> bool {
        self.generate_moves(game, partial_game)
            .map(|mut i| i.find(|m| m == mv))
            .flatten()
            .is_some()
    }
}
