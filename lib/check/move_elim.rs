use crate::*;
use crate::gen::*;
use super::threat::*;

pub fn is_legal_move<'a>(
    mv: Move,
    game: &'a Game,
    partial_game: &'a PartialGame<'a>,
) -> Option<bool> {
    let mut new_partial_game = PartialGame::empty(partial_game.info.clone(), None);

    if mv.from.1.non_physical() == mv.to.1.non_physical() {
        mv.generate_partial_game(
            game,
            partial_game,
            &mut new_partial_game,
            PartialGameGenKind::Both,
        );
        new_partial_game.parent = Some(partial_game);
        filter_physical_move(game, &new_partial_game)
    } else {
        mv.generate_partial_game(
            game,
            partial_game,
            &mut new_partial_game,
            PartialGameGenKind::Target,
        );
        match filter_non_physical_move(game, &new_partial_game) {
            Some(true) => {}
            Some(false) => return Some(false),
            None => return None,
        }

        new_partial_game.parent = Some(partial_game);
        new_partial_game.set_boards(PartialGameStorage::Shallow(
            vec![None; new_partial_game.info.timelines_white.len()],
            vec![None; new_partial_game.info.timelines_black.len()]
        ));
        mv.generate_partial_game(
            game,
            partial_game,
            &mut new_partial_game,
            PartialGameGenKind::Source,
        );

        filter_physical_move(game, &new_partial_game)
    }
}

pub fn is_legal_move_optional<'a>(
    mv: Move,
    game: &'a Game,
    partial_game: &'a PartialGame<'a>,
) -> Option<bool> {
    let n_own_boards = partial_game.own_boards(game).count();
    if n_own_boards <= 2 {
        Some(true)
    } else if n_own_boards == 3 {
        let n_opponent_boards = partial_game.opponent_boards(game).count();
        if n_opponent_boards <= 8 {
            is_legal_move(mv, game, partial_game)
        } else {
            Some(true)
        }
    } else {
        is_legal_move(mv, game, partial_game)
    }
}

/// Looks through the opponent's boards
fn filter_physical_move<'a>(game: &'a Game, partial_game: &'a PartialGame<'a>) -> Option<bool> {
    if game.width as usize <= MAX_BITBOARD_WIDTH {
        for board in partial_game.opponent_boards(game) {
            if let Some(_) = is_threatened_bitboard_board(game, partial_game, board) {
                return Some(false)
            }
        }
    } else {
        for board in partial_game.opponent_boards(game) {
            for mv in board.generate_moves_flag(game, partial_game, GenMovesFlag::Check)? {
                if cfg!(castling) {
                    if mv.to.1.non_physical() == board.non_physical() {
                        if let Some((x1, y1, x2, y2)) = board.castle {
                            if (mv.to.1).2 == x1 && (mv.to.1).3 == y1 || (mv.to.1).2 == x2 && (mv.to.1).3 == y2 {
                                return Some(false)
                            }
                        }
                    }
                }

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
    }

    Some(true)
}

// Must be given a partial game with only the target board!
fn filter_non_physical_move<'a>(game: &'a Game, partial_game: &'a PartialGame<'a>) -> Option<bool> {
    if game.width as usize <= MAX_BITBOARD_WIDTH {
        for board in partial_game.iter_shallow() {
            if let Some(_) = threats_within_board(board) {
                return Some(false)
            }
        }
    } else {
        for board in partial_game.iter_shallow() {
            for mv in board.generate_moves_flag(game, partial_game, GenMovesFlag::Check)? {
                match mv.to.0 {
                    Some(piece) => {
                        if
                            piece.is_royal()
                            && piece.white == partial_game.info.active_player
                            && mv.to.1.non_physical() == mv.from.1.non_physical()
                        {
                            return Some(false)
                        }
                    }
                    None => {}
                }
            }
        }
    }

    Some(true)
}

/**
    Iterator that filters moves from a parent iterator that are legal.
**/
#[derive(Clone)]
pub struct FilterLegalMove<'a, I>
where
    I: Iterator<Item = Move>,
{
    pub iter: I,
    pub game: &'a Game,
    pub partial_game: &'a PartialGame<'a>,
}

impl<'a, I> FilterLegalMove<'a, I>
where
    I: Iterator<Item = Move>,
{
    pub fn new(iter: I, game: &'a Game, partial_game: &'a PartialGame<'a>) -> Self {
        Self {
            iter,
            game,
            partial_game,
        }
    }
}

impl<'a, I> Iterator for FilterLegalMove<'a, I>
where
    I: Iterator<Item = Move>,
{
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(mv) => {
                    if is_legal_move_optional(mv, self.game, self.partial_game)? {
                        return Some(mv);
                    }
                }
                None => return None,
            }
        }
    }
}
