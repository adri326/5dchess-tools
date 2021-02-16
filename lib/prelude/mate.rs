use super::*;
use std::collections::{HashSet};
use std::time::{Duration, Instant};

/**
    Status for checkmate detection.
**/

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Mate {
    Checkmate,
    Stalemate,
    Error,
    TimeoutCheckmate,
    TimeoutStalemate,
    None(Moveset),
}

macro_rules! unwrap_mate {
    ( $x:expr ) => {
        match $x {
            Some(x) => x,
            None => panic!(),
        }
    };
}

/**
    Checks whether or not the current position is checkmate, stalemate or none of those.
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
    if partial_game
        .info
        .timeline_advantage(partial_game.info.active_player)
        > 0
    {
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
        let mut danger: HashSet<Coords> = HashSet::with_capacity(64 * own_boards.len());

        for board in idle_boards.opponent_boards(game) {
            for mv in unwrap_mate!(board.generate_moves(game, &idle_boards)) {
                if idle_boards.get_board(mv.to.1.non_physical()).is_some() {
                    danger.insert(mv.to.1 - Coords(0, 1, 0, 0));
                }
            }
        }

        let mut reconstructed_moves: Vec<CacheMoves<Timed<CacheIterOrVec>>> =
            Vec::with_capacity(moves.len());

        for (&board, own_moves) in own_boards.iter().zip(moves.into_iter()) {
            let mut promising_moves: Vec<Move> = Vec::with_capacity(own_moves.cache.len());
            let mut other_moves: Vec<Move> = Vec::with_capacity(own_moves.cache.len());
            let mut unpromising_moves: Vec<Move> = Vec::with_capacity(own_moves.cache.len());

            if own_moves.done() {
                // Re-order own_moves

                for mv in &own_moves.cache {
                    if mv.from.0.is_royal() && danger.contains(&mv.to.1) {
                        unpromising_moves.push(*mv);
                    } else if !mv.from.0.is_royal() && danger.contains(&mv.to.1) {
                        promising_moves.push(*mv);
                    } else {
                        other_moves.push(*mv);
                    }
                }
            } else {
                // Re-generate own_moves using the danger map
                for mv in unwrap_mate!(board.generate_moves(game, partial_game)) {
                    if mv.from.0.is_royal() && danger.contains(&mv.to.1) {
                        unpromising_moves.push(mv);
                    } else if !mv.from.0.is_royal() && danger.contains(&mv.to.1) {
                        promising_moves.push(mv);
                    } else {
                        other_moves.push(mv);
                    }
                }
            }

            promising_moves.append(&mut other_moves);
            promising_moves.append(&mut unpromising_moves);

            if own_moves.done() {
                reconstructed_moves.push(with_new_cache(
                    own_moves,
                    promising_moves,
                    max_duration,
                    start,
                ));
            } else {
                reconstructed_moves.push(from_vec(
                    promising_moves,
                    game,
                    partial_game,
                    max_duration,
                    start,
                ));
            }
        }

        reconstructed_moves
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
                    !is_illegal(game, &new_partial_game).unwrap_or((true, None)).0
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

enum CacheIterOrVec<'a> {
    Iter(std::iter::Fuse<FilterLegalMove<'a, BoardIter<'a>>>),
    Vec(FilterLegalMove<'a, std::vec::IntoIter<Move>>),
}

impl<'a> Iterator for CacheIterOrVec<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CacheIterOrVec::Iter(i) => i.next(),
            CacheIterOrVec::Vec(i) => i.next(),
        }
    }
}

fn transfer_cache<'a>(
    iter: CacheMoves<FilterLegalMove<'a, BoardIter<'a>>>,
    duration: Duration,
    start: Instant,
) -> CacheMoves<Timed<CacheIterOrVec<'a>>> {
    let (iter, cache, done) = iter.into_raw_parts();
    CacheMoves::from_raw_parts(
        Timed::with_start(CacheIterOrVec::Iter(iter), Some(start), duration).fuse(),
        cache,
        done,
    )
}

fn with_new_cache<'a>(
    iter: CacheMoves<FilterLegalMove<'a, BoardIter<'a>>>,
    cache: Vec<Move>,
    duration: Duration,
    start: Instant,
) -> CacheMoves<Timed<CacheIterOrVec<'a>>> {
    let (iter, _cache, done) = iter.into_raw_parts();
    CacheMoves::from_raw_parts(
        Timed::with_start(CacheIterOrVec::Iter(iter), Some(start), duration).fuse(),
        cache,
        done,
    )
}

fn from_vec<'a>(
    vec: Vec<Move>,
    game: &'a Game,
    partial_game: &'a PartialGame<'a>,
    duration: Duration,
    start: Instant,
) -> CacheMoves<Timed<CacheIterOrVec<'a>>> {
    CacheMoves::new(Timed::with_start(
        CacheIterOrVec::Vec(FilterLegalMove::new(vec.into_iter(), game, partial_game)),
        Some(start),
        duration,
    ))
}
