// Handles moves
use super::{game::*, moveset::*, resolve::*};
use std::fmt;

// Generate permutations for the basic, symmetric piece movements
lazy_static! {
    pub static ref PERMUTATIONS: Vec<Vec<(isize, isize, isize, isize)>> = {
        [
            (
                vec![
                    (2, 1, 0, 0),
                    (1, 2, 0, 0),
                    (0, 2, 1, 0),
                    (0, 1, 2, 0),
                    (0, 0, 2, 1),
                    (0, 0, 1, 2),
                    (1, 0, 0, 2),
                    (2, 0, 0, 1),
                    (2, 0, 1, 0),
                    (1, 0, 2, 0),
                    (0, 2, 0, 1),
                    (0, 1, 0, 2),
                ],
                2,
            ),
            (
                vec![(1, 0, 0, 0), (0, 1, 0, 0), (0, 0, 1, 0), (0, 0, 0, 1)],
                1,
            ),
            (
                vec![
                    (1, 1, 0, 0),
                    (0, 1, 1, 0),
                    (0, 0, 1, 1),
                    (1, 0, 0, 1),
                    (1, 0, 1, 0),
                    (0, 1, 0, 1),
                ],
                2,
            ),
            (
                vec![(0, 1, 1, 1), (1, 0, 1, 1), (1, 1, 0, 1), (1, 1, 1, 0)],
                3,
            ),
            (vec![(1, 1, 1, 1)], 4),
        ]
        .iter()
        .map(|(group, cardinality)| {
            let mut res: Vec<(isize, isize, isize, isize)> =
                Vec::with_capacity(group.len() * 2usize.pow(*cardinality));
            for element in group {
                for perm_index in 0..(2usize.pow(*cardinality)) {
                    let mut perm: Vec<isize> = Vec::with_capacity(4);
                    let mut o = 0usize;
                    for i in vec![element.0, element.1, element.2, element.3] {
                        if i != 0 {
                            perm.push(if (perm_index >> o) % 2 == 1 { -i } else { i });
                            o += 1;
                        } else {
                            perm.push(0);
                        }
                    }
                    res.push((perm[0], perm[1], perm[2], perm[3]));
                }
            }
            res
        })
        .collect()
    };
}

#[derive(Clone, Copy, PartialEq)]
pub struct Move {
    pub src: (i32, isize, u8, u8), // l, t, x, y
    pub dst: (i32, isize, u8, u8), // l, t, x, y
    pub castle: bool,
    pub castle_long: bool,
    pub en_passant: Option<(u8, u8)>,
    pub src_piece: Piece,
    pub dst_piece: Piece,
    pub noop: bool,
}

impl fmt::Debug for Move {
    /// Prints out a move in semi-readable format; does not match any of the existing and known notations
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.noop {
            return write!(f, "_");
        }
        if self.castle {
            if self.castle_long {
                write!(
                    f,
                    "({}T{})O-O-O",
                    (self.src.0).to_string(),
                    self.src.1 / 2 + 1
                )
            } else {
                write!(
                    f,
                    "({}T{})O-O",
                    (self.src.0).to_string(),
                    self.src.1 / 2 + 1
                )
            }
        } else {
            if self.src.0 == self.dst.0 && self.src.1 == self.dst.1 {
                if self.src_piece.is_pawn() {
                    if self.dst_piece.is_blank() {
                        write!(
                            f,
                            "({}T{}){}{}",
                            (self.src.0).to_string(),
                            self.src.1 / 2 + 1,
                            write_file(self.dst.2),
                            (self.dst.3 + 1),
                        )
                    } else {
                        write!(
                            f,
                            "({}T{}){}x{}{}",
                            (self.src.0).to_string(),
                            self.src.1 / 2 + 1,
                            write_file(self.src.2),
                            write_file(self.dst.2),
                            (self.dst.3 + 1),
                        )
                    }
                } else {
                    if self.dst_piece.is_blank() {
                        write!(
                            f,
                            "({}T{}){}{}{}{}{}",
                            (self.src.0).to_string(),
                            self.src.1 / 2 + 1,
                            self.src_piece.as_uppercase(),
                            write_file(self.src.2),
                            (self.src.3 + 1),
                            write_file(self.dst.2),
                            (self.dst.3 + 1),
                        )
                    } else {
                        write!(
                            f,
                            "({}T{}){}{}{}x{}{}",
                            (self.src.0).to_string(),
                            self.src.1 / 2 + 1,
                            self.src_piece.as_uppercase(),
                            write_file(self.src.2),
                            (self.src.3 + 1),
                            write_file(self.dst.2),
                            (self.dst.3 + 1),
                        )
                    }
                }
            } else {
                if self.dst_piece.is_blank() {
                    write!(
                        f,
                        "({}T{}){}{}{}>>({}T{}){}{}{}",
                        (self.src.0).to_string(),
                        self.src.1 / 2 + 1,
                        self.src_piece.as_uppercase(),
                        write_file(self.src.2),
                        (self.src.3 + 1),
                        (self.dst.0).to_string(),
                        self.dst.1 / 2 + 1,
                        self.dst_piece,
                        write_file(self.dst.2),
                        (self.dst.3 + 1),
                    )
                } else {
                    write!(
                        f,
                        "({}T{}){}{}{}>>x({}T{}){}{}{}",
                        (self.src.0).to_string(),
                        self.src.1 / 2 + 1,
                        self.src_piece.as_uppercase(),
                        write_file(self.src.2),
                        (self.src.3 + 1),
                        (self.dst.0).to_string(),
                        self.dst.1 / 2 + 1,
                        self.dst_piece,
                        write_file(self.dst.2),
                        (self.dst.3 + 1),
                    )
                }
            }
        }
    }
}

