use crate::{game::*, moves::*};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use scoped_threadpool::Pool;

// Tree search algorithms

type Node = (Vec<Move>, Vec<Board>, GameInfo, f32);

pub mod dfs {
    use super::*;
    /**
        b-limited αβ-pruned depth-first search

        - `depth` is the depth to which the algorithm will look (`d`)
        - `max_ms` corresponds to the maximum number of probable movesets to consider before admitting that no moveset can be made. Set to 0 for ∞ (not recommended!)
        - `bucket_size` correspond to the number of movesets to score and sort; ignored if `<= max_bf`
        - `max_bf` corresponds to the maximum number of movesets (branching factor, or `b`) to consider per tree node; note that αβ-pruning has a time complexity of `O(b^(d/2))`
        - `n_threads` is the number of threads to run concurrently; they will work on different starting moves to recursively rate them
    **/
    pub fn dfs<'a>(
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
            for node in initial_iter {
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
                        let (best_branch, new_value) = dfs_rec(
                            &game,
                            &virtual_boards,
                            node.clone(),
                            depth - 1,
                            std::f32::NEG_INFINITY,
                            std::f32::INFINITY,
                            node.2.active_player,
                            max_ms,
                            bucket_size,
                            max_bf,
                        );
                        if let Some(best_branch) = best_branch {
                            let mut res: String = format!("1. {:?} -> {}\n", node.0, new_value);
                            for (k, mv) in best_branch.iter().enumerate() {
                                res.push_str(format!("{}. {:?}\n", k + 2, mv.0).as_str());
                            }
                            info!("{}", res);
                        } else {
                            info!("1. {:?} -> {}", node.0, new_value);
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

    /// Recursive bit of `dfs(...)`, see the documentation about `dfs` for more information!
    fn dfs_rec(
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
        // TODO: merge white's and black's code?
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

            if white { // White:
                let mut value = std::f32::NEG_INFINITY;
                let mut yielded_move = false;
                let mut best_move: Option<Vec<Node>> = None;
                for ms in opt_apply_bucket(bucket_size, max_bf, white, movesets) {
                    if ms.0.len() > game.timelines.len() * 20 {
                        info!("Abnormally high number of dimensions: {}", ms.0.len());
                        info!("{:?}", ms.0);
                    }
                    yielded_move = true;
                    let (best_branch, n_value) = dfs_rec(
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
            } else { // Black:
                let mut value = std::f32::INFINITY;
                let mut yielded_move = false;
                let mut best_move: Option<Vec<Node>> = None;
                for ms in opt_apply_bucket(bucket_size, max_bf, white, movesets) {
                    if ms.0.len() > game.timelines.len() * 20 {
                        info!("Abnormally high number of dimensions: {}", ms.0.len());
                        info!("{:?}", ms.0);
                    }
                    yielded_move = true;
                    let (best_branch, n_value) = dfs_rec(
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
}

pub mod bfs {
    use super::*;
    /** b-limited Breadth-first search with periodical pruning.

        This algorithm works on a growable ring (or queue; it should not need to be resized if bucket_downsize < pool_size). As a node is taken out of the beginning of the queue, its child nodes are added at the end of the queue (or itself, with its score being updated, should there be no legal moves following that node).

        A separate tree is built to keep track of each node's score. Each node's ancestors are not updated when expanding the tree.

        Pruning happens when the pool's size exceeds `pool_size` (it is initially allocated with `2 * pool_size` elements as to prevent reallocations).
        It happens in three steps:

        - the tree is traversed once, to recalculate the scores of each node assuming perfect play
        - the tree is traversed a second time: branches of the tree are marked as prunable should their score differ by more than `tolerance` from the best score; pruned branches are removed from the tree to ease future prunings
        - the ring vector is fully traversed once, popping elements and only pushing them back in if their corresponding node was not pruned

        As such, this method alternates between deepening and pruning, until the maximum duration is reached.

        This tree search method is, because of its pruning, less precise than the αβ search. This precision depends on the quality of the ranking methods.
        To accomodate for the inevitable inaccuracy, the `tolerance` and `tolerance_mult` options have been introduced:

        - `tolerance` is the maximum score difference from the best scoring node that there can be for a branch to not be pruned. If `0`, only the best scoring branches will be kept; they might turn out to not score as well deeper down the tree.
        - `tolerance_mult` is the multiplier for that score difference that will be applied to it should there be more than one consecutive pruning step; it must be lower than 1 (or else this algorithm will loop forever).
        - The `pool_size` option can also be increased to reduce the number of times that the pruning has to be ran. Doing so will, however, increase the memory usage of the program.
    **/
    pub fn bfs<'a>(
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
                    let res = bfs_sub(
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

    /**
        A branch or node for `bfs(...)`
    **/
    #[derive(Clone, Debug)]
    struct IDBranch {
        boards: Vec<Board>,
        moves: Vec<Vec<Move>>,
        info: GameInfo,
        depth: usize,
        score: f32,
        tree: RBFSTree,
    }

    impl From<(Node, &IDBranch, RBFSTree)> for IDBranch {
        fn from(raw: (Node, &IDBranch, RBFSTree)) -> Self {
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

    type RBFSTree = Rc<RefCell<BFSTree>>;

    /**
        Tree built alongside the different `IDBranch`es to keep track of which branch needs to be pruned.
    **/
    #[derive(Debug)]
    struct BFSTree {
        depth: usize,
        white: bool,
        children: Vec<RBFSTree>,
        score: f32,
        pruned: bool,
    }

    impl BFSTree {
        /// Creates a new BFSTree instance, as a child from `node`
        fn after(node: &RBFSTree, score: f32) -> Option<RBFSTree> {
            let mut node = node.borrow_mut();

            let res = Rc::new(RefCell::new(BFSTree {
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

    /// Per-thread bit of the `bfs(...)` method. See this function's documentation for more information.
    fn bfs_sub<'a>(
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
        let initial_tree = Rc::new(RefCell::new(BFSTree {
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
        let mut n_nodes: usize = 1;

        while begin.elapsed() < max_duration {
            if pool.len() > pool_size {
                consecutive_prunes += 1;
                if consecutive_prunes > 1 {
                    tolerance *= tolerance_mult;
                }
                bfs_prune(&mut pool, initial_tree.clone(), tolerance);
            } else {
                consecutive_prunes = 0;
                if let Some(mut branch) = pool.pop_front() {
                    if if branch.info.active_player {branch.score == std::f32::NEG_INFINITY} else {branch.score == std::f32::INFINITY} {
                        pool.push_back(branch);
                        continue;
                    }
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
                            n_nodes += 1;
                            if pool.len() < pool_size * 2 {
                                if let Some(new_tree) = BFSTree::after(&branch.tree, node.3) {
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
                        if pool.len() == 1 {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        }

        info!("{} nodes in {}; {} N/s", n_nodes, max_duration.as_secs_f32(), (n_nodes as f32) / max_duration.as_secs_f32());

        bfs_prune(&mut pool, initial_tree.clone(), 0.0);

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
        info!("{} -> {}", res_str, score);

        score
    }

    /// Runs the different pruning steps as described in `bfs(...)`'s documentation
    fn bfs_prune(pool: &mut VecDeque<IDBranch>, initial_tree: RBFSTree, tolerance: f32) {
        bfs_prune_rec(&initial_tree);
        bfs_prune_rec_2(&initial_tree, false, tolerance);
        for _ in 0..pool.len() {
            let node = pool.pop_front().unwrap();
            if !node.tree.borrow().pruned {
                pool.push_back(node);
            }
        }
    }

    /// First step of the pruning: re-calculate the score of each branch
    fn bfs_prune_rec(tree: &RBFSTree) {
        if tree.borrow().children.len() == 0 {
            return;
        }
        let children = tree
            .borrow()
            .children
            .clone()
            .into_iter()
            .inspect(bfs_prune_rec);
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

    /// Second step of the pruning: mark branches as pruned
    fn bfs_prune_rec_2(tree: &RBFSTree, prune: bool, tolerance: f32) {
        let score = tree.borrow().score;
        let white = tree.borrow().white;
        tree.borrow_mut().pruned = prune;
        if prune {
            for c in tree.borrow().children.iter() {
                bfs_prune_rec_2(c, true, tolerance);
            }
            if tree.borrow().children.len() > 0 {
                tree.borrow_mut().children = Vec::new();
            }
        }

        if !white {
            for c in tree.borrow().children.iter() {
                let should_prune = c.borrow().score < score - tolerance;
                bfs_prune_rec_2(c, should_prune, tolerance);
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
                bfs_prune_rec_2(c, should_prune, tolerance);
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
}

/// Optionally applies the `bucket_size` option to the legal movesets iterator; `bucket_size` will be ignored if it is less than or equal to `max_bf`
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
