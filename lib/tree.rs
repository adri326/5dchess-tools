use crate::{game::*, moves::*, moveset::*};
use std::sync::{Arc, Mutex};

// Tree search algorithms

type Node = (Vec<Move>, Vec<Board>, GameInfo, f32);

pub fn alphabeta<'a>(game: &'a Game, depth: usize, max_ms: usize, max_bf: usize, n_threads: usize) -> Option<(Node, f32)> {
    // let mut pool = Pool::new(n_threads as u32);
    let virtual_boards: Vec<&Board> = Vec::new();
    let initial_iter = legal_movesets(&game, &game.info, &virtual_boards, 0, 0).take(max_bf);

    let res_data = Arc::new(Mutex::new((
        None,
        if game.info.active_player {std::f32::NEG_INFINITY} else {std::f32::INFINITY}
    )));

    crossbeam::scope(|scope| {
        for mut node in initial_iter {
            let virtual_boards: Vec<&Board> = Vec::new();
            let info = game.info.clone();
            let depth = depth;
            let res_data = Arc::clone(&res_data);

            scope.spawn(move |_| {
                {
                    match res_data.lock() {
                        Ok(res_data) => {
                            if if info.active_player {res_data.1 == std::f32::INFINITY} else {res_data.1 == std::f32::NEG_INFINITY} {
                                return;
                            }
                        }
                        _ => panic!("Couldn't lock res_data")
                    }
                }

                if depth > 0 {
                    node.2.active_player = !node.2.active_player;
                    let new_value = alphabeta_rec(&game, &virtual_boards, node.clone(), depth - 1, std::f32::NEG_INFINITY, std::f32::INFINITY, !node.2.active_player, max_ms, max_bf);
                    println!("{:?} -> {}", node.0, new_value);
                    match res_data.lock() {
                        Ok(mut res_data) => {
                            if if info.active_player {new_value >= res_data.1} else {new_value <= res_data.1} {
                                res_data.1 = new_value;
                                res_data.0 = Some(node);
                            }
                        }
                        _ => panic!("Couldn't lock res_data")
                    }
                } else {
                    match res_data.lock() {
                        Ok(mut res_data) => {
                            if if info.active_player {node.3 >= res_data.1} else {node.3 <= res_data.1} {
                                res_data.1 = node.3;
                                res_data.0 = Some(node);
                            }
                        }
                        _ => panic!("Couldn't lock res_data")
                    }
                }
            });
        }
    }).unwrap();

    let res = {
        match res_data.lock() {
            Ok(res_data) => res_data.clone(),
            _ => panic!()
        }
    };

    match res {
        (Some(n), v) => Some((n, v)),
        _ => None
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
            let mut yielded_move = false;
            for ms in movesets {
                yielded_move = true;
                value = alphabeta_rec(game, &merged_vboards, ms, depth - 1, alpha, beta, false, max_ms, max_bf).max(value);
                alpha = alpha.max(value);
                if alpha >= beta {
                    break;
                }
            }
            if !yielded_move {
                // Look for a draw
                if is_draw(game, &merged_vboards, &info) {
                    value = 0.0;
                }
            }
            value
        } else {
            let mut value = std::f32::INFINITY;
            let mut yielded_move = false;
            for ms in movesets {
                yielded_move = true;
                value = alphabeta_rec(game, &merged_vboards, ms, depth - 1, alpha, beta, true, max_ms, max_bf).min(value);
                beta = beta.min(value);
                if beta <= alpha {
                    break;
                }
            }
            if !yielded_move {
                // Look for a draw
                if is_draw(game, &merged_vboards, &info) {
                    value = 0.0;
                }
            }
            value
        }
    }
}
