use super::*;

#[derive(Clone, Debug)]
pub struct PhaseBoard<B>
where
    B: Clone + AsRef<Board>
{
    pub board: B,
    /// Encoded as one-tile moves for ranging pieces
    pub moves: Vec<Move>,
}

pub static QUEEN_2D_MOVES: [(Physical, Physical); 8] = [(0, 1), (0, -1), (1, 0), (-1, 0), (1, 1), (1, -1), (-1, 1), (-1, -1)];
pub static ROOK_2D_MOVES: [(Physical, Physical); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
pub static BISHOP_2D_MOVES: [(Physical, Physical); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

impl<B> PhaseBoard<B>
where
    B: Clone + AsRef<Board>
{
    pub fn new(board: B) -> Self {
        Self {
            board,
            moves: Vec::new(),
        }
    }
}

impl<'a, B, C> PopulateBoard<'a, C> for PhaseBoard<B>
where
    B: Clone + AsRef<Board>,
    C: Clone + AsRef<Board>,
    for<'b> B: PopulateBoard<'b, C>,
{
    fn populate(&mut self, game: &'a Game, partial_game: &'a PartialGame<'a, C>) -> Option<()> {
        let mut moves: Vec<Move> = Vec::new();
        for (index, piece) in self.board.as_ref().pieces.iter().enumerate() {
            if let Tile::Piece(piece) = piece {
                if piece.white != self.board.as_ref().white() {
                    let x = index as Physical % self.board.as_ref().width();
                    let y = index as Physical / self.board.as_ref().width();
                    match piece.kind {
                        PieceKind::Queen | PieceKind::Princess => add_moves(&mut moves, *piece, self.board.as_ref(), x, y, &QUEEN_2D_MOVES),
                        PieceKind::Rook => add_moves(&mut moves, *piece, self.board.as_ref(), x, y, &ROOK_2D_MOVES),
                        PieceKind::Bishop => add_moves(&mut moves, *piece, self.board.as_ref(), x, y, &BISHOP_2D_MOVES),
                        _ => {},
                    }
                }
            }
        }
        self.moves = moves;

        self.board.populate(game, partial_game)
    }
}

impl<'a, B> GenMoves<'a, PhaseBoard<B>> for &'a PhaseBoard<B>
where
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, PhaseBoard<B>>,
{
    type Iter = PhaseIter<'a, B>;

    #[inline]
    fn generate_moves_flag(
        self,
        game: &'a Game,
        partial_game: &'a PartialGame<'a, PhaseBoard<B>>,
        flag: GenMovesFlag,
    ) -> Option<Self::Iter> {
        match flag {
            GenMovesFlag::Check => Some(
                PhaseIter::Check(self.moves.iter(), self.board.generate_moves_flag(game, partial_game, flag)?, &self.board)
            ),
            GenMovesFlag::Any => Some(PhaseIter::Any(self.board.generate_moves_flag(game, partial_game, flag)?)),
        }
    }
}

pub enum PhaseIter<'a, B>
where
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, PhaseBoard<B>>,
{
    Check(std::slice::Iter<'a, Move>, <&'a B as GenMoves<'a, PhaseBoard<B>>>::Iter, &'a B),
    Any(<&'a B as GenMoves<'a, PhaseBoard<B>>>::Iter),
}

impl<B> AsRef<Board> for PhaseBoard<B>
where
    B: Clone + AsRef<Board>,
{
    fn as_ref(&self) -> &Board {
        self.board.as_ref()
    }
}

impl<'a, B> Iterator for PhaseIter<'a, B>
where
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, PhaseBoard<B>>,
{
    type Item = Move;

    fn next(&mut self) -> Option<Move> {
        match self {
            PhaseIter::Check(phasing, standard, board) => {
                let res = loop {
                    match phasing.next() {
                        Some(mv) => {
                            if let Some(mv2) = is_move_valid(mv, board) {
                                break Some(mv2)
                            }
                        },
                        None => break None,
                    }
                };
                match res {
                    Some(mv) => Some(mv),
                    None => standard.next(),
                }
            },
            PhaseIter::Any(standard) => standard.next(),
        }
    }
}

fn add_moves(moves: &mut Vec<Move>, piece: Piece, board: &Board, x: Physical, y: Physical, dirs: &'static [(Physical, Physical)]) {
    for (dx, dy) in dirs {
        let mut distance = 0;
        let mut phased = false;
        loop {
            distance += 1;
            match board.get((x + dx * distance, y + dy * distance)) {
                Tile::Piece(p) => {
                    if p.white == board.white() {
                        break
                    } else if p.is_royal() {
                        moves.push(Move::from_raw(
                            (piece, Coords(board.l(), board.t() + 1, x, y)),
                            (None, Coords(board.l(), board.t() + 1, x + dx * distance, y + dy * distance)),
                            MoveKind::Normal,
                        ));
                        break
                    } else {
                        if phased {
                            break
                        } else {
                            phased = true;
                        }
                    }
                }
                Tile::Void => break,
                Tile::Blank => {},
            }
        }
    }
}

fn is_move_valid<B: Clone + AsRef<Board>>(mv: &Move, board: &B) -> Option<Move> {
    let board: &Board = board.as_ref();
    let pos = mv.from.1;
    let dpos = mv.to.1 - mv.from.1;
    let mut distance = 0;
    loop {
        distance += 1;
        let npos = pos + dpos * distance;

        match board.get(npos.physical()) {
            Tile::Piece(p) => {
                if p.is_royal() && p.white == mv.from.0.white {
                    break Some(Move::from_raw(mv.from, (Some(p), npos), MoveKind::Normal));
                } else {
                    break None
                }
            },
            Tile::Void => break None,
            Tile::Blank => {},
        }
    }
}

#[macro_export]
macro_rules! genmoves_phase {
    ( $board:ident ) => {
        impl<'a> GenMoves<'a, PhaseBoard<$board>> for &'a $board {
            type Iter = crate::prelude::gen::board::BoardIter<'a, PhaseBoard<$board>>;

            #[inline]
            fn generate_moves_flag(
                self,
                game: &'a Game,
                partial_game: &'a PartialGame<'a, PhaseBoard<$board>>,
                flag: GenMovesFlag,
            ) -> Option<Self::Iter> {
                self.as_ref().generate_moves_flag(game, partial_game, flag)
            }
        }
    }
}