impl Move {
    /// Creates a new normal move; extracts piece information from `game` and `virtual_boards`
    pub fn new(
        src: (i32, isize, u8, u8),
        dst: (i32, isize, u8, u8),
        game: &Game,
        virtual_boards: &Vec<&Board>,
    ) -> Option<Self> {
        let src_piece = get(game, virtual_boards, src)?;
        let dst_piece = get(game, virtual_boards, dst)?;
        Some(Move {
            src,
            dst,
            castle: false,
            castle_long: false,
            en_passant: if dst.3 != src.3 || !src_piece.is_pawn() || !dst_piece.is_blank() {
                None
            } else {
                Some((
                    dst.2,
                    if src_piece.is_blank() {
                        dst.3 - 1
                    } else {
                        dst.3 + 1
                    },
                ))
            },
            src_piece,
            dst_piece,
            noop: false,
        })
    }

    /// Creates a new normal move; extracts piece information from `game`, `board` `virtual_boards`. Does not need `board` to be within `game` or `virtual_boards`
    fn new_with_board(
        src: (i32, isize, u8, u8),
        dst: (i32, isize, u8, u8),
        game: &Game,
        board: &Board,
        virtual_boards: &Vec<&Board>,
    ) -> Option<Self> {
        let src_piece = get_with_board(game, board, virtual_boards, src)?;
        let dst_piece = get_with_board(game, board, virtual_boards, dst)?;
        Some(Move {
            src,
            dst,
            castle: false,
            castle_long: false,
            en_passant: if dst.3 != src.3 || !src_piece.is_pawn() || !dst_piece.is_blank() {
                None
            } else {
                Some((
                    dst.2,
                    if src_piece.is_blank() {
                        dst.3 - 1
                    } else {
                        dst.3 + 1
                    },
                ))
            },
            src_piece,
            dst_piece,
            noop: false,
        })
    }

    /// Creates a new castling move
    pub fn castle(
        long: bool,
        src: (i32, isize, u8, u8),
        dst: (u8, u8),
        white: bool,
    ) -> Option<Self> {
        let src_piece = if white { Piece::KingW } else { Piece::KingB };
        Some(Move {
            src,
            dst: (src.0, src.1, dst.0, dst.1),
            castle: true,
            castle_long: long,
            en_passant: None,
            src_piece,
            dst_piece: if white { Piece::RookW } else { Piece::RookB },
            noop: false,
        })
    }

    /// Creates an empty move (which moves no piece)
    pub fn noop(src: (i32, isize)) -> Self {
        Move {
            src: (src.0, src.1, 0, 0),
            dst: (src.0, src.1, 0, 0),
            castle: false,
            castle_long: false,
            en_passant: None,
            src_piece: Piece::Blank,
            dst_piece: Piece::Blank,
            noop: true,
        }
    }

