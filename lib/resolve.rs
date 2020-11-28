// Functions around scoring states and moves

use crate::{game::*, moves::*};

pub const JUMP_COST: i32 = -4;
pub const JUMP_INACTIVE_COST: i32 = -24;
pub const TAKE_ENEMY_REWARD: i32 = 20;
pub const KING_DANGER_COST: i32 = -10;

pub const ROOK_DANGER_COST: i32 = -3;
pub const KNIGHT_DANGER_COST: i32 = -4;
pub const BISHOP_DANGER_COST: i32 = -5;
pub const QUEEN_DANGER_COST: i32 = -10;
pub const UNICORN_DANGER_COST: i32 = -2;
pub const DRAGON_DANGER_COST: i32 = -2;

pub const PROTECT_KING_REWARD: i32 = 3;

pub const TAKE_ROOK_REWARD: i32 = 3;
pub const TAKE_KNIGHT_REWARD: i32 = 4;
pub const TAKE_BISHOP_REWARD: i32 = 5;
pub const TAKE_PRINCESS_REWARD: i32 = 8;
pub const TAKE_QUEEN_REWARD: i32 = 10;
pub const TAKE_UNICORN_REWARD: i32 = 2;
pub const TAKE_DRAGON_REWARD: i32 = 2;

pub const CHECK_QUEEN_REWARD: i32 = 8;
pub const CHECK_PRINCESS_REWARD: i32 = 6;
pub const CHECK_KNIGHT_REWARD: i32 = 5;
pub const CHECK_BISHOP_REWARD: i32 = 5;
pub const CHECK_ROOK_REWARD: i32 = 3;
pub const CHECK_UNICORN_REWARD: i32 = 4;
pub const CHECK_DRAGON_REWARD: i32 = 4;

pub const ATTACK_QUEEN_REWARD: i32 = 2;
pub const ATTACK_PRINCESS_REWARD: i32 = 2;
pub const ATTACK_BISHOP_REWARD: i32 = 1;
pub const ATTACK_KNIGHT_REWARD: i32 = 1;
pub const ATTACK_ROOK_REWARD: i32 = 1;

pub const MANY_KINGS_COST: i32 = -6;

/**
    Structure containing information about hotspots on a board, enemies attacking the current king, danger zones, etc.
**/
#[derive(Debug)]
pub struct Lore<'a> {
    pub board: &'a Board,
    pub danger: Vec<usize>,
    pub enemies: Vec<(f32, isize, usize, usize)>,
}

impl<'a> Lore<'a> {
    /**
        Generates a board's "Lore" (danger map and target pieces)
    **/
    pub fn new<'b, T: Iterator<Item = &'b Board>>(
        game: &Game,
        virtual_boards: &Vec<&Board>,
        board: &'a Board,
        opponent_boards: T,
        _info: &GameInfo,
    ) -> Lore<'a> {
        let mut res = Lore {
            board,
            danger: vec![0; board.pieces.len()],
            enemies: Vec::new(),
        };

        let mut noop_board = board.clone();
        noop_board.t += 1;

        let mut n_virtual_boards = virtual_boards.clone();
        n_virtual_boards.push(&noop_board);

        for b in opponent_boards {
            let probables = probable_moves(game, b, &n_virtual_boards);
            for mv in probables {
                if mv.dst_piece.is_king() {
                    res.register_enemy(&mv);
                }
                res.register_danger(&mv);
            }
        }

        let probables = probable_moves(game, &noop_board, &n_virtual_boards);
        for mv in probables {
            if mv.dst_piece.is_king() {
                res.register_enemy(&mv);
            }
            res.register_danger(&mv);
        }

        res
    }

    #[inline]
    fn register_enemy(&mut self, mv: &Move) {
        if !self.enemies.iter().find(|e| **e == mv.src).is_some() {
            self.enemies.push(mv.src);
        }
    }

    #[inline]
    fn register_danger(&mut self, mv: &Move) {
        if mv.dst.0 == self.board.l && (mv.dst.1 == self.board.t + 1 || mv.dst.1 == self.board.t) {
            self.danger[mv.dst.2 + mv.dst.3 * self.board.width] += 1;
        }
    }
}

