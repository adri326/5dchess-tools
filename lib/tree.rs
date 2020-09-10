use crate::{game::*, moves::*, moveset::*};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use scoped_threadpool::Pool;

// Tree search algorithms

type Node = (Vec<Move>, Vec<Board>, GameInfo, f32);

pub fn alphabeta<'a>(
    game: &'a Game,
    depth: usize,
    max_ms: usize,
    bucket_size: usize,
    max_bf: usize,
    n_threads: u32,
) -> Option<(Node, f32)> {
    let virtual_boards: Vec<&Board> = Vec::new();
    let initial_iter = legal_movesets(&game, &game.info, &virtual_boards, 0, 0).take(max_bf);
    let mut pool = Pool::new(n_threads);

    let res_data = Arc::new(Mutex::new((
        None,
        if game.info.active_player {
            std::f32::NEG_INFINITY
        } else {
            std::f32::INFINITY
        },
    )));

    pool.scoped(|scope| {
        for mut node in initial_iter {
            let virtual_boards: Vec<&Board> = Vec::new();
            let info = game.info.clone();
            let depth = depth;
            let res_data = Arc::clone(&res_data);

            scope.execute(move || {
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
                        let mut res: String = format!("1. {:?} -> {}\n", node.0, new_value);
                        for (k, mv) in best_branch.iter().enumerate() {
                            res.push_str(format!("{}. {:?}\n", k + 2, mv.0).as_str());
                        }
                        println!("{}", res);
                    } else {
                        println!("1. {:?} -> {}", node.0, new_value);
                    }
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
    });

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
                if ms.0.len() > game.timelines.len() * 20 {
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
                if ms.0.len() > game.timelines.len() * 20 {
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

fn opt_apply_bucket<'a, T: Iterator<Item = Node> + 'a>(
    bucket_size: usize,
    max_bf: usize,
    white: bool,
    iter: T,
) -> Box<dyn Iterator<Item = Node> + 'a> {
    if bucket_size > max_bf {
        let mut res: Vec<Node> = iter.take(bucket_size).collect();
        res.sort_by(|a, b| {
            if white {
                b.3.partial_cmp(&a.3).unwrap()
            } else {
                a.3.partial_cmp(&b.3).unwrap()
            }
        });
        Box::new(res.into_iter().take(max_bf))
    } else {
        Box::new(iter.take(max_bf))
    }
}

pub fn iterative_deepening<'a>(
    game: &'a Game,
    max_ms: usize,
    bucket_size: usize,
    bucket_downsize: usize,
    pool_size: usize,
    initial_movesets: usize,
    tolerance: f32,
    tolerance_mult: f32,
    n_threads: u32,
    max_duration: Duration,
) -> Option<(Node, f32)> {
    let mut pool = Pool::new(n_threads);
    let mut res = pool.scoped(|scope| {
        let initial_virtual_boards: Vec<&Board> = Vec::new();
        let children = Arc::new(Mutex::new(Vec::new()));
        for initial_node in
            legal_movesets(game, &game.info, &initial_virtual_boards, 0, 0).take(initial_movesets)
        {
            let children = Arc::clone(&children);
            scope.execute(move || {
                let res = iterative_deepening_sub(
                    game,
                    initial_node.clone(),
                    max_ms,
                    bucket_size,
                    bucket_downsize,
                    pool_size,
                    tolerance,
                    tolerance_mult,
                    max_duration,
                );
                children.lock().unwrap().push((initial_node, res));
            });
        }
        scope.join_all();
        let res = children.lock().unwrap().clone();
        res
    });

    if game.info.active_player {
        res.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    } else {
        res.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    }

    // Need to hold the state of the search per branch
    // Have some branch pruning action?
    res.pop()
}

#[derive(Clone, Debug)]
struct IDBranch {
    boards: Vec<Board>,
    moves: Vec<Vec<Move>>,
    info: GameInfo,
    depth: usize,
    score: f32,
    tree: RIDTree,
}

impl From<(Node, &IDBranch, RIDTree)> for IDBranch {
    fn from(raw: (Node, &IDBranch, RIDTree)) -> Self {
        IDBranch {
            boards: (raw.1)
                .boards
                .clone()
                .into_iter()
                .chain(((raw.0).1).into_iter())
                .collect(),
            moves: (raw.1)
                .moves
                .clone()
                .into_iter()
                .chain(vec![(raw.0).0].into_iter())
                .collect(),
            info: (raw.0).2,
            depth: raw.1.depth + 1,
            score: (raw.0).3,
            tree: raw.2,
        }
    }
}

type RIDTree = Rc<RefCell<IDTree>>;

#[derive(Debug)]
struct IDTree {
    depth: usize,
    white: bool,
    children: Vec<RIDTree>,
    score: f32,
    pruned: bool,
}

impl IDTree {
    fn after(node: &RIDTree, score: f32) -> Option<RIDTree> {
        let mut node = node.borrow_mut();

        let res = Rc::new(RefCell::new(IDTree {
            depth: node.depth + 1,
            white: !node.white,
            children: Vec::new(),
            score,
            pruned: false,
        }));

        node.children.push(res.clone());
        Some(res)
    }
}

fn iterative_deepening_sub<'a>(
    game: &'a Game,
    initial_node: Node,
    max_ms: usize,
    bucket_size: usize,
    bucket_downsize: usize,
    pool_size: usize,
    mut tolerance: f32,
    tolerance_mult: f32,
    max_duration: Duration,
) -> f32 {
    let mut pool: VecDeque<IDBranch> = VecDeque::with_capacity(pool_size * 2);
    let initial_tree = Rc::new(RefCell::new(IDTree {
        depth: 0,
        children: vec![],
        score: if initial_node.2.active_player {
            std::f32::INFINITY
        } else {
            std::f32::NEG_INFINITY
        },
        white: !initial_node.2.active_player,
        pruned: false,
    }));
    pool.push_back(IDBranch {
        moves: vec![initial_node.0],
        boards: initial_node.1,
        info: initial_node.2,
        score: initial_node.3,
        depth: 0,
        tree: initial_tree.clone(),
    });
    let begin = Instant::now();

    let mut consecutive_prunes: usize = 0;

    while begin.elapsed() < max_duration {
        if pool.len() > pool_size {
            consecutive_prunes += 1;
            if consecutive_prunes > 1 {
                tolerance *= tolerance_mult;
            }
            iterative_deepening_prune(&mut pool, initial_tree.clone(), tolerance);
        } else {
            consecutive_prunes = 0;
            if let Some(mut branch) = pool.pop_front() {
                let virtual_boards = branch.boards.iter().collect::<Vec<_>>();
                let mut movesets = legal_movesets(game, &branch.info, &virtual_boards, 0, max_ms)
                    .take(bucket_size)
                    .collect::<Vec<_>>();
                movesets.sort_by(|a, b| {
                    if branch.info.active_player {
                        b.3.partial_cmp(&a.3).unwrap()
                    } else {
                        a.3.partial_cmp(&b.3).unwrap()
                    }
                });

                if movesets.len() > 0 {
                    for node in movesets.into_iter().take(bucket_downsize) {
                        if pool.len() < pool_size * 2 {
                            if let Some(new_tree) = IDTree::after(&branch.tree, node.3) {
                                pool.push_back(IDBranch::from((node, &branch, new_tree)));
                            }
                        }
                    }
                } else {
                    if is_draw(game, &virtual_boards, &branch.info) {
                        branch.score = 0.0;
                        branch.tree.borrow_mut().score = branch.score;
                    } else {
                        branch.score = if branch.info.active_player {
                            std::f32::NEG_INFINITY
                        } else {
                            std::f32::INFINITY
                        };
                        branch.tree.borrow_mut().score = branch.score;
                    }
                    pool.push_back(branch);
                }
            } else {
                break;
            }
        }
    }

    iterative_deepening_prune(&mut pool, initial_tree.clone(), 0.0);

    let score = initial_tree.borrow().score;

    let mut res_str = String::new();
    for (k, mv) in pool
        .pop_front()
        .expect("Expected pool to contain at least one item!")
        .moves
        .iter()
        .enumerate()
    {
        res_str.push_str(format!("{}: {:?}\n", k + 1, mv).as_str());
    }
    println!("{} -> {}", res_str, score);

    score
}

fn iterative_deepening_prune(pool: &mut VecDeque<IDBranch>, initial_tree: RIDTree, tolerance: f32) {
    iterative_deepening_prune_rec(&initial_tree);
    iterative_deepening_prune_rec2(&initial_tree, false, tolerance);
    for _ in 0..pool.len() {
        let node = pool.pop_front().unwrap();
        if !node.tree.borrow().pruned {
            pool.push_back(node);
        }
    }
}

fn iterative_deepening_prune_rec(tree: &RIDTree) {
    if tree.borrow().children.len() == 0 {
        return;
    }
    let children = tree
        .borrow()
        .children
        .clone()
        .into_iter()
        .inspect(iterative_deepening_prune_rec);
    let white = tree.borrow().white;
    if !white {
        let mut score = std::f32::NEG_INFINITY;
        for c in children {
            score = score.max(c.borrow().score);
        }
        tree.borrow_mut().score = score;
    } else {
        let mut score = std::f32::INFINITY;
        for c in children {
            score = score.min(c.borrow().score);
        }
        tree.borrow_mut().score = score;
    }
}

fn iterative_deepening_prune_rec2(tree: &RIDTree, prune: bool, tolerance: f32) {
    let score = tree.borrow().score;
    let white = tree.borrow().white;
    tree.borrow_mut().pruned = prune;
    if prune {
        for c in tree.borrow().children.iter() {
            iterative_deepening_prune_rec2(c, true, tolerance);
        }
        if tree.borrow().children.len() > 0 {
            tree.borrow_mut().children = Vec::new();
        }
    }

    if !white {
        for c in tree.borrow().children.iter() {
            let should_prune = c.borrow().score < score - tolerance;
            iterative_deepening_prune_rec2(c, should_prune, tolerance);
        }
        let children = tree
            .borrow()
            .children
            .clone()
            .into_iter()
            .filter(|c| !c.borrow().pruned)
            .collect::<Vec<_>>();
        tree.borrow_mut().children = children;
    } else {
        for c in tree.borrow().children.iter() {
            let should_prune = c.borrow().score > score + tolerance;
            iterative_deepening_prune_rec2(c, should_prune, tolerance);
        }
        let children = tree
            .borrow()
            .children
            .clone()
            .into_iter()
            .filter(|c| !c.borrow().pruned)
            .collect::<Vec<_>>();
        tree.borrow_mut().children = children;
    }
}
