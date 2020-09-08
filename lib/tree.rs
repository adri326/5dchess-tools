use crate::{game::*, moves::*, moveset::*};
use std::sync::{Arc, Mutex};

// Tree search algorithms

type Node = (Vec<Move>, Vec<Board>, GameInfo, f32);

pub fn alphabeta<'a>(
    game: &'a Game,
    depth: usize,
    max_ms: usize,
    bucket_size: usize,
    max_bf: usize,
    n_threads: usize,
) -> Option<(Node, f32)> {
    // let mut pool = Pool::new(n_threads as u32);
    let virtual_boards: Vec<&Board> = Vec::new();
    let initial_iter = legal_movesets(&game, &game.info, &virtual_boards, 0, 0).take(max_bf);

    let res_data = Arc::new(Mutex::new((
        None,
        if game.info.active_player {
            std::f32::NEG_INFINITY
        } else {
            std::f32::INFINITY
        },
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
                            if if info.active_player {
                                res_data.1 == std::f32::INFINITY
                            } else {
                                res_data.1 == std::f32::NEG_INFINITY
                            } {
                                return;
                            }
                        }
                        _ => panic!("Couldn't lock res_data"),
                    }
                }

                if depth > 0 {
                    node.2.active_player = !node.2.active_player;
                    let (best_branch, new_value) = alphabeta_rec(
                        &game,
                        &virtual_boards,
                        node.clone(),
                        depth - 1,
                        std::f32::NEG_INFINITY,
                        std::f32::INFINITY,
                        !node.2.active_player,
                        max_ms,
                        bucket_size,
                        max_bf,
                    );
                    if let Some(best_branch) = best_branch {
                        println!("1. {:?}", node.0);
                        for (k, mv) in best_branch.iter().enumerate() {
                            println!("{}. {:?}", k + 2, mv.0);
                        }
                    }
                    println!("{:?} -> {}", node.0, new_value);
                    match res_data.lock() {
                        Ok(mut res_data) => {
                            if if info.active_player {
                                new_value > res_data.1
                            } else {
                                new_value < res_data.1
                            } {
                                res_data.1 = new_value;
                                res_data.0 = Some(node);
                            }
                        }
                        _ => panic!("Couldn't lock res_data"),
                    }
                } else {
                    match res_data.lock() {
                        Ok(mut res_data) => {
                            if if info.active_player {
                                node.3 > res_data.1
                            } else {
                                node.3 < res_data.1
                            } {
                                res_data.1 = node.3;
                                res_data.0 = Some(node);
                            }
                        }
                        _ => panic!("Couldn't lock res_data"),
                    }
                }
            });
        }
    })
    .unwrap();

    let res = {
        match res_data.lock() {
            Ok(res_data) => res_data.clone(),
            _ => panic!(),
        }
    };

    match res {
        (Some(n), v) => Some((n, v)),
        _ => None,
    }
}

fn alphabeta_rec(
    game: &Game,
    virtual_boards: &Vec<&Board>,
    node: Node,
    depth: usize,
    mut alpha: f32,
    mut beta: f32,
    white: bool,
    max_ms: usize,
    bucket_size: usize,
    max_bf: usize,
) -> (Option<Vec<Node>>, f32) {
    if depth == 0 {
        let s = node.3;
        (None, s)
    } else {
        let mut info = node.2.clone();
        info.active_player = white;
        let merged_vboards: Vec<&Board> = virtual_boards
            .iter()
            .map(|x| *x)
            .chain(node.1.iter())
            .collect::<Vec<&Board>>();
        let movesets = legal_movesets(game, &info, &merged_vboards, 0, max_ms);

        if white {
            let mut value = std::f32::NEG_INFINITY;
            let mut yielded_move = false;
            let mut best_move: Option<Vec<Node>> = None;
            for ms in opt_apply_bucket(bucket_size, max_bf, white, movesets) {
                if ms.0.len() > game.timelines.len() * 5 {
                    println!("Abnormally high number of dimensions: {}", ms.0.len());
                    println!("{:?}", ms.0);
                }
                yielded_move = true;
                let (best_branch, n_value) = alphabeta_rec(
                    game,
                    &merged_vboards,
                    ms.clone(),
                    depth - 1,
                    alpha,
                    beta,
                    false,
                    max_ms,
                    bucket_size,
                    max_bf,
                );
                if n_value > value {
                    if let Some(mut best_branch) = best_branch {
                        best_move = Some({
                            let mut res = vec![ms];
                            res.append(&mut best_branch);
                            res
                        });
                    } else {
                        best_move = Some(vec![ms]);
                    }
                    value = n_value;
                    alpha = alpha.max(value);
                }
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

            (best_move, value)
        } else {
            let mut value = std::f32::INFINITY;
            let mut yielded_move = false;
            let mut best_move: Option<Vec<Node>> = None;
            for ms in opt_apply_bucket(bucket_size, max_bf, white, movesets) {
                if ms.0.len() > game.timelines.len() * 5 {
                    println!("Abnormally high number of dimensions: {}", ms.0.len());
                    println!("{:?}", ms.0);
                }
                yielded_move = true;
                let (best_branch, n_value) = alphabeta_rec(
                    game,
                    &merged_vboards,
                    ms.clone(),
                    depth - 1,
                    alpha,
                    beta,
                    true,
                    max_ms,
                    bucket_size,
                    max_bf,
                );
                if n_value < value {
                    if let Some(mut best_branch) = best_branch {
                        best_move = Some({
                            let mut res = vec![ms];
                            res.append(&mut best_branch);
                            res
                        });
                    } else {
                        best_move = Some(vec![ms]);
                    }
                    value = n_value;
                    beta = beta.min(value);
                }
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

            (best_move, value)
        }
    }
}

fn opt_apply_bucket<'a, T: Iterator<Item=Node> + 'a>(bucket_size: usize, max_bf: usize, white: bool, iter: T) -> Box<dyn Iterator<Item=Node> + 'a> {
    if bucket_size > max_bf {
        let mut res: Vec<Node> = iter.take(bucket_size).collect();
        res.sort_by(|a, b| if white {b.3.partial_cmp(&a.3).unwrap()} else {a.3.partial_cmp(&b.3).unwrap()});
        Box::new(res.into_iter().take(max_bf))
    } else {
        Box::new(iter.take(max_bf))
    }
}