/**
    Gives each move in a set of moves (all of which happen on one board) a score and sorts them.
**/
#[allow(unused_variables)]
pub fn score_moves<'a>(
    game: &Game,
    virtual_boards: &Vec<&Board>,
    board: &'a Board,
    lore: &Lore<'a>,
    moves: Vec<(Move, GameInfo, Vec<Board>)>,
    info: &GameInfo,
) -> Vec<(Move, Vec<Board>, GameInfo, i32)> {
    coz::scope!("score_moves");
    let mut res = moves
        .into_iter()
        .map(|(mv, info, boards)| {
            let mut score: i32 = 0;

            if (mv.src.0 != mv.dst.0 || mv.src.1 != mv.dst.1)
                && !is_last(
                    game,
                    virtual_boards,
                    get_board(game, virtual_boards, (mv.dst.0, mv.dst.1)).unwrap(),
                )
            {
                if if info.active_player {
                    info.max_timeline >= -info.min_timeline + 1.0
                } else {
                    info.max_timeline <= -info.min_timeline - 1.0
                } {
                    score += JUMP_INACTIVE_COST;
                } else {
                    score += JUMP_COST;
                }
            }

            if lore
                .enemies
                .iter()
                .find(|e| {
                    e.0 == mv.dst.0 && e.1 == mv.dst.1 + 1 && e.2 == mv.dst.2 && e.3 == mv.dst.3
                })
                .is_some()
            {
                score += TAKE_ENEMY_REWARD;
            }

            if mv.dst_piece.is_knight() {
                score += TAKE_KNIGHT_REWARD;
            } else if mv.dst_piece.is_rook() {
                score += TAKE_ROOK_REWARD;
            } else if mv.dst_piece.is_bishop() {
                score += TAKE_BISHOP_REWARD;
            } else if mv.dst_piece.is_queen() {
                score += TAKE_QUEEN_REWARD;
            } else if mv.dst_piece.is_unicorn() {
                score += TAKE_UNICORN_REWARD;
            } else if mv.dst_piece.is_dragon() {
                score += TAKE_DRAGON_REWARD;
            } else if mv.dst_piece.is_princess() {
                score += TAKE_PRINCESS_REWARD;
            }

            let mut moves: Vec<Move> = Vec::new();

            probable_moves_for(
                game,
                get_board(game, virtual_boards, (mv.dst.0, mv.dst.1)).unwrap(),
                virtual_boards,
                &mut moves,
                mv.src_piece,
                mv.dst.2,
                mv.dst.3,
            );

            for mv in moves {
                if mv.dst_piece.is_king() {
                    if mv.src_piece.is_knight() {
                        score += CHECK_KNIGHT_REWARD;
                    } else if mv.src_piece.is_rook() {
                        score += CHECK_ROOK_REWARD;
                    } else if mv.src_piece.is_bishop() {
                        score += CHECK_BISHOP_REWARD;
                    } else if mv.src_piece.is_queen() {
                        score += CHECK_QUEEN_REWARD;
                    } else if mv.src_piece.is_unicorn() {
                        score += CHECK_UNICORN_REWARD;
                    } else if mv.src_piece.is_dragon() {
                        score += CHECK_DRAGON_REWARD;
                    } else if mv.src_piece.is_princess() {
                        score += CHECK_PRINCESS_REWARD;
                    }
                } else if mv.dst.0 == mv.src.0 && mv.dst.1 == mv.src.1 {
                    if mv.dst_piece.is_queen() {
                        score += ATTACK_PRINCESS_REWARD;
                    } else if mv.dst_piece.is_queen() {
                        score += ATTACK_QUEEN_REWARD;
                    } else if mv.dst_piece.is_bishop() {
                        score += ATTACK_BISHOP_REWARD;
                    } else if mv.dst_piece.is_knight() {
                        score += ATTACK_BISHOP_REWARD;
                    }
                }
            }

            for b in &boards {
                let mut n_kings: usize = 0;
                for (index, piece) in b.pieces.iter().enumerate() {
                    if *piece != Piece::Blank && piece.is_white() == board.active_player() {
                        if piece.is_king() {
                            n_kings += 1;
                            score += (lore.danger[index] as i32) * KING_DANGER_COST;
                            if n_kings > 1 {
                                score += MANY_KINGS_COST;
                            }
                        }
                        if piece.is_rook() {
                            score += (lore.danger[index] as i32) * ROOK_DANGER_COST;
                        }
                        if piece.is_knight() {
                            score += (lore.danger[index] as i32) * KNIGHT_DANGER_COST;
                        }
                        if piece.is_bishop() {
                            score += (lore.danger[index] as i32) * BISHOP_DANGER_COST;
                        }
                        if piece.is_queen() {
                            score += (lore.danger[index] as i32) * QUEEN_DANGER_COST;
                        }
                        if piece.is_unicorn() {
                            score += (lore.danger[index] as i32) * UNICORN_DANGER_COST;
                        }
                        if piece.is_dragon() {
                            score += (lore.danger[index] as i32) * DRAGON_DANGER_COST;
                        }
                    }
                }
            }

            (mv, boards, info, score)
        })
        .filter(|(_mv, boards, info, _score)| {
            is_moveset_legal(game, virtual_boards, info, boards.iter())
        })
        .collect::<Vec<_>>();
    res.sort_unstable_by_key(|(_mv, _boards, _info, score)| -(*score as i32));

    res
}

