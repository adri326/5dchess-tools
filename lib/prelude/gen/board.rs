use super::*;

pub struct BoardIter<'a, B: Clone + AsRef<Board> + 'a> {
    pub board: &'a Board,
    pub game: &'a Game,
    pub partial_game: &'a PartialGame<'a, B>,
    pub index: Physical,
    pub current_piece: Option<super::piece::PieceMoveIter<'a, B>>,
}

impl<'a, B: Clone + AsRef<Board> + 'a> Iterator for BoardIter<'a, B> {
    type Item = Move;

    fn next(&mut self) -> Option<Move> {
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
                Some(m) => return Some(m),
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
            .generate_moves(self.game, self.partial_game);
        }

        // TCR ftw :)
        self.next()
    }
}

impl<'a, B: Clone + AsRef<Board> + 'a> GenMoves<'a, B> for &'a Board {
    type Iter = BoardIter<'a, B>;

    /** Generates the moves for a given board.
        Moves are supposed valid and are only made by the pieces that belong to the given board's color.
    **/
    fn generate_moves(
        self,
        game: &'a Game,
        partial_game: &'a PartialGame<'a, B>,
    ) -> Option<BoardIter<'a, B>> {
        Some(BoardIter {
            board: self,
            game,
            partial_game,
            index: 0,
            current_piece: None,
        })
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

pub enum BoardIterOr<'a, B>
where
    B: Clone + AsRef<Board> + 'a,
    for<'b> &'b B: GenMoves<'b, B>,
{
    Board(BoardIter<'a, B>),
    B(<&'a B as GenMoves<'a, B>>::Iter),
}

impl<'a, B> Iterator for BoardIterOr<'a, B>
where
    B: Clone + AsRef<Board> + 'a,
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
    B: Clone + AsRef<Board> + 'a,
    for<'b> &'b B: GenMoves<'b, B>,
{
    type Iter = BoardIterOr<'a, B>;

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
}
