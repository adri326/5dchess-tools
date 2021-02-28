use crate::prelude::*;
use super::*;

/**
    Yields moves for every pieces on a board.
    The type of the items is `Option<Move>`; `BoardIter` is a wrapper around `BoardIterSub`, which
    does a specialized `filter_map` to turn these into `Move` items.
**/
#[derive(Clone)]
pub struct BoardIterSub<'a> {
    pub board: &'a Board,
    pub game: &'a Game,
    pub partial_game: &'a PartialGame<'a>,
    pub index: Physical,
    pub current_piece: Option<super::piece::PieceMoveIter<'a>>,
    pub flag: GenMovesFlag,
}

impl<'a> Iterator for BoardIterSub<'a> {
    type Item = Option<Move>;

    /// If the current piece iterator is empty, finds the next piece to iterate upon and returns Some(None). Otherwise, yields the move.
    fn next(&mut self) -> Option<Option<Move>> {
        if self.index >= self.board.width() * self.board.height() {
            return None;
        }

        let mut new_iter = false;

        match &mut self.current_piece {
            None => {
                while self.index < self.board.width() * self.board.height()
                    && self.board.pieces[self.index as usize]
                        .map(|p| p.white != self.board.white())
                        .unwrap_or(true)
                {
                    self.index += 1;
                }
                if self.index < self.board.width() * self.board.height() {
                    new_iter = true;
                }
            }
            Some(i) => match i.next() {
                Some(m) => return Some(Some(m)),
                None => {
                    self.index += 1;
                    while self.index < self.board.width() * self.board.height()
                        && self.board.pieces[self.index as usize]
                            .map(|p| p.white != self.board.white())
                            .unwrap_or(true)
                    {
                        self.index += 1;
                    }
                    if self.index < self.board.width() * self.board.height() {
                        new_iter = true;
                    }
                }
            },
        }

        // Swap out the iter for a new one
        if new_iter {
            self.current_piece = PiecePosition::new(
                self.board.pieces[self.index as usize].piece().unwrap(),
                Coords::new(
                    self.board.l(),
                    self.board.t(),
                    self.index % self.board.width(),
                    self.index / self.board.width(),
                ),
            )
            .generate_moves_flag(self.game, self.partial_game, self.flag);

            if self.current_piece.is_none() {
                self.index += 1;
            }
        }

        // No more TCR :(
        Some(None)
    }
}

/**
    Yields all of the moves of the pieces in a board.
    It is a wrapper around `BoardIterSub`.
**/
#[derive(Clone)]
pub struct BoardIter<'a>(pub BoardIterSub<'a>);

impl<'a> Iterator for BoardIter<'a> {
    type Item = Move;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                Some(Some(mv)) => return Some(mv),
                None => return None,
                _ => {}
            }
        }
    }
}

impl<'a> GenMoves<'a> for &'a Board {
    type Iter = BoardIter<'a>;

    /** Generates the moves for a given board.
        Moves are supposed valid and are only made by the pieces that belong to the given board's color.
    **/
    fn generate_moves_flag(
        self,
        game: &'a Game,
        partial_game: &'a PartialGame<'a>,
        flag: GenMovesFlag,
    ) -> Option<Self::Iter> {
        Some(BoardIter(BoardIterSub {
            board: self,
            game,
            partial_game,
            index: 0,
            current_piece: None,
            flag,
        }))
    }

    fn validate_move(self, game: &Game, partial_game: &PartialGame, mv: &Move) -> bool {
        if self.l() != mv.from.1.l() || self.t() != mv.from.1.t() {
            return false;
        } else if self.get(mv.from.1.physical()).is_empty() {
            return false;
        } else {
            return PiecePosition::new(self.get(mv.from.1.physical()).piece().unwrap(), mv.from.1)
                .validate_move(game, partial_game, mv);
        }
    }
}