    /// Generate the boards that are created as a result of the move being played out. The target and source boards must be present in either `game`, `virtual_boards` or `already_generated`
    pub fn generate_vboards(
        &self,
        game: &Game,
        info: &GameInfo,
        virtual_boards: &Vec<&Board>,
        already_generated: &Vec<Board>,
    ) -> Option<(GameInfo, Vec<Board>)> {
        // TODO: properly handle Board::set's result
        if self.noop {
            return Some((info.clone(), vec![]));
        }

        let mut new_board = get_board(game, virtual_boards, (self.src.0, self.src.1))?.clone();

        if !is_last(game, virtual_boards, &new_board)
            || already_generated
                .iter()
                .find(|b| b.l == new_board.l && b.t == new_board.t + 1)
                .is_some()
        {
            return None;
        }

        if self.castle {
            new_board.t += 1;
            new_board.set(self.src.2, self.src.3, Piece::Blank).unwrap();
            new_board.set(self.dst.2, self.dst.3, Piece::Blank).unwrap();

            new_board.set(
                self.src.2,
                if self.castle_long { 2 } else { game.width - 2 },
                if new_board.active_player() {
                    Piece::KingB
                } else {
                    Piece::KingW
                },
            ).unwrap();
            new_board.set(
                self.dst.2,
                if self.castle_long { 3 } else { game.width - 3 },
                if new_board.active_player() {
                    Piece::RookB
                } else {
                    Piece::RookW
                },
            ).unwrap();
            Some((info.clone(), vec![new_board]))
        } else if self.en_passant.is_some() {
            new_board.t += 1;
            new_board.set(self.src.2, self.src.3, Piece::Blank).unwrap();
            new_board.set(self.en_passant?.0, self.en_passant?.1, Piece::Blank).unwrap();
            new_board.set(self.dst.2, self.dst.3, self.src_piece).unwrap();
            Some((info.clone(), vec![new_board]))
        } else {
            if self.src.0 == self.dst.0 && self.src.1 == self.dst.1 {
                // Non-branching move
                new_board.t += 1;
                new_board.set(self.src.2, self.src.3, Piece::Blank).unwrap();
                new_board.set(self.dst.2, self.dst.3, self.src_piece).unwrap();

                let info = info.clone();

                if self.src_piece.is_pawn()
                    && self.dst.3
                        == if self.src_piece.is_white() {
                            new_board.height - 1
                        } else {
                            0
                        }
                {
                    new_board.set(
                        self.dst.2,
                        self.dst.3,
                        if self.src_piece.is_white() {
                            Piece::QueenW
                        } else {
                            Piece::QueenB
                        },
                    ).unwrap();
                }

                // Impossible!
                // for b in already_generated {
                //     if b.t == new_board.t && b.l == new_board.l {
                //         // Uhm actually, it's a branching move
                //         new_board.l = if new_board.active_player() {
                //             info.max_timeline = timeline_above(game, info.max_timeline);
                //             info.max_timeline
                //         } else {
                //             info.min_timeline = timeline_below(game, info.min_timeline);
                //             info.min_timeline
                //         };
                //         break;
                //     }
                // }

                Some((info, vec![new_board]))
            } else {
                let mut new_src_board = new_board;
                let mut new_dst_board =
                    get_board(game, virtual_boards, (self.dst.0, self.dst.1))?.clone();

                let mut new_info = info.clone();
                if !is_last(game, virtual_boards, &new_dst_board)
                    || already_generated
                        .iter()
                        .find(|b| b.l == new_dst_board.l && b.t == new_dst_board.t + 1)
                        .is_some()
                {
                    new_dst_board.l = if new_src_board.active_player() {
                        new_info.max_timeline = info.max_timeline + 1;
                        new_info.max_timeline
                    } else {
                        new_info.min_timeline = info.min_timeline - 1;
                        new_info.min_timeline
                    };
                }

                new_src_board.t += 1;
                new_dst_board.t += 1;
                // TODO: timeline reactivation
                if new_dst_board.t < new_info.present && new_dst_board.is_active(&new_info) {
                    new_info.present = new_dst_board.t;
                }

                if if self.src_piece.is_white() {
                    -info.min_timeline > info.max_timeline
                } else {
                    -info.min_timeline < info.max_timeline
                } {
                    new_info.present = find_present(game, virtual_boards, info);
                }

                new_src_board.set(self.src.2, self.src.3, Piece::Blank).unwrap();
                new_dst_board.set(self.dst.2, self.dst.3, self.src_piece).unwrap();

                Some((new_info, vec![new_src_board, new_dst_board]))
            }
        }
    }
}

