use crate::*;
use crate::gen::*;
use crate::check::*;
use rand::seq::SliceRandom;
use std::time::Duration;

// Goals

// Random movesets, useful for MCTS

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RandomLegalMovesetReason {
    Checkmate,
    Stalemate,
    Error,
    TimeoutCheckmate,
    TimeoutStalemate,
}

// TODO: make this less bad
pub fn random_legal_moveset<'a>(
    game: &'a Game,
    partial_game: &'a PartialGame<'a>,
    duration: Option<Duration>,
) -> Result<(Moveset, PartialGame<'a>), RandomLegalMovesetReason> {
    let duration = duration.unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1));
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
        let mut moves: Vec<_> = FilterLegalMove::new(
            board
                .generate_moves(game, partial_game)
                .ok_or(RandomLegalMovesetReason::Error)?,
            game,
            partial_game,
        )
        .collect();
        moves.shuffle(&mut rng);
        iters.push(CacheMoves::new(moves.into_iter()));
    }

    let mut iter = SigmaFilter::new(
        GenMovesetIter::from_cached_iters(iters, game, partial_game)
            .filter_timed(|_iter| true, duration)
            .flatten()
            .map(|ms| {
                let res = match ms {
                    Ok(ms) => match ms.generate_partial_game(game, partial_game) {
                        Some(new_partial_game) => Some((ms, new_partial_game)),
                        None => None,
                    },
                    Err(_) => None,
                };
                res
            })
            .filter_timed(|x| x.is_some(), duration),
        |opt| {
            let (_ms, new_partial_game) = opt.as_ref().unwrap();
            !is_illegal(game, &new_partial_game).unwrap_or((true, None)).0
        },
        duration,
    );

    match iter.next() {
        Some(Some(x)) => Ok(x),
        _ => {
            if iter.timed_out() {
                match generate_idle_boards(game, partial_game) {
                    Some(idle_partial_game) => match is_threatened(game, &idle_partial_game) {
                        Some((true, _)) => Err(RandomLegalMovesetReason::TimeoutCheckmate),
                        Some((false, _)) => Err(RandomLegalMovesetReason::TimeoutStalemate),
                        None => Err(RandomLegalMovesetReason::Error),
                    },
                    None => Err(RandomLegalMovesetReason::Error),
                }
            } else {
                match generate_idle_boards(game, partial_game) {
                    Some(idle_partial_game) => match is_threatened(game, &idle_partial_game) {
                        Some((true, _)) => Err(RandomLegalMovesetReason::Checkmate),
                        Some((false, _)) => Err(RandomLegalMovesetReason::Stalemate),
                        None => Err(RandomLegalMovesetReason::Error),
                    },
                    None => Err(RandomLegalMovesetReason::Error),
                }
            }
        }
    }
}
