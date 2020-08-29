// Handles moves
use super::game::*;
use std::fmt;

#[derive(Clone, Copy)]
pub struct Move {
    pub src: (f32, usize, usize, usize), // l, t, x, y
    pub dst: (f32, usize, usize, usize), // l, t, x, y
    pub castle: bool,
    pub castle_long: bool,
    pub src_piece: Piece,
    pub dst_piece: Piece,
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.castle {
            if self.castle_long {
                write!(f, "({}T{})O-O-O", write_timeline(self.src.0), self.src.1 / 2)
            } else {
                write!(f, "({}T{})O-O", write_timeline(self.src.0), self.src.1 / 2)
            }
        } else {
            write!(
                f,
                "({}T{}){}{}{} → ({}T{}){}{}{}",
                write_timeline(self.src.0),
                self.src.1 / 2,
                self.src_piece,
                write_file(self.src.2),
                (self.src.3 + 1),
                write_timeline(self.dst.0),
                self.dst.1 / 2,
                self.dst_piece,
                write_file(self.dst.2),
                (self.dst.3 + 1),
            )
        }
    }
}

impl Move {
    fn new(
        src: (f32, usize, usize, usize),
        dst: (f32, usize, usize, usize),
        game: &Game,
        board: &Board,
        virtual_boards: &Vec<Board>,
    ) -> Option<Self> {
        Some(Move {
            src,
            dst,
            castle: false,
            castle_long: false,
            src_piece: get(game, board, virtual_boards, src)?,
            dst_piece: get(game, board, virtual_boards, dst)?,
        })
    }
}

pub fn probable_moves(game: &Game, board: &Board, virtual_boards: &Vec<Board>) -> Vec<Move> {
    let mut res: Vec<Move> = Vec::new();

    for y in 0..board.height {
        for x in 0..board.width {
            if let Some(piece) = board.get(x, y) {
                if if board.active_player() {
                    piece.is_white()
                } else {
                    piece.is_black()
                } {
                    probable_moves_for(game, board, virtual_boards, &mut res, piece, x, y);
                }
            }
        }
    }

    res
}

fn get(
    game: &Game,
    board: &Board,
    virtual_boards: &Vec<Board>,
    pos: (f32, usize, usize, usize),
) -> Option<Piece> {
    if pos.0 == board.l && pos.1 == board.t {
        return board.get(pos.2, pos.3);
    }
    for vboard in virtual_boards.iter() {
        if pos.0 == vboard.l && pos.1 == vboard.t {
            return vboard.get(pos.2, pos.3);
        }
    }
    game.get(pos.0, pos.1, pos.2, pos.3)
}

fn probable_moves_for(
    game: &Game,
    board: &Board,
    virtual_boards: &Vec<Board>,
    res: &mut Vec<Move>,
    piece: Piece,
    x: usize,
    y: usize,
) -> Option<()> {
    let src = (board.l, board.t, x, y);
    let active_player = board.active_player();
    if piece.is_pawn() {
        let dy: isize = if piece.is_white() { 1 } else { -1 };
        let y1 = ((y as isize) + dy) as usize;
        let y2 = ((y as isize) + 2 * dy) as usize;
        if get(game, board, virtual_boards, (board.l, board.t, x, y1))? == Piece::Blank {
            res.push(Move::new(
                src,
                (board.l, board.t, x, y1),
                game,
                board,
                virtual_boards,
            )?);
            if if piece.is_white() {
                y <= 1
            } else {
                y >= game.height - 2
            } && get(game, board, virtual_boards, (board.l, board.t, x, y2))? == Piece::Blank
            {
                // TODO: handle 1-pawn better
                res.push(Move::new(
                    src,
                    (board.l, board.t, x, y2),
                    game,
                    board,
                    virtual_boards,
                )?);
            }
        }
        // Try to take on x + 1
        if x < game.width - 1
            && (may_en_passant(game, board, virtual_boards, x + 1, y1)
                || get(game, board, virtual_boards, (board.l, board.t, x + 1, y1))?
                    .is_opponent_piece(active_player))
        {
            res.push(Move::new(
                src,
                (board.l, board.t, x + 1, y1),
                game,
                board,
                virtual_boards,
            )?);
        }
        // Try to take on x - 1
        if x > 0
            && (may_en_passant(game, board, virtual_boards, x - 1, y1)
                || get(game, board, virtual_boards, (board.l, board.t, x - 1, y1))?
                    .is_opponent_piece(active_player))
        {
            res.push(Move::new(
                src,
                (board.l, board.t, x - 1, y1),
                game,
                board,
                virtual_boards,
            )?);
        }
    } else if piece.is_king() {
        for dl in -1isize..=1isize {
            for dt in -1isize..=1isize {
                for dy in -1isize..=1isize {
                    for dx in -1isize..=1isize {
                        if dx == 0 && dy == 0 && dl == 0 && dt == 0
                            || x == 0 && dx < 0
                            || x == game.width - 1 && dx > 0
                            || y == 0 && dy < 0
                            || y == game.height && dy > 0
                            || board.t < 2 && dt < 0
                        {
                            continue;
                        }
                        let l1 = if dl == -1 {
                            timeline_below(game, board.l)
                        } else if dl == 1 {
                            timeline_above(game, board.l)
                        } else {
                            board.l
                        };
                        let t1 = ((board.t as isize) + 2 * dt) as usize;
                        let x1 = ((x as isize) + dx) as usize;
                        let y1 = ((y as isize) + dy) as usize;
                        if let Some(true) = get(game, board, virtual_boards, (l1, t1, x1, y1)).map(|p| p.is_takable_piece(active_player)) {
                            res.push(
                                Move::new(
                                    src,
                                    (l1, t1, x1, y1),
                                    game,
                                    board,
                                    virtual_boards,
                                )?
                            );
                        }
                    }
                }
            }
        }
    }
    Some(())
}

fn may_en_passant(
    game: &Game,
    board: &Board,
    virtual_boards: &Vec<Board>,
    x: usize,
    y: usize,
) -> bool {
    if board.t < 2 {
        return false;
    }
    let active_player = board.active_player();
    let dst_y = if active_player { y - 1 } else { y + 1 };
    let src_y = if active_player { y + 1 } else { y - 1 };
    let piece = if active_player {
        Piece::PawnB
    } else {
        Piece::PawnW
    };
    let a = get(game, board, virtual_boards, (board.l, board.t, x, dst_y)).map(|p| p == piece);
    let b =
        get(game, board, virtual_boards, (board.l, board.t, x, src_y)).map(|p| p == Piece::Blank);
    let c = get(
        game,
        board,
        virtual_boards,
        (board.l, board.t - 2, x, dst_y),
    )
    .map(|p| p == Piece::Blank);
    let d = get(
        game,
        board,
        virtual_boards,
        (board.l, board.t - 2, x, src_y),
    )
    .map(|p| p == piece);
    match (a, b, c, d) {
        (Some(true), Some(true), Some(true), Some(true)) => true,
        _ => false,
    }
}