/// Returns the set of moves that can be made from `board`; does not check for the legality of said move (ie. if it puts the player in check)
pub fn probable_moves(game: &Game, board: &Board, virtual_boards: &Vec<&Board>) -> Vec<Move> {
    let mut res: Vec<Move> = Vec::new();

    for y in 0..board.height {
        for x in 0..board.width {
            if let Some(piece) = board.get(x, y) {
                if if board.active_player() {
                    piece.is_white()
                } else {
                    piece.is_black()
                } {
                    probable_moves_for(game, board, virtual_boards, &mut res, piece, x, y).unwrap();
                }
            }
        }
    }

    if board.active_player() && board.width > 5 {
        if board.castle_w.0 {
            // TODO: check the b and c file
            let king_w = board.king_w.unwrap();
            let (mut x, y) = king_w;
            if x != 0 {
                x -= 1;
                while let Some(piece) = board.get(x, y) {
                    if let Piece::RookW = piece {
                        res.push(
                            Move::castle(true, (board.l, board.t, king_w.0, king_w.1), (x, y), true)
                                .unwrap(),
                        );
                        break;
                    } else if let Piece::Blank = piece {
                        if x == 0 {
                            break;
                        }
                        x -= 1;
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
        if board.castle_w.1 {
            // TODO: check the f and g file
            let king_w = board.king_w.unwrap();
            let (mut x, y) = king_w;
            x += 1;
            while let Some(piece) = board.get(x, y) {
                if let Piece::RookW = piece {
                    res.push(
                        Move::castle(false, (board.l, board.t, king_w.0, king_w.1), (x, y), true)
                            .unwrap(),
                    );
                    break;
                } else if let Piece::Blank = piece {
                    x += 1;
                    continue;
                } else {
                    break;
                }
            }
        }
    }
    if !board.active_player() && board.width > 5 {
        if board.castle_b.0 {
            // TODO: check the b and c file
            let king_b = board.king_b.unwrap();
            let (mut x, y) = king_b;
            if x != 0 {
                x -= 1;
                while let Some(piece) = board.get(x, y) {
                    if let Piece::RookB = piece {
                        res.push(
                            Move::castle(true, (board.l, board.t, king_b.0, king_b.1), (x, y), false)
                                .unwrap(),
                        );
                        break;
                    } else if let Piece::Blank = piece {
                        if x == 0 {
                            break;
                        }
                        x -= 1;
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
        if board.castle_b.1 {
            // TODO: check the f and g file
            let king_b = board.king_b.unwrap();
            let (mut x, y) = king_b;
            x += 1;
            while let Some(piece) = board.get(x, y) {
                if let Piece::RookB = piece {
                    res.push(
                        Move::castle(false, (board.l, board.t, king_b.0, king_b.1), (x, y), false)
                            .unwrap(),
                    );
                    break;
                } else if let Piece::Blank = piece {
                    x += 1;
                    continue;
                } else {
                    break;
                }
            }
        }
    }

    res
}

/// Returns whether or not a moveset is legal (ie. if it doesn't put the player in check).
pub fn is_moveset_legal<'a, U>(
    game: &Game,
    virtual_boards: &Vec<&Board>,
    info: &GameInfo,
    boards: U,
) -> bool
where
    U: Iterator<Item = &'a Board>,
{
    let opponent = !info.active_player;

    for board in boards {
        if is_last(game, virtual_boards, board) {
            if board.active_player() == opponent {
                for m in probable_moves(game, board, virtual_boards) {
                    if m.dst_piece == (if opponent { Piece::KingB } else { Piece::KingW }) {
                        return false;
                    }
                }
            } else {
                if board.is_active(info) {
                    return false;
                }
            }
        }
    }

    true
}

/// Returns whether or not every mandatory boards were played on (accepts time travel escapes)
pub fn all_boards_played(game: &Game, virtual_boards: &Vec<&Board>, info: &GameInfo) -> bool {
    for board in get_own_boards(game, virtual_boards, info) {
        if board.t <= info.present {
            return false;
        }
    }
    true
}

/// Returns the set of boards on which the opponent can make a move
pub fn get_opponent_boards<'a>(
    game: &'a Game,
    virtual_boards: &'a Vec<&'a Board>,
    info: &'a GameInfo,
) -> Vec<&'a Board> {
    let mut res: Vec<&Board> = game
        .timelines
        .values()
        .map(|tl| &tl.states[tl.states.len() - 1])
        .filter(|b| b.active_player() != info.active_player && is_last(game, virtual_boards, b))
        .collect();
    for b in virtual_boards {
        if b.active_player() != info.active_player && is_last(game, virtual_boards, b) {
            res.push(b);
        }
    }
    res
}

/// Returns the set of board on which the active player can make a move
pub fn get_own_boards<'a>(
    game: &'a Game,
    virtual_boards: &'a Vec<&'a Board>,
    info: &'a GameInfo,
) -> Vec<&'a Board> {
    let mut res: Vec<&Board> = game
        .timelines
        .values()
        .map(|tl| &tl.states[tl.states.len() - 1])
        .filter(|b| b.active_player() == info.active_player && is_last(game, virtual_boards, b))
        .collect();
    for b in virtual_boards {
        if b.active_player() == info.active_player && is_last(game, virtual_boards, b) {
            res.push(b);
        }
    }
    res
}

/// Returns a lazy iterator over the legal movesets that the active player can make
pub fn legal_movesets<'a>(
    game: &'a Game,
    info: &'a GameInfo,
    virtual_boards: &'a Vec<&'a Board>,
    max_moves_considered: usize,
    max_movesets_considered: usize,
) -> impl Iterator<Item = (Vec<Move>, Vec<Board>, GameInfo, f32)> + 'a {
    let ranked_moves = get_own_boards(&game, &virtual_boards, &info)
        .into_iter()
        .map(|board| {
            let lore = Lore::new(
                game,
                virtual_boards,
                board,
                get_opponent_boards(&game, &virtual_boards, &info).into_iter(),
                &info,
            );
            let probables = probable_moves(&game, board, &virtual_boards)
                .into_iter()
                .map(|mv| {
                    let (new_info, new_vboards) = mv
                        .generate_vboards(&game, &info, &virtual_boards, &vec![])
                        .unwrap();
                    (mv, new_info, new_vboards)
                })
                .collect::<Vec<_>>();
            score_moves(&game, &virtual_boards, board, &lore, probables, &info)
        })
        .collect::<Vec<_>>();

    let mut iter = MovesetIter::new(&game, &virtual_boards, &info, ranked_moves);

    iter.max_moves_considered = max_moves_considered;
    iter.max_movesets_considered = max_movesets_considered;

    iter.score()
}

/// Returns the `(l, t)` board within `game` or `virtual_boards`
pub fn get_board<'a, 'b, 'd>(
    game: &'a Game,
    virtual_boards: &'b Vec<&'b Board>,
    pos: (i32, isize),
) -> Option<&'d Board>
where
    'a: 'd,
    'b: 'd,
{
    for vboard in virtual_boards.iter() {
        if pos.0 == vboard.l && pos.1 == vboard.t {
            return Some(vboard);
        }
    }
    game.get_board(pos.0, pos.1)
}

