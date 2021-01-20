use super::*;
use std::collections::HashMap;
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

/**
    A strategy combining two strategies into one strategy, representing the conjunction of both strategies.
    Both strategies are expected to have a common type `F` as input and `bool` as output.
    The type `F` must be copiable; note that
    [`&T` implements `Copy` regardless of the type `T`](https://doc.rust-lang.org/std/primitive.reference.html#trait-implementations-1)

    ## Example

    ```
    generate_movesets_with_strategy<
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
    generate_movesets_with_strategy<
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


// TODO: somehow store the boards in the move itself for speed~
// (I don't want to waste another 500ns)
pub struct LegalMove;
pub struct OptLegalMove;

impl<'a, B> Strategy<'a, B> for LegalMove
where
    B: Clone + AsRef<Board> + 'a,
    for<'b> B: From<(Board, &'b Game, &'b PartialGame<'b, B>)>,
    for<'b> &'b B: GenMoves<'b, B>,
{
    type From = Move;
    type To = bool;

    fn apply<'b>(mv: Move, game: &'b Game, partial_game: &'b PartialGame<'b, B>) -> Option<bool> {
        let mut new_partial_game = PartialGame::new(HashMap::new(), partial_game.info.clone(), None);
        mv.generate_partial_game(game, partial_game, &mut new_partial_game);
        new_partial_game.parent = Some(partial_game);

        is_legal_move(game, &new_partial_game)
    }
}


impl<'a, B> Strategy<'a, B> for OptLegalMove
where
    B: Clone + AsRef<Board> + 'a,
    for<'b> B: From<(Board, &'b Game, &'b PartialGame<'b, B>)>,
    for<'b> &'b B: GenMoves<'b, B>,
{
    type From = Move;
    type To = bool;

    fn apply<'b>(mv: Move, game: &'b Game, partial_game: &'b PartialGame<'b, B>) -> Option<bool> {
        let n_own_boards = partial_game.own_boards(game).count();
        if n_own_boards <= 2 {
            Some(true)
        } else if n_own_boards == 3 {
            let n_opponent_boards = partial_game.opponent_boards(game).count();
            Some(n_opponent_boards <= 8)
        } else {
            LegalMove::apply(mv, game, partial_game)
        }
    }
}

fn is_legal_move<'a, B>(game: &'a Game, partial_game: &'a PartialGame<'a, B>) -> Option<bool>
where
    B: Clone + AsRef<Board> + 'a,
    for<'b> B: From<(Board, &'b Game, &'b PartialGame<'b, B>)>,
    for<'b> &'b B: GenMoves<'b, B>,
{
    for board in partial_game.opponent_boards(game) {
        for mv in board.generate_moves_flag(game, partial_game, GenMovesFlag::Check)? {
            match mv.to.0 {
                Some(piece) => {
                    if piece.is_royal() && piece.white == partial_game.info.active_player {
                        return Some(false)
                    }
                }
                None => {}
            }
        }
    }

    Some(true)
}
