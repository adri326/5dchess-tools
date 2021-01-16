use super::*;

pub trait Strategy<'a, B: Clone + AsRef<Board> + 'a> {
    type From;
    type To;

    fn apply(from: Self::From, game: &Game, partial_game: &PartialGame<'a, B>) -> Option<Self::To>;
}

pub struct DefaultStrategy<T> {
    _phantom: std::marker::PhantomData<*const T>,
}

impl<'a, B: Clone + AsRef<Board> + 'a, T> Strategy<'a, B> for DefaultStrategy<T> {
    type From = T;
    type To = T;

    fn apply(from: T, _game: &Game, _partial_game: &PartialGame<'a, B>) -> Option<T> {
        Some(from)
    }
}
