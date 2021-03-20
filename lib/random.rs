use crate::check::*;
use crate::gen::*;
use crate::*;
use rand::seq::SliceRandom;
use std::borrow::Cow;
use std::time::Duration;

/**
    Error value returned by `random_legal_moveset`, should it find mate, error out of time out.
**/
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RandomLegalMovesetReason {
    Checkmate,
    Stalemate,
    Error,
    TimeoutCheckmate,
    TimeoutStalemate,
}

/**
    Returns a random, legal moveset. This function can be used within a monte carlo-like tree search.
    This algorithm works by shuffling the moves for each boards and the boards, before feeding them to
    `GenLegalMovesetIter`.
    This means that performances are slightly worse than `GenLegalMovesetIter`, as promising moves aren't considered first.
**/
pub fn random_legal_moveset<'a>(
    game: &'a Game,
    partial_game: &'a PartialGame<'a>,
    max_duration: Option<Duration>,
) -> Result<(Moveset, PartialGame<'a>), RandomLegalMovesetReason> {
    let mut rng = rand::thread_rng();
    let mut active_boards: Vec<&Board> = partial_game
        .own_boards(game)
        .filter(|b| b.active(&partial_game.info))
        .collect();
    let mut inactive_boards: Vec<&Board> = partial_game
        .own_boards(game)
        .filter(|b| !b.active(&partial_game.info))
        .collect();
    active_boards.shuffle(&mut rng);
    inactive_boards.shuffle(&mut rng);
    let boards: Vec<&Board> = active_boards
        .into_iter()
        .chain(inactive_boards.into_iter())
        .collect();
    let mut iters = Vec::new();

    for board in boards {
        let mut boards: Vec<_> = Vec::new();
        let mut iter = FilterLegalMove::new(
            board
                .generate_moves(game, partial_game)
                .ok_or(RandomLegalMovesetReason::Error)?,
            game,
            partial_game,
        );

        let mut moves: Vec<_> = Vec::new();
        for mv in &mut iter {
            moves.push(mv);
        }
        moves.shuffle(&mut rng);

        for mv in &moves {
            let src = mv
                .generate_source_board(game, partial_game)
                .ok_or(RandomLegalMovesetReason::Error)?;
            if mv.is_jump() {
                let dst = mv
                    .generate_target_board(game, partial_game)
                    .ok_or(RandomLegalMovesetReason::Error)?;
                boards.push((src, Some(dst)));
            } else {
                boards.push((src, None));
            }
        }

        iters.push(CacheMovesBoards::from_raw_parts((
            iter,
            moves,
            boards,
            true,
            game,
            partial_game,
        )));
    }

    let mut iter =
        GenLegalMovesetIter::with_iterators(game, Cow::Borrowed(partial_game), iters, max_duration);

    match iter.next() {
        Some((ms, pos)) => Ok((ms, pos)),
        _ => {
            if iter.timed_out() {
                match is_in_check(game, partial_game) {
                    Some((true, _)) => Err(RandomLegalMovesetReason::TimeoutCheckmate),
                    Some((false, _)) => Err(RandomLegalMovesetReason::TimeoutStalemate),
                    None => Err(RandomLegalMovesetReason::Error),
                }
            } else {
                match is_in_check(game, partial_game) {
                    Some((true, _)) => Err(RandomLegalMovesetReason::Checkmate),
                    Some((false, _)) => Err(RandomLegalMovesetReason::Stalemate),
                    None => Err(RandomLegalMovesetReason::Error),
                }
            }
        }
    }
}
