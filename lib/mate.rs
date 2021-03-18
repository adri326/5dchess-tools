use crate::*;
use crate::gen::*;
use crate::check::*;
use std::collections::{HashSet};
use std::time::{Duration, Instant};
use std::fmt;

/**
    Status for checkmate detection.
**/

#[derive(Clone)]
pub enum Mate<'a> {
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
    None(Moveset, PartialGame<'a>, Option<GenLegalMovesetIter<'a>>),
}

impl<'a> PartialEq for Mate<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Mate::Checkmate, Mate::Checkmate) => true,
            (Mate::Stalemate, Mate::Stalemate) => true,
            (Mate::Error, Mate::Error) => true,
            (Mate::TimeoutCheckmate, Mate::TimeoutCheckmate) => true,
            (Mate::TimeoutStalemate, Mate::TimeoutStalemate) => true,
            (Mate::None(x, _, _), Mate::None(y, _, _)) => x == y,
            _ => false,
        }
    }
}

impl<'a> fmt::Debug for Mate<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mate::Checkmate => {
                write!(f, "Mate::Checkmate")
            }
            Mate::Stalemate => {
                write!(f, "Mate::Stalemate")
            }
            Mate::Error => {
                write!(f, "Mate::Error")
            }
            Mate::TimeoutCheckmate => {
                write!(f, "Mate::TimeoutCheckmate")
            }
            Mate::TimeoutStalemate => {
                write!(f, "Mate::TimeoutStalemate")
            }
            Mate::None(ms, _pos, iter) => {
                write!(f, "Mate::None({:?}, _, {})", ms, if iter.is_some() { "Some(<iter>)" } else { "None" })
            }
        }
    }
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
) -> Mate<'a> {
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
        moves.push(CacheMovesBoards::new(FilterLegalMove::new(
            unwrap_mate!(board.generate_moves(game, partial_game)),
            game,
            partial_game,
        ), game, partial_game));
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
                        Ok(mut ms) => {
                            if let Some(new_partial_game) =
                                ms.generate_partial_game(game, partial_game)
                            {
                                if !unwrap_mate!(is_illegal(game, &new_partial_game)).0 {
                                    ms.necessary_branching = true;
                                    return Mate::None(ms, new_partial_game, None);
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
        let mut reconstructed_moves: Vec<(Vec<Move>, Vec<(Board, Option<Board>)>, _)> = Vec::with_capacity(moves.len());

        let mut possible_move_white: Vec<bool> = vec![false; partial_game.info.timelines_white.len()];
        let mut possible_move_black: Vec<bool> = vec![false; partial_game.info.timelines_black.len()];

        for (&_board, mut own_moves) in own_boards.iter().zip(moves.into_iter()) {
            let mut promising_moves: Vec<Move> = Vec::with_capacity(own_moves.cache.len());
            let mut promising_boards: Vec<(Board, Option<Board>)> = Vec::with_capacity(own_moves.cache.len());

            let mut other_moves: Vec<Move> = Vec::with_capacity(own_moves.cache.len());
            let mut other_boards: Vec<(Board, Option<Board>)> = Vec::with_capacity(own_moves.cache.len());

            let mut unpromising_moves: Vec<Move> = Vec::with_capacity(own_moves.cache.len());
            let mut unpromising_boards: Vec<(Board, Option<Board>)> = Vec::with_capacity(own_moves.cache.len());

            own_moves.consume();
            let (iterator, moves, boards, _, _, _) = own_moves.into_raw_parts();

            // Re-order own_moves
            for (mv, board) in moves.into_iter().zip(boards.into_iter()) {
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
                    unpromising_boards.push(board);
                } else if !mv.from.0.is_royal() && danger.contains(&mv.to.1) {
                    promising_moves.push(mv);
                    promising_boards.push(board);
                } else {
                    other_moves.push(mv);
                    other_boards.push(board);
                }
            }

            promising_moves.append(&mut other_moves);
            promising_moves.append(&mut unpromising_moves);

            promising_boards.append(&mut other_boards);
            promising_boards.append(&mut unpromising_boards);

            reconstructed_moves.push((promising_moves, promising_boards, iterator));
        }

        // Handle shifting impossibility
        // TODO: add condition that there are no legal branching move
        if partial_game.info.timeline_advantage(partial_game.info.active_player) == 0 {
            for l in partial_game.info.min_timeline()..=partial_game.info.max_timeline() {
                if let Some(tl) = partial_game.info.get_timeline(l) {
                    if tl.last_board <= partial_game.info.present && partial_game.info.is_active(l) {
                        if l >= 0 {
                            if !possible_move_white[l as usize] {
                                if attacked_pieces.len() > 0 {
                                    return Mate::Checkmate
                                } else {
                                    return Mate::Stalemate
                                }
                            }
                        } else {
                            if !possible_move_white[(-l) as usize - 1] {
                                if attacked_pieces.len() > 0 {
                                    return Mate::Checkmate
                                } else {
                                    return Mate::Stalemate
                                }
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
                    for (moves, boards, _) in &mut reconstructed_moves {
                        let old_boards = std::mem::replace(boards, Vec::with_capacity(boards.len()));
                        let old_moves = std::mem::replace(moves, Vec::with_capacity(moves.len()));

                        for (mv, board) in old_moves.into_iter().zip(old_boards.into_iter()) {
                            if let Some(tl) = partial_game.info.get_timeline(mv.to.1.l()) {
                                if mv.to.1.t() == tl.last_board {
                                    moves.push(mv);
                                    boards.push(board);
                                }
                            }
                        }
                    }

                    // TODO: recalculate possible_moves_*?
                }

                // Ignore spatial moves from inactive timelines
                for (moves, boards, _) in &mut reconstructed_moves {
                    let old_boards = std::mem::replace(boards, Vec::with_capacity(boards.len()));
                    let old_moves = std::mem::replace(moves, Vec::with_capacity(moves.len()));

                    for (mv, board) in old_moves.into_iter().zip(old_boards.into_iter()) {
                        if partial_game.info.active_player {
                            let min_l = partial_game.info.min_timeline() + debt as Layer - death_limit as Layer;
                            if mv.from.1.non_physical() != mv.to.1.non_physical() || mv.from.1.l() >= min_l {
                                moves.push(mv);
                                boards.push(board);
                            }
                        } else {
                            let max_l = partial_game.info.max_timeline() - debt as Layer + death_limit as Layer;
                            if mv.from.1.non_physical() != mv.to.1.non_physical() || mv.from.1.l() <= max_l {
                                moves.push(mv);
                                boards.push(board);
                            }
                        }
                    }
                }
            }
        }

        reconstructed_moves
            .into_iter()
            .map(|(moves, boards, previous_iter)| CacheMovesBoards::from_raw_parts((
                previous_iter,
                moves,
                boards,
                true,
                game,
                partial_game,
            )))
            .collect()
    } else {
        // Do nothing
        moves
            .into_iter()
            // .map(|moves| transfer_cache(moves, max_duration, start))
            .collect()
    };

    let mut iter = GenLegalMovesetIter::with_iterators(
        game,
        std::borrow::Cow::Borrowed(partial_game),
        moves,
        Some(max_duration.checked_sub(start.elapsed()).unwrap_or(Duration::new(0, 0))),
    );

    // Big boy to look for legal moves
    match iter.next() {
        Some((ms, pos)) => Mate::None(ms, pos, Some(iter)),
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
