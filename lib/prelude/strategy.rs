use super::*;
use std::marker::PhantomData;

pub trait Strategy<'a, B: Clone + AsRef<Board> + 'a> {
    type From;
    type To;

    fn apply<'b>(from: Self::From, game: &'b Game, partial_game: &'b PartialGame<'b, B>) -> Option<Self::To>;
}

pub struct DefaultStrategy<T> {
    _phantom: PhantomData<*const T>,
}

impl<'a, B: Clone + AsRef<Board> + 'a, T> Strategy<'a, B> for DefaultStrategy<T> {
    type From = T;
    type To = T;

    fn apply<'b>(from: T, _game: &'b Game, _partial_game: &'b PartialGame<'b, B>) -> Option<T> {
        Some(from)
    }
}

/**
    A strategy combining two strategies into one strategy, representing the conjunction of both strategies.
    Both strategies are expected to have a common type `F` as input and `bool` as output.
    The type `F` must be copiable; note that
    [`&T` implements `Copy` regardless of the type `T`](https://doc.rust-lang.org/std/primitive.reference.html#trait-implementations-1)

    ## Example

    ```
    generate_movesets_filter_strategy<
        AndStrategy<Board, Move, NoCastling, LegalMove>
    >(...); // will only yield the legal movesets with no castling
    ```
**/
pub struct AndStrategy<'a, B, F, Left, Right>
where
    B: Clone + AsRef<Board> + 'a,
    F: Copy,
    Left: Strategy<'a, B, From=F, To=bool>,
    Right: Strategy<'a, B, From=F, To=bool>,
{
    _b: PhantomData<&'a B>,
    _left: PhantomData<*const Left>,
    _right: PhantomData<*const Right>,
}

impl<'a, B, F, Left, Right> Strategy<'a, B> for AndStrategy<'a, B, F, Left, Right>
where
    B: Clone + AsRef<Board> + 'a,
    F: Copy,
    Left: Strategy<'a, B, From=F, To=bool>,
    Right: Strategy<'a, B, From=F, To=bool>,
{
    type From = F;
    type To = bool;

    fn apply<'b>(from: F, game: &'b Game, partial_game: &'b PartialGame<'b, B>) -> Option<bool> {
        // and := λleft, λright, (left (right true))
        match Left::apply(from, game, partial_game) {
            Some(true) => Right::apply(from, game, partial_game),
            Some(false) => Some(false),
            None => None
        }
    }
}

/**
    A strategy combining two strategies into one strategy, representing the disjunction of both strategies.
    Both strategies are expected to have a common type `F` as input and `bool` as output.
    The type `F` must be copiable; note that
    [`&T` implements `Copy` regardless of the type `T`](https://doc.rust-lang.org/std/primitive.reference.html#trait-implementations-1)

    ## Example

    ```
    generate_movesets_filter_strategy<
        OrStrategy<Board, Move, NoCastling, LegalMove>
    >(...); // will only yield the movesets with moves being either legal, or not castling moves
    ```
**/
pub struct OrStrategy<'a, B, F, Left, Right>
where
    B: Clone + AsRef<Board> + 'a,
    F: Copy,
    Left: Strategy<'a, B, From=F, To=bool>,
    Right: Strategy<'a, B, From=F, To=bool>,
{
    _b: PhantomData<&'a B>,
    _left: PhantomData<*const Left>,
    _right: PhantomData<*const Right>,
}

impl<'a, B, F, Left, Right> Strategy<'a, B> for OrStrategy<'a, B, F, Left, Right>
where
    B: Clone + AsRef<Board> + 'a,
    F: Copy,
    Left: Strategy<'a, B, From=F, To=bool>,
    Right: Strategy<'a, B, From=F, To=bool>,
{
    type From = F;
    type To = bool;

    fn apply<'b>(from: F, game: &'b Game, partial_game: &'b PartialGame<'b, B>) -> Option<bool> {
        // or := λleft, λright, (left true right)
        match Left::apply(from, game, partial_game) {
            Some(false) => Right::apply(from, game, partial_game),
            Some(true) => Some(true),
            None => None
        }
    }
}
