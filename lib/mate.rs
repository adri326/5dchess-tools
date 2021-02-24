use crate::*;
use crate::gen::*;
use crate::check::*;
use std::collections::{HashSet};
use std::time::{Duration, Instant};

/**
    Status for checkmate detection.
**/

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Mate {
    /// The position is checkmate: the current player cannot make any moveset and is in check
    Checkmate,
    /// The position is stalemate: the current player cannot make any moveset and isn't in check
    Stalemate,
    /// Error: an error occured while generating moves
    Error,
    /// Timed out while looking for checkmate: the current player is in check and no moveset was found in a timely manner
    TimeoutCheckmate,
    /// Timed out while looking for stalemate: the current player isn't in check and no moveset was found in a timely manner
    TimeoutStalemate,
    /// None: the current player can make a moveset
    None(Moveset),
}

macro_rules! unwrap_mate {
    ( $x:expr ) => {
        match $x {
            Some(x) => x,
            // TODO: uncomment the following line
            // None => return Mate::Error
            None => panic!(),
        }
    };
}

/**
    Checks whether or not the current position is checkmate, stalemate or neither.
    It first tries to make simple branching moves if the current player has branching priority.
    If there is a considerable number of timelines (default is 3), then it will pre-order moves.

    ## Example

    ```
    let game: &Game;
    let partial_game: &PartialGame;

    // Fill in game and partial_game here

    match is_mate(game, partial_game, Some(Duration::new(10, 0))) {
        Mate::None(ms) => {
            println!("Not mate! Player can play {}", ms);
        }
        Mate::Checkmate => {
            println!("Checkmate!");
        }
        Mate::Stalemate => {
            println!("Stalemate!");
        }
        Mate::Error => {
            panic!("Error while looking for checkmate!");
        }
        Mate::TimeoutCheckmate => {
            println!("Timed out while looking for checkmate: probably checkmate.");
        }
        Mate::TimeoutStalemate => {
            println!("Timed out while looking for stalemate: probably stalemate.");
        }
    }
    ```
**/
pub fn is_mate<'a>(
    game: &'a Game,
    partial_game: &'a PartialGame<'a>,
    max_duration: Option<Duration>,
) -> Mate {
    let start = Instant::now();
    let max_duration = max_duration.unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1));
    let idle_boards = unwrap_mate!(generate_idle_boards(game, partial_game));
    let n_timelines = partial_game.info.len_timelines();

    // Here, attacked pieces and attackers are stored as pairs of their coordinate (as they appear after passing along the boards) and whether or not the attack was physical
    // ie. (Coords, physical?)
    let mut attacked_pieces: HashSet<(Coords, bool)> = HashSet::new();
    let mut attackers: HashSet<(Coords, bool)> = HashSet::new();
    let mut attacking_moves: Vec<Move> = Vec::new();

    for board in idle_boards.opponent_boards(game) {
        for mv in unwrap_mate!(board.generate_moves_flag(game, &idle_boards, GenMovesFlag::Check)) {
            if let Some(p) = mv.to.0 {
                if p.is_royal() && p.white == partial_game.info.active_player {
                    let physical = mv.from.1.non_physical() == mv.to.1.non_physical();
                    attacked_pieces.insert((mv.to.1, physical));
                    attackers.insert((mv.from.1, physical));
                    attacking_moves.push(mv);
                }
            }
        }
    }

    // List all boards and moves (we'll need them anyways)
    let own_boards: Vec<&Board> = partial_game.own_boards(game).collect();
    let mut moves: Vec<_> = Vec::new();
    for board in own_boards.iter() {
        moves.push(CacheMoves::new(FilterLegalMove::new(
            unwrap_mate!(board.generate_moves(game, partial_game)),
            game,
            partial_game,
        )));
    }

    // The current player may create inactive timeline, look for branching moves that shift the present back
    if partial_game.info.timeline_advantage(partial_game.info.active_player) > 0 {
        for own_moves in &mut moves {
            for mv in own_moves {
                if mv.from.1.non_physical() != mv.to.1.non_physical()
                    && (mv.to.1).1 < partial_game.info.present
                    && unwrap_mate!(partial_game.info.get_timeline((mv.to.1).0)).last_board
                        != (mv.to.1).1
                {
                    match Moveset::new(vec![mv], &partial_game.info) {
                        Ok(ms) => {
                            if let Some(new_partial_game) =
                                ms.generate_partial_game(game, partial_game)
                            {
                                if !unwrap_mate!(is_illegal(game, &new_partial_game)).0 {
                                    return Mate::None(ms);
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }
                if start.elapsed() > max_duration {
                    if attacked_pieces.len() > 0 {
                        return Mate::TimeoutCheckmate;
                    } else {
                        return Mate::TimeoutStalemate;
                    }
                }
            }
        }
    }

    let moves = if n_timelines > 2 {
        // Build the three danger maps

        // TODO: look at moveset impossibility (corollary 6)
        let mut danger: HashSet<Coords> = HashSet::with_capacity(64 * own_boards.len());

        for board in idle_boards.opponent_boards(game) {
            for mv in unwrap_mate!(board.generate_moves(game, &idle_boards)) {
                if idle_boards.get_board(mv.to.1.non_physical()).is_some() {
                    danger.insert(mv.to.1 - Coords(0, 1, 0, 0));
                }
            }
        }

        // let mut reconstructed_moves: Vec<CacheMoves<Timed<CacheIterOrVec>>> =
        let mut reconstructed_moves: Vec<Vec<Move>> = Vec::with_capacity(moves.len());

        let mut possible_move_white: Vec<bool> = vec![false; partial_game.info.timelines_white.len()];
        let mut possible_move_black: Vec<bool> = vec![false; partial_game.info.timelines_black.len()];

        for (&_board, own_moves) in own_boards.iter().zip(moves.into_iter()) {
            let mut promising_moves: Vec<Move> = Vec::with_capacity(own_moves.cache.len());
            let mut other_moves: Vec<Move> = Vec::with_capacity(own_moves.cache.len());
            let mut unpromising_moves: Vec<Move> = Vec::with_capacity(own_moves.cache.len());

            let (iterator, cache, _) = own_moves.into_raw_parts();
            // Re-order own_moves

            for mv in cache.into_iter().chain(iterator) {
                // Update possible_move_* arrays
                if mv.from.1.l() >= 0 {
                    possible_move_white[mv.from.1.l() as usize] = true;
                } else {
                    possible_move_black[(-mv.from.1.l()) as usize - 1] = true;
                }

                if mv.to.1.l() >= 0 {
                    possible_move_white[mv.to.1.l() as usize] = true;
                } else {
                    possible_move_black[(-mv.to.1.l()) as usize - 1] = true;
                }

                // Push moves to their respective promising/unpromising sets
                if mv.from.0.is_royal() && danger.contains(&mv.to.1) {
                    unpromising_moves.push(mv);
                } else if !mv.from.0.is_royal() && danger.contains(&mv.to.1) {
                    promising_moves.push(mv);
                } else {
                    other_moves.push(mv);
                }
            }

            promising_moves.append(&mut other_moves);
            promising_moves.append(&mut unpromising_moves);

            reconstructed_moves.push(promising_moves);
        }

        // Handle shifting impossibility
        if partial_game.info.timeline_advantage(partial_game.info.active_player) == 0 {
            for l in partial_game.info.min_timeline()..=partial_game.info.max_timeline() {
                if let Some(tl) = partial_game.info.get_timeline(l) {
                    if tl.last_board <= partial_game.info.present && partial_game.info.is_active(l) {
                        if l >= 0 {
                            if !possible_move_white[l as usize] {
                                return Mate::Checkmate
                            }
                        } else {
                            if !possible_move_white[(-l) as usize - 1] {
                                return Mate::Checkmate
                            }
                        }
                    }
                }
            }
        } else if partial_game.info.timeline_debt(!partial_game.info.active_player) > 0 {
            // Handle dead timelines
            let debt = partial_game.info.timeline_debt(!partial_game.info.active_player);

            // Note to an interested code reader about this variable name:
            // I'm doing my best to minimize the number of (king) casualties.
            // This variable represents the limit beyond which deaths occur;
            // you don't want to cross it.
            let mut death_limit: usize = 0;

            for n_branching_moves in 1..=debt {
                if partial_game.info.active_player {
                    let l = partial_game.info.min_timeline() + debt as Layer - n_branching_moves as Layer;
                    if !possible_move_black[(-l) as usize - 1] {
                        break
                    }
                    death_limit = n_branching_moves;
                } else {
                    let l = partial_game.info.max_timeline() - debt as Layer + n_branching_moves as Layer;
                    if !possible_move_white[l as usize] {
                        break
                    }
                    death_limit = n_branching_moves;
                }
            }

            if death_limit < debt {
                // Only retain the moves that aren't branching
                // TODO: pass along death_limit to GenLegalMovesetIter? Moveset::new_shifting is doing a pretty good job though

                if death_limit == 0 {
                    for moves in &mut reconstructed_moves {
                        moves.retain(|mv| {
                            if let Some(tl) = partial_game.info.get_timeline(mv.to.1.l()) {
                                mv.to.1.t() == tl.last_board
                            } else {
                                false
                            }
                        });
                    }

                    // TODO: recalculate possible_moves_*?
                }

                // Ignore spatial moves from inactive timelines
                for moves in &mut reconstructed_moves {
                    moves.retain(|mv| {
                        if partial_game.info.active_player {
                            let min_l = partial_game.info.min_timeline() + debt as Layer - death_limit as Layer;
                            mv.from.1.non_physical() != mv.to.1.non_physical() || mv.from.1.l() >= min_l
                        } else {
                            let max_l = partial_game.info.max_timeline() - debt as Layer + death_limit as Layer;
                            mv.from.1.non_physical() != mv.to.1.non_physical() || mv.from.1.l() <= max_l
                        }
                    })
                }
            }
        }

        reconstructed_moves
            .into_iter()
            .map(|moves| from_vec(moves, max_duration, start))
            .collect()
    } else {
        // Do nothing
        moves
            .into_iter()
            .map(|moves| transfer_cache(moves, max_duration, start))
            .collect()
    };

    let iter = Timed::with_start(
        GenMovesetIter::from_cached_iters(moves, game, partial_game),
        Some(start),
        max_duration,
    );

    // Big boy to look for legal moves
    match iter
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
        .filter_timed(
            |ms| match ms {
                Some((_ms, new_partial_game)) => {
                    for &(pos, _physical) in attackers.iter() {
                        match new_partial_game.get_with_game(game, pos) {
                            Tile::Piece(p) if p.white != partial_game.info.active_player => {
                                for mv in unwrap_mate!(PiecePosition::new(p, pos)
                                    .generate_moves_flag(
                                        game,
                                        new_partial_game,
                                        GenMovesFlag::Check
                                    ))
                                {
                                    if mv.to.0.is_some() && mv.to.0.unwrap().is_royal() {
                                        return false;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if game.width <= MAX_BITBOARD_WIDTH as Physical {
                        !is_illegal_bitboard(game, &new_partial_game).unwrap_or((true, None)).0
                    } else {
                        !is_illegal(game, &new_partial_game).unwrap_or((true, None)).0
                    }
                }
                None => false,
            },
            max_duration - start.elapsed(),
        )
        .next()
    {
        Some(Some((ms, _pos))) => Mate::None(ms),
        Some(None) => Mate::Error, // Unreachable
        None => {
            if start.elapsed() > max_duration {
                if attacked_pieces.len() > 0 {
                    Mate::TimeoutCheckmate
                } else {
                    Mate::TimeoutStalemate
                }
            } else {
                if attacked_pieces.len() > 0 {
                    Mate::Checkmate
                } else {
                    Mate::Stalemate
                }
            }
        }
    }
}

/// Internal enum used by `is_mate` to determine that a position is mate.
/// It works as an iterator that is passed to CacheMoves and put into GenMovesetIter
enum CacheIterOrVec<'a> {
    Iter(FilterLegalMove<'a, BoardIter<'a>>),
    Vec,
}

impl<'a> Iterator for CacheIterOrVec<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CacheIterOrVec::Iter(i) => i.next(),
            CacheIterOrVec::Vec => None,
        }
    }
}

/// Turns a CacheMoves<FilterLegalMove> into a CacheMoves<Timed<CacheIterOrVec::Iter>>, keeping the cache of the former
fn transfer_cache<'a>(
    iter: CacheMoves<FilterLegalMove<'a, BoardIter<'a>>>,
    duration: Duration,
    start: Instant,
) -> CacheMoves<Timed<CacheIterOrVec<'a>>> {
    let (iter, cache, done) = iter.into_raw_parts();
    CacheMoves::from_raw_parts(
        Timed::with_start(CacheIterOrVec::Iter(iter), Some(start), duration),
        cache,
        done,
    )
}

/// Turns a Vec<Move> into a CacheMoves<Timed<CacheIterOrVec::Vec>>
fn from_vec<'a>(
    vec: Vec<Move>,
    duration: Duration,
    start: Instant,
) -> CacheMoves<Timed<CacheIterOrVec<'a>>> {
    CacheMoves::from_raw_parts(
        Timed::with_start(CacheIterOrVec::Vec, Some(start), duration),
        vec,
        true,
    )
}