/// Returns the `(l, t, x, y)` square within `game` or `virtual_boards`
fn get(
    game: &Game,
    virtual_boards: &Vec<&Board>,
    pos: (i32, isize, u8, u8),
) -> Option<Piece> {
    get_board(game, virtual_boards, (pos.0, pos.1))
        .map(|b| b.get(pos.2, pos.3))
        .flatten()
}

/// Returns the `(l, t, x, y)` square within either `game`, `virtual_boards` or `board`
fn get_with_board(
    game: &Game,
    board: &Board,
    virtual_boards: &Vec<&Board>,
    pos: (i32, isize, u8, u8),
) -> Option<Piece> {
    if pos.0 == board.l && pos.1 == board.t {
        board.get(pos.2, pos.3)
    } else {
        get_board(game, virtual_boards, (pos.0, pos.1))
            .map(|b| b.get(pos.2, pos.3))
            .flatten()
    }
}

/// Returns whether or not `board` is the last board of its timeline (looks in `game` and `virtual_boards`)
pub fn is_last(game: &Game, virtual_boards: &Vec<&Board>, board: &Board) -> bool {
    if let Some(tl) = game.get_timeline(board.l) {
        if (tl.states.len() as isize) + tl.begins_at - 1 > board.t {
            return false;
        }
    }
    for vboard in virtual_boards.iter() {
        if vboard.l == board.l && vboard.t > board.t {
            return false;
        }
    }
    true
}