// Piece values: (how much they are worth)
pub const ROOK_VALUE: f32 = 3.0;
pub const KNIGHT_VALUE: f32 = 4.5;
pub const PRINCESS_VALUE: f32 = 8.0;
pub const QUEEN_VALUE: f32 = 14.0;
pub const KING_VALUE: f32 = -4.0;
pub const BISHOP_VALUE: f32 = 5.0;
pub const UNICORN_VALUE: f32 = 3.5;
pub const DRAGON_VALUE: f32 = 3.0;
pub const PAWN_VALUE: f32 = 0.9;

// How much it is worth to have a well-protected king
pub const KING_PROTECTION_VALUE: f32 = 1.5;
pub const KING_PROTECTION_VALUE_2: f32 = 2.5;

// How much it is worth to have branching priority
pub const BRANCH_VALUE: f32 = 4.0;
// How much it costs to have inactive timelines
pub const INACTIVE_BRANCH_COST: f32 = 20.0;
// Makes inactive branches (timelines) less important (ie. making a new, inactive timeline with a good board won't be worth as much as an active timeline)
pub const INACTIVE_BRANCH_MULTIPLIER: f32 = 0.8;
// Penalty for making a move on an inactive timeline
pub const INACTIVE_BOARD_MOVE_COST: f32 = 2.5;
// Penalty for having more than one king on a board
pub const MANY_KINGS_VALUE: f32 = -8.0;

// Bonus for each controlled square
pub const CONTROLLED_SQUARE_SCORE: f32 = 0.025;

