use crate::prelude::*;
use super::*;
use std::collections::HashMap;

// TODO: somehow store the boards in the move itself for speed~
// (I don't want to waste another 500ns)
pub struct LegalMove;
pub struct OptLegalMove;

impl<'a, B> Strategy<'a, B> for LegalMove
where
    B: Clone + AsRef<Board>,
    for<'b> B: From<(Board, &'b Game, &'b PartialGame<'b, B>)>,
    for<'b> &'b B: GenMoves<'b, B>,
{
    type From = Move;
    type To = bool;

    fn apply(mv: Move, game: &'a Game, partial_game: &'a PartialGame<'a, B>) -> Option<bool> {
        let mut new_partial_game = PartialGame::new(HashMap::new(), partial_game.info.clone(), None);
        mv.generate_partial_game(game, partial_game, &mut new_partial_game);
        new_partial_game.parent = Some(partial_game);

        is_legal_move(game, &new_partial_game)
    }
}


impl<'a, B> Strategy<'a, B> for OptLegalMove
where
    B: Clone + AsRef<Board>,
    for<'b> B: From<(Board, &'b Game, &'b PartialGame<'b, B>)>,
    for<'b> &'b B: GenMoves<'b, B>,
{
    type From = Move;
    type To = bool;

    fn apply(mv: Move, game: &'a Game, partial_game: &'a PartialGame<'a, B>) -> Option<bool> {
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
    B: Clone + AsRef<Board>,
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
