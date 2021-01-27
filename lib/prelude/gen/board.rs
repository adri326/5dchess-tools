use super::*;

/**
    Yields moves for every pieces on a board.
    The type of the items is `Option<Move>`; `BoardIter` is a wrapper around `BoardIterSub`, which
    does a specialized `filter_map`.
**/
#[derive(Clone)]
pub struct BoardIterSub<'a, B: Clone + AsRef<Board>> {
    pub board: &'a Board,
    pub game: &'a Game,
    pub partial_game: &'a PartialGame<'a, B>,
    pub index: Physical,
    pub current_piece: Option<super::piece::PieceMoveIter<'a, B>>,
    pub flag: GenMovesFlag,
}

impl<'a, B: Clone + AsRef<Board>> Iterator for BoardIterSub<'a, B> {
    type Item = Option<Move>;

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
pub struct BoardIter<'a, B: Clone + AsRef<Board>>(pub BoardIterSub<'a, B>);

impl<'a, B: Clone + AsRef<Board>> Iterator for BoardIter<'a, B> {
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

impl<'a, B: Clone + AsRef<Board> + 'a> GenMoves<'a, B> for &'a Board {
    type Iter = BoardIter<'a, B>;

    /** Generates the moves for a given board.
        Moves are supposed valid and are only made by the pieces that belong to the given board's color.
    **/
    fn generate_moves_flag(
        self,
        game: &'a Game,
        partial_game: &'a PartialGame<'a, B>,
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

    fn validate_move(self, game: &Game, partial_game: &PartialGame<B>, mv: &Move) -> bool {
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

#[derive(Clone)]
pub enum BoardIterOr<'a, B>
where
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
{
    Board(<&'a Board as GenMoves<'a, B>>::Iter),
    B(<&'a B as GenMoves<'a, B>>::Iter),
}

impl<'a, B> Iterator for BoardIterOr<'a, B>
where
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
{
    type Item = Move;

    fn next(&mut self) -> Option<Move> {
        match self {
            BoardIterOr::Board(iter) => iter.next(),
            BoardIterOr::B(iter) => iter.next(),
        }
    }
}

impl<'a, B> GenMoves<'a, B> for BoardOr<'a, B>
where
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
{
    type Iter = BoardIterOr<'a, B>;

    /**
        Returns an iterator yielding all of the moves of the current player on a board.
    **/
    #[inline]
    fn generate_moves(
        self,
        game: &'a Game,
        partial_game: &'a PartialGame<'a, B>,
    ) -> Option<Self::Iter> {
        Some(match self {
            BoardOr::Board(board) => BoardIterOr::Board(board.generate_moves(game, partial_game)?),
            BoardOr::B(board) => BoardIterOr::B(board.generate_moves(game, partial_game)?),
        })
    }

    fn validate_move(self, game: &'a Game, partial_game: &'a PartialGame<B>, mv: &Move) -> bool {
        match self {
            BoardOr::Board(board) => board.validate_move(game, partial_game, mv),
            BoardOr::B(board) => board.validate_move(game, partial_game, mv),
        }
    }

    /**
        Returns a specialized iterator over the moves on the board.
        Although `Board::generate_moves_flag` does not do any kind of specialization, this
        will still take advantage from the acceleration offered by `B::generate_moves_flag`.
    **/
    #[inline]
    fn generate_moves_flag(
        self,
        game: &'a Game,
        partial_game: &'a PartialGame<'a, B>,
        flag: GenMovesFlag,
    ) -> Option<Self::Iter> {
        Some(match self {
            BoardOr::Board(board) => {
                BoardIterOr::Board(board.generate_moves_flag(game, partial_game, flag)?)
            }
            BoardOr::B(board) => {
                BoardIterOr::B(board.generate_moves_flag(game, partial_game, flag)?)
            }
        })
    }
}
