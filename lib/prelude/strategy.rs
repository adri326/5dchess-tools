use super::*;

pub trait Strategy<'a, B: Clone + AsRef<Board> + 'a> {
    type From;
    type To;

    fn apply(from: Self::From, game: &'a Game, partial_game: &'a PartialGame<'a, B>) -> Option<Self::To>;
}

pub struct DefaultStrategy<T> {
    _phantom: std::marker::PhantomData<*const T>,
}

impl<'a, B: Clone + AsRef<Board> + 'a, T> Strategy<'a, B> for DefaultStrategy<T> {
    type From = T;
    type To = T;

    fn apply(from: T, _game: &'a Game, _partial_game: &'a PartialGame<'a, B>) -> Option<T> {
        Some(from)
    }
}

// This is a sample strategy
pub struct NoCastling;

fn no_castling(mv: &Move) -> bool {
    mv.kind != MoveKind::Castle
}

impl<'a, B> Strategy<'a, B> for NoCastling
where
    B: Clone + AsRef<Board> + 'a,
    &'a B: GenMoves<'a, B>
{
    type From = BoardOr<'a, B>;
    type To = std::iter::Filter<<BoardOr<'a, B> as GenMoves<'a, B>>::Iter, fn(&Move) -> bool>;

    fn apply(from: Self::From, game: &'a Game, partial_game: &'a PartialGame<'a, B>) -> Option<Self::To> {
        let iter = from.generate_moves(game, partial_game)?;
        Some(iter.filter(no_castling))
    }
}
