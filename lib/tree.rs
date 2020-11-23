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
            let results = Arc::new(Mutex::new(Vec::new()));
            for initial_node in
                legal_movesets(game, &game.info, &initial_virtual_boards, 0, 0).take(initial_movesets)
            {
                let results = Arc::clone(&results);
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
                    results.lock().unwrap().push((initial_node, res));
                });
            }
            scope.join_all();
            let res = results.lock().unwrap().clone();
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
    pub struct BFSBranch {
        pub boards: Vec<Board>,
        pub moves: Vec<Vec<Move>>,
        pub info: GameInfo,
        pub depth: usize,
        pub score: f32,
        pub tree: RBFSTree,
    }

    impl From<(Node, &BFSBranch, RBFSTree)> for BFSBranch {
        fn from(raw: (Node, &BFSBranch, RBFSTree)) -> Self {
            BFSBranch {
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

    impl Into<Node> for BFSBranch {
        fn into(self) -> Node {
            (self.moves.last().unwrap_or(&vec![]).clone(), self.boards, self.info, self.score)
        }
    }

    impl Into<Node> for &BFSBranch {
        fn into(self) -> Node {
            (self.moves.last().unwrap_or(&vec![]).clone(), self.boards.clone(), self.info.clone(), self.score)
        }
    }

    pub type RBFSTree = Rc<RefCell<BFSTree>>;

    /**
        Tree built alongside the different `BFSBranch`es to keep track of which branch needs to be pruned.
    **/
    #[derive(Debug)]
    pub struct BFSTree {
        pub depth: usize,
        pub white: bool,
        pub children: Vec<RBFSTree>,
        pub score: f32,
        pub pruned: bool,
    }

    impl BFSTree {
        /// Creates a new BFSTree instance, as a child from `node`
        pub fn after(node: &RBFSTree, score: f32) -> Option<RBFSTree> {
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
        let mut pool: VecDeque<BFSBranch> = VecDeque::with_capacity(pool_size * 2);
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
        pool.push_back(BFSBranch {
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
                                    pool.push_back(BFSBranch::from((node, &branch, new_tree)));
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
    fn bfs_prune(pool: &mut VecDeque<BFSBranch>, initial_tree: RBFSTree, tolerance: f32) {
        bfs_recalculate_tree(&initial_tree);
        bfs_prune_rec_2(&initial_tree, false, tolerance);
        for _ in 0..pool.len() {
            let node = pool.pop_front().unwrap();
            if !node.tree.borrow().pruned {
                pool.push_back(node);
            }
        }
    }

    /// First step of the pruning: re-calculate the score of each branch
    pub fn bfs_recalculate_tree(tree: &RBFSTree) {
        if tree.borrow().children.len() == 0 {
            return;
        }
        let children = tree
            .borrow()
            .children
            .clone()
            .into_iter()
            .inspect(bfs_recalculate_tree);
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

pub mod iddfs {
    use super::*;
    use super::bfs::*;

    /** Iterative deepening depth-first search with initial breadth-first search.

    A set of initial nodes is first generated using a standard BFS algorithm, until the queue reaches the desired amount of IDDFS jobs (`pool_size`).

    From then on, DFS searches are scheduled on each of these nodes with increasing depth until the time runs out.

    See the documentation of `bfs::bfs` and `dfs::dfs` for more details.

    - `game` is the game instance to look moves on
    - `max_ms` is the maximum number of movesets to consider until the position is deemed to be draw or checkmate
    - `bucket_size` is the maximum number of movesets to process
    - `pool_size` is the desired number of tasks to have. The actual number of tasks might exceed that number and is subject to change should some lines be worse than others. Tasks will be properly scheduled among the different threads
    - `n_threads` is the number of threads to run this with
    - `max_duration` is the maximum duration that this algorithm may take; once that maximum duration is reached, the process is stopped as soon as possible and early results, if available, are returned
    **/
    pub fn iddfs_bfs<'a>(
        game: &'a Game,
        max_ms: usize,
        bucket_size: usize,
        pool_size: usize,
        n_threads: u32,
        max_duration: Duration,
    ) -> Option<(Node, f32)> {
        let queue_fail_threshold = 4;
        let begin = Instant::now();
        let mut queue: VecDeque<BFSBranch> = VecDeque::new();
        let root = Rc::new(RefCell::new(BFSTree {
            depth: 0,
            children: vec![],
            score: 0.0,
            white: !game.info.active_player,
            pruned: false,
        }));
        queue.push_back(BFSBranch {
            moves: vec![],
            boards: vec![],
            info: game.info.clone(),
            depth: 0,
            score: 0.0,
            tree: root.clone(),
        });
        let mut depth = 0;
        let mut pool = Pool::new(n_threads);
        'deepening_loop: while begin.elapsed() < max_duration {
            depth += 1;

            // Fill up the queue; break after enough successive "fails" (leaves) were encountered
            let mut queue_fails = 0;
            while queue.len() < pool_size {
                if let Some(mut branch) = queue.pop_front() {
                    let virtual_boards = branch.boards.iter().collect::<Vec<_>>();
                    let mut has_looped = false;
                    for moveset in legal_movesets(game, &branch.info, &virtual_boards, 0, max_ms)
                        .take(bucket_size)
                    {
                        has_looped = true;
                        let new_tree = BFSTree::after(&branch.tree, moveset.3).unwrap();
                        queue.push_back(BFSBranch::from((moveset, &branch, new_tree)));
                    }
                    if !has_looped {
                        if is_draw(game, &branch.boards.iter().collect(), &branch.info) {
                            branch.score = 0.0;
                            branch.tree.borrow_mut().score = 0.0;
                        } else {
                            if branch.info.active_player {
                                branch.score = std::f32::NEG_INFINITY;
                                branch.tree.borrow_mut().score = std::f32::NEG_INFINITY;
                            } else {
                                branch.score = std::f32::INFINITY;
                                branch.tree.borrow_mut().score = std::f32::INFINITY;
                            }
                        }
                        queue_fails += 1;
                        queue.push_back(branch);
                    } else {
                        queue_fails = 0;
                    }
                }
                // println!("> {}", queue.len());
                if queue.len() == 0 {
                    panic!("Queue got emptied!");
                }
                if queue_fails >= queue_fail_threshold || queue_fails >= queue.len() {
                    break;
                }
            }

            let iddfs_res = pool.scoped(|scope| {
                let results: Arc<Mutex<Vec<(usize, Option<(Vec<Node>, f32)>)>>> = Arc::new(Mutex::new(Vec::new()));
                // The queue iterator is reversed as to process the shallower (and usually slower) nodes first
                for (id, node) in queue.iter().enumerate().rev() {
                    if node.depth <= depth {
                        let depth = depth - node.depth;
                        let results = Arc::clone(&results);
                        let node: Node = node.into();
                        scope.execute(move || {
                            let res = iddfs_bfs_sub(
                                game,
                                &vec![],
                                node,
                                max_ms,
                                bucket_size,
                                depth,
                                std::f32::NEG_INFINITY,
                                std::f32::INFINITY,
                                begin,
                                max_duration,
                            );
                            results.lock().unwrap().push((id, res));
                        });
                    }
                }
                scope.join_all();
                // move out of scope guard
                let res = results.lock().unwrap().clone();
                res
            });

            for result in iddfs_res.clone().into_iter() {
                if let (id, Some((_nodes, score))) = result {
                    queue[id].score = score;
                    queue[id].tree.borrow_mut().score = score;
                } else {
                    break 'deepening_loop;
                }
            }

            bfs_recalculate_tree(&root);
            bfs_prune_infinities(&root, false);
            let mut pruned = 0;

            for _ in 0..queue.len() {
                let node = queue.pop_front().unwrap();
                if !node.tree.borrow().pruned {
                    queue.push_back(node);
                } else {
                    pruned += 1;
                }
            }

            if root.borrow().score.is_infinite() {
                break;
            }

            println!("Depth: {}, pruned: {}, queue: {}, score: {}", depth, pruned, queue.len(), root.borrow().score);
            // println!("{:#?}", root);
            // println!("{:#?}", iddfs_res.iter().map(|(i, o)| o.as_ref().map(|(n, v)| (i, n.iter().map(|x| x.0.clone()).collect::<Vec<_>>(), v))).collect::<Vec<_>>());
        }

        bfs_recalculate_tree(&root);
        bfs_keep_best(&root, false);
        for candidate in queue.into_iter() {
            if !candidate.tree.borrow().pruned {
                let score = candidate.score;
                return Some((candidate.into(), score));
            }
        }
        None
    }

    /// Recursive DFS search with time verification
    fn iddfs_bfs_sub<'a>(
        game: &'a Game,
        virtual_boards: &Vec<&Board>,
        node: Node,
        max_ms: usize,
        bucket_size: usize,
        depth: usize,
        mut alpha: f32,
        mut beta: f32,
        begin: Instant,
        max_duration: Duration,
    ) -> Option<(Vec<Node>, f32)> {
        if begin.elapsed() >= max_duration {
            return None;
        } else if depth == 0 {
            Some((vec![node.clone()], node.3))
        } else {
            let merged_vboards: Vec<&Board> = virtual_boards
                .iter()
                .map(|x| *x)
                .chain(node.1.iter())
                .collect::<Vec<&Board>>();
            let mut best = (vec![], if node.2.active_player {std::f32::NEG_INFINITY} else {std::f32::INFINITY});
            // Loop over the child nodes
            for moveset in legal_movesets(game, &node.2, &merged_vboards, 0, max_ms).take(bucket_size) {
                let res = iddfs_bfs_sub(
                    game,
                    &merged_vboards,
                    moveset.clone(),
                    max_ms,
                    bucket_size,
                    depth - 1,
                    alpha,
                    beta,
                    begin,
                    max_duration
                );

                if let None = res {
                    return None;
                } else if let Some(res) = res {
                    if best.0.len() == 0 {
                        best = res;
                    } else if node.2.active_player {
                        if res.1 > best.1 {
                            alpha = res.1;
                            best = res;
                        }
                    } else {
                        if res.1 < best.1 {
                            beta = res.1;
                            best = res;
                        }
                    }
                }

                if alpha >= beta {
                    break;
                }
            }

            if best.0.len() != 0 {
                let mut v = vec![node];
                v.append(&mut best.0);
                Some((v, best.1))
            } else {
                if is_draw(game, &merged_vboards, &node.2) {
                    Some((vec![node.clone()], 0.0))
                } else {
                    Some((vec![node.clone()], if node.2.active_player {std::f32::NEG_INFINITY} else {std::f32::INFINITY}))
                }
            }
        }
    }

    /// Removes branches that are marked as loosing from the tree
    fn bfs_prune_infinities(tree: &RBFSTree, prune: bool) {
        let score = tree.borrow().score;

        if prune {
            tree.borrow_mut().pruned = true;
            for c in tree.borrow().children.iter() {
                bfs_prune_infinities(c, true);
            }
            if tree.borrow().children.len() > 0 {
                tree.borrow_mut().children = Vec::new();
            }
        }

        if score.is_finite() {
            for c in tree.borrow().children.iter() {
                let should_prune = c.borrow().score.is_infinite();
                bfs_prune_infinities(c, should_prune);
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

    /// Only keep the best node in the tree
    fn bfs_keep_best(tree: &RBFSTree, prune: bool) {
        let score = tree.borrow().score;

        if prune {
            tree.borrow_mut().pruned = true;
            for c in tree.borrow().children.iter() {
                bfs_prune_infinities(c, true);
            }
            if tree.borrow().children.len() > 0 {
                tree.borrow_mut().children = Vec::new();
            }
        }

        for c in tree.borrow().children.iter() {
            let should_prune = c.borrow().score != score;
            bfs_keep_best(c, should_prune);
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
