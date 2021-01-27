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
use std::marker::PhantomData;

pub mod piece;
pub use piece::PiecePosition;

pub mod board;

pub mod cache;
pub use cache::CacheMoves;

pub mod moveset;
pub use moveset::{
    generate_movesets_filter_strategy, generate_movesets_iterator_strategy, GenMovesetIter,
};

/**
    An enum containing the different flags used by `GenMoves::generate_moves_flag.`
    Each flag allows you to only yield a subset of the moves, except for `Any`.
    See the documentation for each of them for more details.

    If you wish to add your own flag, consider contacting the repository owner.
**/
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GenMovesFlag {
    /** Tells `GenMoves::generate_moves_flag` to yield all of the moves. **/
    Any,

    /**
        Tells `GenMoves::generate_moves_flag` to only yield the moves attacking a royal pieces.

        - Moves that aren't attacking royal pieces *may* be present.
        - Moves that are attacking royal pieces *must all* be present.
        Failure to do so will result in undefined behavior.
    **/
    Check,
}

/**
    A trait representing the ability for an object (a board, a move cache, a piece, etc.) to generate a list of moves, as an iterator.
**/
pub trait GenMoves<'a, B: Clone + AsRef<Board>>: Sized {
    type Iter: Iterator<Item = Move> + Clone;

    /**
        Returns the iterator that yields all of the moves.
    **/
    #[inline]
    fn generate_moves(
        self,
        game: &'a Game,
        partial_game: &'a PartialGame<'a, B>,
    ) -> Option<Self::Iter> {
        self.generate_moves_flag(game, partial_game, GenMovesFlag::Any)
    }

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

    /**
        Returns an iterator that may only yield a subset of the moves.
        See `GenMovesFlag` for more detail on the flags used.

        You should consider implementing your own `generate_moves_flag` if you can.
        When doing so, you should bundle the different iterators within an `enum`.
    **/
    #[inline]
    fn generate_moves_flag(
        self,
        game: &'a Game,
        partial_game: &'a PartialGame<'a, B>,
        _flag: GenMovesFlag,
    ) -> Option<Self::Iter> {
        self.generate_moves(game, partial_game)
    }
}

#[derive(Clone)]
pub struct GenMovesStrategy<'a, B, T>
where
    B: Clone + AsRef<Board>,
    T: GenMoves<'a, B> + Clone,
{
    _b: PhantomData<&'a B>,
    _t: PhantomData<*const T>,
}

impl<'a, B, T> GenMovesStrategy<'a, B, T>
where
    B: Clone + AsRef<Board>,
    T: GenMoves<'a, B> + Clone,
{
    pub fn new() -> Self {
        Self {
            _b: PhantomData,
            _t: PhantomData,
        }
    }
}

impl<'a, B, T> Strategy<'a, B> for GenMovesStrategy<'a, B, T>
where
    B: Clone + AsRef<Board>,
    T: GenMoves<'a, B> + Clone,
{
    type From = T;
    type To = <T as GenMoves<'a, B>>::Iter;

    fn apply(
        &self,
        generator: T,
        game: &'a Game,
        partial_game: &'a PartialGame<'a, B>,
    ) -> Option<Self::To> {
        generator.generate_moves(game, partial_game)
    }
}