/**
    Checks that `moveset` is legal and gives it a score. The `GameInfo` returned will correspond to that of the submitted move.
**/
pub fn score_moveset<'a, T: Iterator<Item = &'a Board>>(
    game: &Game,
    virtual_boards: &Vec<&Board>,
    info: &GameInfo,
    opponent_boards: T,
    moveset: Vec<Move>,
) -> Option<(Vec<Move>, Vec<Board>, GameInfo, f32)> {
    coz::scope!("score_moveset");
    let mut moveset_boards: Vec<Board> = Vec::new();
    let mut info = info.clone();
    let white = info.active_player;

    for mv in &moveset {
        let (new_info, mut new_vboards) =
            mv.generate_vboards(game, &info, &virtual_boards, &moveset_boards)?;
        moveset_boards.append(&mut new_vboards);
        info = new_info;
    }

    let merged_vboards: Vec<&Board> = virtual_boards
        .iter()
        .map(|x| *x)
        .chain(moveset_boards.iter())
        .collect();

    if is_moveset_legal(game, &merged_vboards, &info, moveset_boards.iter())
        && is_moveset_legal(game, &merged_vboards, &info, opponent_boards)
        && all_boards_played(game, &merged_vboards, &info)
    {
        info.present += 1;
        info.active_player = !info.active_player;

        let mut score: f32 = 0.0;

        for board in &moveset_boards {
            if board.t > info.present {
                score += if white {
                    -INACTIVE_BOARD_MOVE_COST
                } else {
                    INACTIVE_BOARD_MOVE_COST
                };
            }

            let board_mult: f32 = if board.l < 0.0 && -board.l > info.max_timeline + 1.0
                || board.l > 0.0 && board.l > -info.min_timeline + 1.0
            {
                INACTIVE_BRANCH_MULTIPLIER.powf((info.max_timeline + info.min_timeline).abs() - 1.0)
            } else {
                1.0
            };
            let mut w_kings: usize = 0;
            let mut b_kings: usize = 0;

            let mut controlled_squares_w: Vec<bool> = Vec::with_capacity(board.width * board.height);
            let mut controlled_squares_b: Vec<bool> = Vec::with_capacity(board.width * board.height);
            for _ in 0..(board.width * board.height) {
                controlled_squares_w.push(false);
                controlled_squares_b.push(false);
            }

            for (index, piece) in board.pieces.iter().enumerate() {
                let x = index % board.width;
                let y = index / board.width;
                if piece.is_blank() {
                    continue;
                }
                let mult: f32 = if piece.is_white() { 1.0 } else { -1.0 };
                if piece.is_king() {
                    if piece.is_white() {
                        w_kings += 1;
                        if w_kings > 1 {
                            score += MANY_KINGS_VALUE;
                        }
                    } else {
                        b_kings += 1;
                        if b_kings > 1 {
                            score -= MANY_KINGS_VALUE;
                        }
                    }
                    score += KING_VALUE * mult * board_mult;
                    for dx in -1..=1 {
                        for dy in -1..=1 {
                            if dx == 0 && dy == 0
                                || x == 0 && dx < 0
                                || y == 0 && dy < 0
                                || x == board.width - 1 && dx > 0
                                || y == board.height - 1 && dy > 0
                            {
                                continue;
                            }
                            if board
                                .get((x as isize + dx) as usize, (y as isize + dy) as usize)
                                .map(|p| p.is_blank() || p.is_opponent_piece(piece.is_white()))
                                .unwrap_or(false)
                            {
                                score -= KING_PROTECTION_VALUE * mult;

                                if board
                                    .get((x as isize + 2 * dx) as usize, (y as isize + 2 * dy) as usize)
                                    .map(|p| p.is_blank() || p.is_opponent_piece(piece.is_white()))
                                    .unwrap_or(false)
                                {
                                    score -= KING_PROTECTION_VALUE_2 * mult;
                                }
                            }
                        }
                    }
                } else if piece.is_knight() {
                    score += KNIGHT_VALUE * mult * board_mult;
                } else if piece.is_bishop() {
                    score += BISHOP_VALUE * mult * board_mult;
                } else if piece.is_rook() {
                    score += ROOK_VALUE * mult * board_mult;
                } else if piece.is_queen() {
                    score += QUEEN_VALUE * mult * board_mult;
                } else if piece.is_unicorn() {
                    score += UNICORN_VALUE * mult * board_mult;
                } else if piece.is_dragon() {
                    score += DRAGON_VALUE * mult * board_mult;
                } else if piece.is_pawn() {
                    score += PAWN_VALUE * mult * board_mult;
                } else if piece.is_princess() {
                    score += PRINCESS_VALUE * mult * board_mult;
                }

                // Maybe replace with bitboard operations
                // Or just dedupe that horror
                if piece.is_white() {
                    if piece.is_pawn() {
                        set_controlled_square(&mut controlled_squares_w, index, 1, 1, board.width, board.height);
                        set_controlled_square(&mut controlled_squares_w, index, 1, -1, board.width, board.height);
                    } else if piece.is_knight() {
                        set_controlled_square(&mut controlled_squares_w, index, 2, 1, board.width, board.height);
                        set_controlled_square(&mut controlled_squares_w, index, 2, -1, board.width, board.height);
                        set_controlled_square(&mut controlled_squares_w, index, -2, 1, board.width, board.height);
                        set_controlled_square(&mut controlled_squares_w, index, -2, -1, board.width, board.height);

                        set_controlled_square(&mut controlled_squares_w, index, 1, 2, board.width, board.height);
                        set_controlled_square(&mut controlled_squares_w, index, 1, -2, board.width, board.height);
                        set_controlled_square(&mut controlled_squares_w, index, -1, 2, board.width, board.height);
                        set_controlled_square(&mut controlled_squares_w, index, -1, -2, board.width, board.height);
                    }

                    if piece.is_bishop() || piece.is_queen() || piece.is_princess() {
                        set_controlled_square_slide(board, &mut controlled_squares_w, index, 1, 1, board.width, board.height, white);
                        set_controlled_square_slide(board, &mut controlled_squares_w, index, -1, 1, board.width, board.height, white);
                        set_controlled_square_slide(board, &mut controlled_squares_w, index, 1, -1, board.width, board.height, white);
                        set_controlled_square_slide(board, &mut controlled_squares_w, index, -1, -1, board.width, board.height, white);
                    }

                    if piece.is_rook() || piece.is_queen() || piece.is_princess() {
                        set_controlled_square_slide(board, &mut controlled_squares_w, index, 0, 1, board.width, board.height, white);
                        set_controlled_square_slide(board, &mut controlled_squares_w, index, 0, -1, board.width, board.height, white);
                        set_controlled_square_slide(board, &mut controlled_squares_w, index, 1, 0, board.width, board.height, white);
                        set_controlled_square_slide(board, &mut controlled_squares_w, index, -1, 0, board.width, board.height, white);
                    }
                } else if piece.is_black() {
                    if piece.is_pawn() {
                        set_controlled_square(&mut controlled_squares_b, index, -1, 1, board.width, board.height);
                        set_controlled_square(&mut controlled_squares_b, index, -1, -1, board.width, board.height);
                    } else if piece.is_knight() {
                        set_controlled_square(&mut controlled_squares_b, index, 2, 1, board.width, board.height);
                        set_controlled_square(&mut controlled_squares_b, index, 2, -1, board.width, board.height);
                        set_controlled_square(&mut controlled_squares_b, index, -2, 1, board.width, board.height);
                        set_controlled_square(&mut controlled_squares_b, index, -2, -1, board.width, board.height);

                        set_controlled_square(&mut controlled_squares_b, index, 1, 2, board.width, board.height);
                        set_controlled_square(&mut controlled_squares_b, index, 1, -2, board.width, board.height);
                        set_controlled_square(&mut controlled_squares_b, index, -1, 2, board.width, board.height);
                        set_controlled_square(&mut controlled_squares_b, index, -1, -2, board.width, board.height);
                    }

                    if piece.is_bishop() || piece.is_queen() || piece.is_princess() {
                        set_controlled_square_slide(board, &mut controlled_squares_b, index, 1, 1, board.width, board.height, white);
                        set_controlled_square_slide(board, &mut controlled_squares_b, index, -1, 1, board.width, board.height, white);
                        set_controlled_square_slide(board, &mut controlled_squares_b, index, 1, -1, board.width, board.height, white);
                        set_controlled_square_slide(board, &mut controlled_squares_b, index, -1, -1, board.width, board.height, white);
                    }

                    if piece.is_rook() || piece.is_queen() || piece.is_princess() {
                        set_controlled_square_slide(board, &mut controlled_squares_b, index, 0, 1, board.width, board.height, white);
                        set_controlled_square_slide(board, &mut controlled_squares_b, index, 0, -1, board.width, board.height, white);
                        set_controlled_square_slide(board, &mut controlled_squares_b, index, 1, 0, board.width, board.height, white);
                        set_controlled_square_slide(board, &mut controlled_squares_b, index, -1, 0, board.width, board.height, white);
                    }
                }
            }

            for controlled_square in controlled_squares_w {
                if controlled_square {
                    score += CONTROLLED_SQUARE_SCORE;
                }
            }

            for controlled_square in controlled_squares_b {
                if controlled_square {
                    score -= CONTROLLED_SQUARE_SCORE;
                }
            }

        }

        // Timeline advantages
        if info.max_timeline > -info.min_timeline {
            // black advantageous
            score -= BRANCH_VALUE;
            if info.max_timeline > -info.min_timeline + 1.0 {
                score -= INACTIVE_BRANCH_COST * (info.max_timeline + info.min_timeline - 1.0);
            }
        } else if info.max_timeline < -info.min_timeline {
            // white advantageous
            score += BRANCH_VALUE;
            if info.max_timeline < -info.min_timeline - 1.0 {
                score -= INACTIVE_BRANCH_COST * (info.max_timeline + info.min_timeline + 1.0);
            }
        }

        Some((moveset, moveset_boards, info, score))
    } else {
        None
    }
}

fn set_controlled_square(controlled_squares: &mut Vec<bool>, index: usize, dx: isize, dy: isize, width: usize, height: usize) {
    if
        ((index % width) as isize) + dx < 0
        || ((index % width) as isize) + dx >= width as isize
        || ((index / width) as isize) + dy < 0
        || ((index / width) as isize) + dy >= height as isize
    {
        return;
    }

    controlled_squares[((index as isize) + dx + (width as isize) * dy) as usize] = true;
}

fn set_controlled_square_slide(board: &Board, controlled_squares: &mut Vec<bool>, index: usize, dx: isize, dy: isize, width: usize, height: usize, white: bool) {
    let mut length = 1;

    while
        ((index % width) as isize) + length * dx >= 0
        && ((index % width) as isize) + length * dx < width as isize
        && ((index / width) as isize) + length * dy >= 0
        && ((index / width) as isize) + length * dy < height as isize
    {
        let n_index = ((index as isize) + length * (dx + (width as isize) * dy)) as usize;
        if board.pieces[n_index].is_takable_piece(white) {
            controlled_squares[n_index] = true;
        } else {
            break;
        }
        length += 1;
    }
}