/// Returns the set of moves that `piece` can make (does not check the legality of that move)
pub fn probable_moves_for(
    game: &Game,
    board: &Board,
    virtual_boards: &Vec<&Board>,
    res: &mut Vec<Move>,
    piece: Piece,
    x: u8,
    y: u8,
) -> Option<()> {
    let src = (board.l, board.t, x, y);
    let active_player = board.active_player();
    if piece.is_pawn() {
        let dy: isize = if piece.is_white() { 1 } else { -1 };
        let y1 = ((y as isize) + dy) as u8;
        let y2 = ((y as isize) + 2 * dy) as u8;
        if board.get(x, y1)? == Piece::Blank {
            res.push(Move::new_with_board(
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
            } && board.get(x, y2)? == Piece::Blank
            {
                // TODO: handle 1-pawn better
                res.push(Move::new_with_board(
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
                || board.get(x + 1, y1)?
                    .is_opponent_piece(active_player))
        {
            res.push(Move::new_with_board(
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
                || board.get(x - 1, y1)?
                    .is_opponent_piece(active_player))
        {
            res.push(Move::new_with_board(
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
                            || y == game.height - 1 && dy > 0
                            || board.t < 2 && dt < 0
                        {
                            continue;
                        }
                        let l1 = if dl == -1 {
                            board.l - 1
                        } else if dl == 1 {
                            board.l + 1
                        } else {
                            board.l
                        };
                        let t1 = board.t + 2 * dt;
                        let x1 = ((x as isize) + dx) as u8;
                        let y1 = ((y as isize) + dy) as u8;
                        if let Some(true) = get_with_board(game, board, virtual_boards, (l1, t1, x1, y1))
                            .map(|p| p.is_takable_piece(active_player))
                        {
                            res.push(Move::new_with_board(
                                src,
                                (l1, t1, x1, y1),
                                game,
                                board,
                                virtual_boards,
                            )?);
                        }
                    }
                }
            }
        }
    } else if piece.is_knight() {
        n_gonal(
            game,
            board,
            virtual_boards,
            res,
            (board.l, board.t, x, y),
            0,
            active_player,
        )?;
    } else if piece.is_rook() {
        n_gonal(
            game,
            board,
            virtual_boards,
            res,
            (board.l, board.t, x, y),
            1,
            active_player,
        )?;
    } else if piece.is_bishop() {
        n_gonal(
            game,
            board,
            virtual_boards,
            res,
            (board.l, board.t, x, y),
            2,
            active_player,
        )?;
    } else if piece.is_unicorn() {
        n_gonal(
            game,
            board,
            virtual_boards,
            res,
            (board.l, board.t, x, y),
            3,
            active_player,
        )?;
    } else if piece.is_dragon() {
        n_gonal(
            game,
            board,
            virtual_boards,
            res,
            (board.l, board.t, x, y),
            4,
            active_player,
        )?;
    } else if piece.is_queen() {
        n_gonal(
            game,
            board,
            virtual_boards,
            res,
            (board.l, board.t, x, y),
            1,
            active_player,
        )?;
        n_gonal(
            game,
            board,
            virtual_boards,
            res,
            (board.l, board.t, x, y),
            2,
            active_player,
        )?;
        n_gonal(
            game,
            board,
            virtual_boards,
            res,
            (board.l, board.t, x, y),
            3,
            active_player,
        )?;
        n_gonal(
            game,
            board,
            virtual_boards,
            res,
            (board.l, board.t, x, y),
            4,
            active_player,
        )?;
    } else if piece.is_princess() {
        n_gonal(
            game,
            board,
            virtual_boards,
            res,
            (board.l, board.t, x, y),
            1,
            active_player,
        )?;
        n_gonal(
            game,
            board,
            virtual_boards,
            res,
            (board.l, board.t, x, y),
            2,
            active_player,
        )?;
    }
    Some(())
}

/// Returns if the `x, y` square in `board` can be taken as en-passant (`[ɑ̃ pasɑ̃]`)
fn may_en_passant(
    game: &Game,
    board: &Board,
    virtual_boards: &Vec<&Board>,
    x: u8,
    y: u8,
) -> bool {
    if board.t < 2 || y == 0 || y == game.height - 1 {
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
    let a = get(game, virtual_boards, (board.l, board.t, x, dst_y)).map(|p| p == piece);
    let b = get(game, virtual_boards, (board.l, board.t, x, src_y)).map(|p| p == Piece::Blank);
    let c = get(game, virtual_boards, (board.l, board.t - 2, x, dst_y)).map(|p| p == Piece::Blank);
    let d = get(game, virtual_boards, (board.l, board.t - 2, x, src_y)).map(|p| p == piece);
    match (a, b, c, d) {
        (Some(true), Some(true), Some(true), Some(true)) => true,
        _ => false,
    }
}

/// Generate the moves for n-gonals of pieces (knight's is the `0`-th n-gonal)
fn n_gonal(
    game: &Game,
    board: &Board,
    virtual_boards: &Vec<&Board>,
    res: &mut Vec<Move>,
    src: (i32, isize, u8, u8),
    n: usize,
    active_player: bool,
) -> Option<()> {
    for permutation in &PERMUTATIONS[n] {
        let mut length: isize = 1;
        loop {
            let l0 = (src.0 as isize + permutation.0 * length) as i32;
            let t0 = src.1 as isize + permutation.1 * length * 2;
            let x0 = src.2 as isize + permutation.2 * length;
            let y0 = src.3 as isize + permutation.3 * length;
            if x0 < 0 || x0 >= game.width as isize || y0 < 0 || y0 >= game.height as isize
            {
                break;
            }
            let dst = (l0, t0, x0 as u8, y0 as u8);
            let piece = get_with_board(game, board, virtual_boards, dst);

            if let Some(true) = piece.map(|piece| piece.is_takable_piece(active_player)) {
                res.push(Move::new_with_board(src, dst, game, board, virtual_boards)?);
                if piece.unwrap().is_opponent_piece(active_player) {
                    break;
                }
            } else {
                break;
            }
            if n == 0 {
                break;
            }
            length += 1;
        }
    }
    Some(())
}

/// Re-calculate the present
pub fn find_present(game: &Game, virtual_boards: &Vec<&Board>, info: &GameInfo) -> isize {
    let mut min = info.present;
    game.timelines
        .values()
        .map(|tl| &tl.states[tl.states.len() - 1])
        .filter(|b| is_last(game, virtual_boards, b) && b.is_active(info))
        .for_each(|b| {
            if b.t < min {
                min = b.t;
            }
        });
    for b in virtual_boards {
        if is_last(game, virtual_boards, b) && b.t < min && b.is_active(info) {
            min = b.t;
        }
    }

    min
}

// TODO: fix this
/// Returns whether or not making the move `mv` is optional (currently unused, might change it to if the move can be safely omitted)
pub fn is_optional(info: &GameInfo, mv: &Move) -> bool {
    if mv.src.1 > info.present
        || mv.src.0 < -info.max_timeline - 1
        || mv.src.0 > -info.min_timeline + 1
    {
        mv.src.0 == mv.dst.0 && mv.src.1 == mv.dst.1
    } else {
        false
    }
}

/// Returns whether or not the game is a draw; assumes that no move can be made
pub fn is_draw(game: &Game, virtual_boards: &Vec<&Board>, info: &GameInfo) -> bool {
    let opponent_boards = get_opponent_boards(game, virtual_boards, info).into_iter().filter(|b| b.is_active(info)).collect::<Vec<_>>();
    let own_boards = get_own_boards(game, virtual_boards, info)
        .into_iter()
        .cloned()
        .filter(|b| b.is_active(info))
        .map(|mut x| {
            x.t += 1;
            x
        })
        .collect::<Vec<_>>();

    let merged_vboards = virtual_boards
        .iter()
        .map(|x| *x)
        .chain(opponent_boards.iter().map(|x| *x))
        .chain(own_boards.iter())
        .collect::<Vec<_>>();

    // TODO: merge mutated own_boards with virtual_boards

    for b in opponent_boards.into_iter() {
        for mv in probable_moves(game, b, &merged_vboards) {
            if mv.dst_piece.is_king() {
                return false;
            }
        }
    }

    for b in own_boards.iter() {
        for mv in probable_moves(game, &b, &merged_vboards) {
            if mv.dst_piece.is_king() {
                return false;
            }
        }
    }

    true
}
