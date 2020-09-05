use crate::{game::*, moves::*, resolve::*};

// Tree search algorithms

type Node = (Vec<Move>, Vec<Board>, GameInfo, f32);

pub fn alphabeta(game: &Game, depth: usize, max_ms: usize, max_bf: usize) -> Option<(Node, f32)> {
    let virtual_boards: Vec<&Board> = Vec::new();
    let initial_iter = legal_movesets(game, &game.info, &virtual_boards, 0, 0).take(max_bf);

    if game.info.active_player {
        let mut value = std::f32::NEG_INFINITY;
        let mut best_move: Option<Node> = None;
        for mut node in initial_iter {
            if depth > 0 {
                node.2.active_player = !node.2.active_player;
                let new_value = alphabeta_rec(game, &virtual_boards, node.clone(), depth - 1, std::f32::NEG_INFINITY, std::f32::INFINITY, false, max_ms, max_bf);
                if new_value >= value {
                    value = new_value;
                    best_move = Some(node);
                }
            } else {
                if node.3 >= value {
                    value = node.3;
                    best_move = Some(node);
                }
            }
            if value == std::f32::INFINITY {
                break;
            }
        }
        best_move.map(|n| (n, value))
    } else {
        let mut value = std::f32::INFINITY;
        let mut best_move: Option<Node> = None;
        for mut node in initial_iter {
            if depth > 0 {
                node.2.active_player = !node.2.active_player;
                let new_value = alphabeta_rec(game, &virtual_boards, node.clone(), depth - 1, std::f32::NEG_INFINITY, std::f32::INFINITY, true, max_ms, max_bf);
                if new_value <= value {
                    value = new_value;
                    best_move = Some(node);
                }
            } else {
                if node.3 <= value {
                    value = node.3;
                    best_move = Some(node);
                }
            }
            if value == std::f32::NEG_INFINITY {
                break;
            }
        }
        best_move.map(|n| (n, value))
    }
}

fn alphabeta_rec(game: &Game, virtual_boards: &Vec<&Board>, node: Node, depth: usize, mut alpha: f32, mut beta: f32, white: bool, max_ms: usize, max_bf: usize) -> f32 {
    if depth == 0 {
        node.3
    } else {
        let mut info = node.2.clone();
        info.active_player = white;
        let merged_vboards: Vec<&Board> = virtual_boards.iter().map(|x| *x).chain(node.1.iter()).collect::<Vec<&Board>>();
        let movesets = legal_movesets(game, &info, &merged_vboards, 0, max_ms).take(max_bf);

        if white {
            let mut value = std::f32::NEG_INFINITY;
            for ms in movesets {
                value = alphabeta_rec(game, &merged_vboards, ms, depth - 1, alpha, beta, false, max_ms, max_bf).max(value);
                alpha = alpha.max(value);
                if alpha >= beta {
                    break;
                }
            }
            // println!("Depth {} as white, value {}", depth, value);
            value
        } else {
            let mut value = std::f32::INFINITY;
            for ms in movesets {
                value = alphabeta_rec(game, &merged_vboards, ms, depth - 1, alpha, beta, true, max_ms, max_bf).min(value);
                beta = beta.min(value);
                if beta <= alpha {
                    break;
                }
            }
            // println!("Depth {} as black, value {}", depth, value);
            value
        }
    }
}
