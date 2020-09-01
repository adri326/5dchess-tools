use crate::{game::*, moves::*};

pub const JUMP_COST: i32 = -2;
pub const JUMP_INACTIVE_COST: i32 = -6;
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
pub const TAKE_QUEEN_REWARD: i32 = 10;
pub const TAKE_UNICORN_REWARD: i32 = 2;
pub const TAKE_DRAGON_REWARD: i32 = 2;

#[derive(Debug)]
pub struct Lore<'a> {
    pub board: &'a Board,
    pub danger: Vec<usize>,
    pub enemies: Vec<(f32, usize, usize, usize)>,
}

/**
    Generates a board's "Lore" (danger map and target pieces)
**/
pub fn generate_lore<'a, 'b, T: Iterator<Item = &'b Board>>(
    game: &Game,
    virtual_boards: &Vec<Board>,
    board: &'a Board,
    opponent_boards: T,
    info: &GameInfo,
) -> Lore<'a> {
    let mut res = Lore {
        board,
        danger: vec![0; board.pieces.len()],
        enemies: Vec::new(),
    };

    let mut noop_board = board.clone();
    noop_board.t += 1;

    let mut n_virtual_boards = virtual_boards.clone();
    n_virtual_boards.push(noop_board.clone());

    for b in opponent_boards {
        let probables = probable_moves(game, b, &n_virtual_boards);
        for mv in probables {
            if mv.dst_piece.is_king() {
                register_enemy(&mut res, &mv);
            }
            register_danger(&mut res, &mv);
        }
    }

    let probables = probable_moves(game, &noop_board, &n_virtual_boards);
    for mv in probables {
        if mv.dst_piece.is_king() {
            register_enemy(&mut res, &mv);
        }
        register_danger(&mut res, &mv);
    }

    res
}

fn register_enemy(res: &mut Lore, mv: &Move) {
    if !res.enemies.iter().find(|e| **e == mv.src).is_some() {
        res.enemies.push(mv.src);
    }
}

fn register_danger(res: &mut Lore, mv: &Move) {
    if mv.dst.0 == res.board.l && (mv.dst.1 == res.board.t + 1 || mv.dst.1 == res.board.t) {
        res.danger[mv.dst.2 + mv.dst.3 * res.board.width] += 1;
    }
}

pub fn score_moves<'a>(
    game: &Game,
    virtual_boards: &Vec<Board>,
    board: &'a Board,
    lore: &Lore<'a>,
    moves: Vec<(Move, GameInfo, Vec<Board>)>,
    info: &GameInfo,
) -> Vec<(Move, Vec<Board>, GameInfo, i32)> {
    let mut res = moves
        .into_iter()
        .map(|(mv, info, boards)| {
            let mut score: i32 = 0;

            if mv.src.0 != mv.dst.0 || mv.src.1 != mv.dst.1 {
                if if info.active_player {
                    info.max_timeline > -info.min_timeline + 1.0
                } else {
                    info.max_timeline < -info.min_timeline - 1.0
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
            }

            for b in &boards {
                for (index, piece) in b.pieces.iter().enumerate() {
                    if *piece != Piece::Blank && piece.is_white() == board.active_player() {
                        if piece.is_king() {
                            score += (lore.danger[index] as i32) * KING_DANGER_COST;
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
        .collect::<Vec<_>>();
    res.sort_unstable_by_key(|(_mv, _boards, _info, score)| *score);
    res
}

pub const ROOK_VALUE: f32 = 3.0;
pub const KNIGHT_VALUE: f32 = 4.5;
pub const QUEEN_VALUE: f32 = 12.0;
pub const KING_VALUE: f32 = -1.0;
pub const BISHOP_VALUE: f32 = 4.0;
pub const UNICORN_VALUE: f32 = 3.5;
pub const DRAGON_VALUE: f32 = 3.0;
pub const PAWN_VALUE: f32 = 0.9;
pub const KING_PROTECTION_VALUE: f32 = -3.0;

/**
    Checks that `moveset` is legal and gives it a score.
**/
pub fn score_moveset<'a, T: Iterator<Item = &'a Board>>(
    game: &Game,
    virtual_boards: &Vec<Board>,
    info: &GameInfo,
    opponent_boards: T,
    moveset: Vec<Move>,
) -> Option<(Vec<Move>, Vec<Board>, GameInfo, f32)> {
    let mut moveset_boards: Vec<Board> = Vec::new();
    let mut merged_vboards: Vec<Board> = virtual_boards.clone();
    let mut info = info.clone();
    let active_player = info.active_player;
    for mv in &moveset {
        let (new_info, mut new_vboards) = mv.generate_vboards(game, &info, virtual_boards)?;
        moveset_boards.append(&mut new_vboards);
        info = new_info;
    }

    merged_vboards.append(&mut moveset_boards.clone());

    if is_move_legal(game, &merged_vboards, &info, moveset_boards.iter())
        && is_move_legal(game, &merged_vboards, &info, opponent_boards)
    {
        info.present += 1;
        info.active_player = !info.active_player;

        println!("Yay!");
        let mut score: f32 = 0.0;

        for board in &moveset_boards {
            for (index, piece) in board.pieces.iter().enumerate() {
                let x = index % board.width;
                let y = index / board.width;
                if piece.is_blank() {
                    continue;
                }
                let mult: f32 = if piece.is_white() { 1.0 } else { -1.0 };
                if piece.is_king() {
                    score += KING_VALUE * mult;
                    for dx in -1..=1 {
                        for dy in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            if board
                                .get((x as isize + dx) as usize, (y as isize + dy) as usize)
                                .map(|p| p.is_blank() || p.is_opponent_piece(piece.is_white()))
                                .unwrap_or(false)
                            {
                                score += KING_PROTECTION_VALUE * mult;
                            }
                        }
                    }
                } else if piece.is_knight() {
                    score += KNIGHT_VALUE * mult;
                } else if piece.is_knight() {
                    score += KNIGHT_VALUE * mult;
                } else if piece.is_bishop() {
                    score += BISHOP_VALUE * mult;
                } else if piece.is_rook() {
                    score += ROOK_VALUE * mult;
                } else if piece.is_queen() {
                    score += QUEEN_VALUE * mult;
                } else if piece.is_unicorn() {
                    score += UNICORN_VALUE * mult;
                } else if piece.is_dragon() {
                    score += DRAGON_VALUE * mult;
                } else if piece.is_pawn() {
                    score += PAWN_VALUE * mult;
                }
            }
        }

        Some((moveset, moveset_boards, info, score))
    } else {
        None
    }
}