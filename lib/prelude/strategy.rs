use super::*;
use std::marker::PhantomData;

pub trait Strategy<'a>: Sized + Clone {
    type From;
    type To;

    fn apply(
        &self,
        from: Self::From,
        game: &'a Game,
        partial_game: &'a PartialGame<'a>,
    ) -> Option<Self::To>;
}

#[derive(Clone, Copy)]
pub struct IdentityStrategy<T: Clone> {
    _phantom: PhantomData<*const T>,
}

impl<T: Clone> IdentityStrategy<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: Clone> Strategy<'a> for IdentityStrategy<T> {
    type From = T;
    type To = T;

    fn apply(&self, from: T, _game: &'a Game, _partial_game: &'a PartialGame<'a>) -> Option<T> {
        Some(from)
    }
}

/**
    A strategy that will always return true.
**/
#[derive(Clone, Copy)]
pub struct TrueStrategy<F> {
    _phantom: PhantomData<*const F>,
}

impl<F> TrueStrategy<F> {
    pub fn new() -> Self {
        TrueStrategy {
            _phantom: PhantomData,
        }
    }
}

impl<'a, F> Strategy<'a> for TrueStrategy<F>
where
    F: Copy,
{
    type From = F;
    type To = bool;

    fn apply(&self, _from: F, _game: &'a Game, _partial_game: &'a PartialGame<'a>) -> Option<bool> {
        Some(true)
    }
}

/**
    A strategy that will always return false.
**/
#[derive(Clone, Copy)]
pub struct FalseStrategy<F> {
    _phantom: PhantomData<*const F>,
}

impl<F> FalseStrategy<F> {
    pub fn new() -> Self {
        FalseStrategy {
            _phantom: PhantomData,
        }
    }
}

impl<'a, F> Strategy<'a> for FalseStrategy<F>
where
    F: Copy,
{
    type From = F;
    type To = bool;

    fn apply(&self, _from: F, _game: &'a Game, _partial_game: &'a PartialGame<'a>) -> Option<bool> {
        Some(false)
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
#[derive(Clone, Copy)]
pub struct AndStrategy<F, Left, Right>
where
    F: Copy,
{
    _f: PhantomData<*const F>,
    left: Left,
    right: Right,
}

impl<F, Left, Right> AndStrategy<F, Left, Right>
where
    F: Copy,
{
    pub fn new(left: Left, right: Right) -> Self {
        Self {
            _f: PhantomData,
            left,
            right,
        }
    }
}

impl<'a, F, Left, Right> Strategy<'a> for AndStrategy<F, Left, Right>
where
    F: Copy,
    Left: Strategy<'a, From = F, To = bool>,
    Right: Strategy<'a, From = F, To = bool>,
{
    type From = F;
    type To = bool;

    fn apply(&self, from: F, game: &'a Game, partial_game: &'a PartialGame<'a>) -> Option<bool> {
        // and := λleft, λright, (left (right true))
        match self.left.apply(from, game, partial_game) {
            Some(true) => self.right.apply(from, game, partial_game),
            Some(false) => Some(false),
            None => None,
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
#[derive(Clone, Copy)]
pub struct OrStrategy<F, Left, Right>
where
    F: Copy,
{
    _f: PhantomData<*const F>,
    left: Left,
    right: Right,
}

impl<F, Left, Right> OrStrategy<F, Left, Right>
where
    F: Copy,
{
    pub fn new(left: Left, right: Right) -> Self {
        Self {
            _f: PhantomData,
            left,
            right,
        }
    }
}

impl<'a, F, Left, Right> Strategy<'a> for OrStrategy<F, Left, Right>
where
    F: Copy,
    Left: Strategy<'a, From = F, To = bool>,
    Right: Strategy<'a, From = F, To = bool>,
{
    type From = F;
    type To = bool;

    fn apply(&self, from: F, game: &'a Game, partial_game: &'a PartialGame<'a>) -> Option<bool> {
        // or := λleft, λright, (left true right)
        match self.left.apply(from, game, partial_game) {
            Some(false) => self.right.apply(from, game, partial_game),
            Some(true) => Some(true),
            None => None,
        }
    }
}
