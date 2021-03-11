use crate::gen::*;
use crate::prelude::*;
use crate::eval::Eval;
use crate::check::is_in_check;
use super::{TreeNode, EvalNode};
use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// TODO: have some sort of tree structure to keep track of negamax

// Actors would make this *much* easier to implement
#[derive(Debug)]
pub struct TreeHandle(usize, Arc<Mutex<Option<Eval>>>, Option<usize>); // handle index, handle value, parent handle index

impl TreeHandle {
    pub fn report(&self, value: Eval) {
        let mut guard = self.1.lock().unwrap();
        *guard = Some(value);
        // drop guard
    }

    pub fn score(&self) -> Option<Eval> {
        let guard = self.1.lock().ok()?;
        *guard
    }

    pub fn index(&self) -> usize {
        self.0
    }

    pub fn parent(&self) -> Option<usize> {
        self.2
    }
}

/**
    An iterator that splits a given position into a set of "tasks", that you can then use to distribute your tree search across multiple threads.

    ## Example

    ```
    use chess5dlib::parse;
    use chess5dlib::prelude::*;
    use chess5dlib::tree::*;
    use std::time::Duration;

    let game = parse::parse(/* ... */);

    let tasks = Tasks::new(&game, 64, Some(Duration::new(10, 0)));

    for task in tasks {
        println!("{:?}", task.path); // This line will be, if possible, reached at least 64 times
    }
    ```
**/
#[derive(Debug)]
pub struct Tasks<'a> {
    game: &'a Game,

    roots: Vec<TreeNode<'static>>,
    pool: VecDeque<(TreeNode<'static>, usize)>, // Node and index in the tree
    pool_size: usize,
    max_pool_size: usize,
    pool_yielded: usize,

    max_duration: Duration,
    sigma: Duration,

    pub tree: Vec<TreeHandle>,
    recyclable: bool,
    index: usize,
}

impl<'a> Tasks<'a> {
    pub fn new(
        game: &'a Game,
        pool_size: usize,
        max_pool_size: usize,
        max_duration: Option<Duration>,
    ) -> Self {
        let mut pool = VecDeque::with_capacity(pool_size);

        pool.push_back((
            TreeNode::empty(game),
            0
        ));

        let tree = vec![TreeHandle(0, Arc::new(Mutex::new(None)), None)];

        Self {
            game,

            roots: vec![],
            pool,
            pool_size,
            max_pool_size,
            pool_yielded: 0,

            max_duration: max_duration.unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1)),
            sigma: Duration::new(0, 0),

            tree,
            recyclable: true,
            index: 0,
        }
    }

    /**
        Grows the underlying "task pool", using a lazy breadth-first search algorithm.
        Calling this function and consuming the iterator until `Tasks::done()` is true is guaranteed to cover all of the children
        from the root node.

        Alternatively, you can call `Tasks::next`.

        ## Example

        Following is a simple example that tries to print the tasks yielded by `Tasks`:

        ```
        use chess5dlib::parse;
        use chess5dlib::prelude::*;
        use chess5dlib::tree::*;
        use std::time::Duration;

        let game = parse::parse(/* ... */);

        let tasks = Tasks::new(&game, 64, Some(Duration::new(10, 0)));
        if !tasks.fill_pool() {
            panic!("Couldn't fill the pool!");
        }

        while !tasks.done {
            println!("{:?}", tasks.next_cached().unwrap().path);
        }
        ```

        ## Note

        Calling this function on a consumed, non-`recyclable` instance will do nothing and the instance will remain consumed.
    **/
    pub fn fill_pool(&mut self, max_depth: usize) -> bool {
        let start = Instant::now();

        if self.pool.len() < self.pool_size && self.pool.len() != 0 {
            let mut attempts: usize = 0;
            let (base_node, mut parent_index) = loop {
                let x = self.pool.pop_front().unwrap();
                if x.0.path.len() <= max_depth {
                    break x
                } else {
                    self.pool.push_back(x);
                    attempts += 1;
                    if attempts > self.pool.len() {
                        return true
                    }
                }
            };
            let mut current_path = base_node.path;
            let mut current_partial_game = base_node.partial_game.clone();
            let mut gen = GenLegalMovesetIter::new(
                self.game,
                Cow::Owned(base_node.partial_game),
                Some(self.max_duration - self.sigma),
            );
            loop {
                // Add all of the items in gen into the pool
                let mut yielded = false;
                for (ms, partial_game) in &mut gen {
                    yielded = true;
                    if self.sigma + start.elapsed() > self.max_duration {
                        self.sigma += start.elapsed();
                        return false
                    }
                    let mut path = current_path.clone();
                    path.push(ms);
                    let partial_game = partial_game.flatten();
                    let branches = partial_game.info.len_timelines() - self.game.info.len_timelines();

                    // Add entry in the tree
                    self.tree.push(TreeHandle(self.tree.len(), Arc::new(Mutex::new(None)), Some(parent_index)));

                    // Add entry to the roots
                    if parent_index == 0 {
                        self.roots.push(TreeNode::new(
                            partial_game.clone(),
                            path.clone(),
                            branches,
                        ));
                    }

                    // Add entry in the pool
                    self.pool.push_back((TreeNode::new(
                        partial_game,
                        path,
                        branches
                    ), self.tree.len() - 1));

                    // If the pool is at its max size
                    if self.pool.len() > self.max_pool_size {
                        self.sigma += start.elapsed();
                        return false
                    }
                }

                if !yielded && !gen.timed_out() {
                    match is_in_check(self.game, &current_partial_game) {
                        Some((true, _)) => *self.tree[parent_index].1.lock().unwrap() = Some(Eval::NEG_INFINITY),
                        Some((false, _)) => *self.tree[parent_index].1.lock().unwrap() = Some(0.0),
                        None => return false
                    }
                }

                if self.sigma + start.elapsed() > self.max_duration {
                    self.sigma += start.elapsed();
                    return false
                }
                // Regenerate gen
                if self.pool.len() < self.pool_size && self.pool.len() != 0 {
                    let mut attempts: usize = 0;
                    let elem = loop {
                        let x = self.pool.pop_front().unwrap();
                        if x.0.path.len() <= max_depth {
                            break x
                        } else {
                            self.pool.push_back(x);
                            attempts += 1;
                            if attempts > self.pool.len() {
                                return true
                            }
                        }
                    };
                    let base_node = elem.0;
                    // println!("-> {:?}", base_node.path);
                    parent_index = elem.1;
                    current_path = base_node.path;
                    current_partial_game = base_node.partial_game.clone();
                    gen = GenLegalMovesetIter::new(
                        self.game,
                        Cow::Owned(base_node.partial_game),
                        Some(self.max_duration - self.sigma - start.elapsed()),
                    );
                } else {
                    break
                }
            }
        }

        self.sigma += start.elapsed();
        true
    }

    /**
        Returns the underlying reference to `Game`.
    **/
    pub fn game(&self) -> &Game {
        &self.game
    }

    /**
        Returns the current length of the pool.
    **/
    pub fn pool_len(&self) -> usize {
        self.pool.len()
    }

    /**
        Returns the desired pool size.
    **/
    pub fn pool_size(&self) -> usize {
        self.pool_size
    }

    /**
        Returns whether or not the task iterator is done handing out tasks.
        The task iterator is considered to be done once (or):
        - the current, underlying iterator is done and the pool is empty
        - the current, underlying iterator timed out
        - the scheduler timed out
    **/
    pub fn done(&self) -> bool {
        self.sigma >= self.max_duration
    }

    /**
        Returns the n-th element of the underlying pool.
    **/
    fn get_cached(&mut self, index: usize) -> Option<(TreeNode<'static>, TreeHandle)> {
        match self.pool.get(index) {
            Some((elem, handle_index)) => {
                let handle = &self.tree[*handle_index];

                Some((elem.clone(), TreeHandle(handle.0, Arc::clone(&handle.1), handle.2)))
            }
            None => None
        }
    }

    fn pop_cached(&mut self) -> Option<(TreeNode<'static>, TreeHandle)> {
        match self.pool.pop_front() {
            Some((elem, handle_index)) => {
                let handle = &self.tree[handle_index];

                Some((elem, TreeHandle(handle.0, Arc::clone(&handle.1), handle.2)))
            }
            None => None
        }
    }

    pub fn update_tree(&mut self) {
        // TODO: cache the guard of the parent entries
        for handle in self.tree.iter().rev() {
            match handle.2 {
                Some(parent) => {
                    debug_assert_ne!(parent, handle.0);
                    let parent_handle = &self.tree[parent];
                    let value = handle.1.lock().unwrap();
                    let mut parent_value = parent_handle.1.lock().unwrap();

                    // println!("{:?} {:?} {:?}", parent, *value, *parent_value);

                    match (*value, *parent_value) {
                        (Some(x), Some(y)) => {
                            if -x > y {
                                *parent_value = Some(-x);
                            }
                        }
                        (Some(x), None) => {
                            *parent_value = Some(-x);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // for _handle in &self.tree {
        //     println!("=> {:?}", _handle);
        // }

        // for task in &self.pool {
        //     if self.tree[task.1].1.lock().unwrap().unwrap().is_finite() {
        //         println!("{:?} {:?}", task.0.path, self.tree[task.1]);
        //         if self.tree[task.1].2 == Some(2) {
        //             println!("{:#?}", task);
        //         }
        //     }
        // }
    }

    pub fn best_move(&self) -> Option<(EvalNode, Eval)> {
        let mut best_move: Option<&TreeNode> = None;
        let mut best_score: Option<Eval> = None;

        for handle in self.tree.iter() {
            if handle.2 == Some(0) {
                if let Some(value) = *handle.1.lock().unwrap() {
                    match best_score {
                        Some(b) if -value > b => {
                            best_score = Some(-value);
                            best_move = Some(&self.roots[handle.0 - 1]); // The -1 is here because the first element of the tree is always the base node
                        }
                        None => {
                            best_score = Some(-value);
                            best_move = Some(&self.roots[handle.0 - 1]);
                        }
                        _ => {}
                    }
                }
            }
        }

        best_move.map(|bm| (bm.into(), best_score.unwrap()))
    }

    pub fn root_eval(&self) -> Option<Eval> {
        self.tree[0].1.try_lock().ok().map(|x| *x).flatten()
    }

    pub fn reset(&mut self, prune: bool, prune_empty: bool, depth: usize) {
        if self.recyclable {
            self.index = 0;
            if prune {
                for _ in 0..self.pool.len() {
                    if let Some((node, handle_index)) = self.pool.pop_front() {
                        let handle = &self.tree[handle_index];
                        let keep = if let Ok(guard) = handle.1.try_lock() {
                            match *guard {
                                Some(value) => value.is_finite(),
                                None => !prune_empty,
                            }
                        } else {
                            // Couldn't lock the mutex, so retain the value
                            true
                        };
                        if keep {
                            self.pool.push_back((node, handle_index))
                        } else {
                            // println!("<- {:?}: {:?}", *handle.1.lock().unwrap(), node.path);
                        }
                    }
                }
                for handle in &self.tree {
                    if let Ok(mut guard) = handle.1.try_lock() {
                        if guard.map(|x| x.is_finite()).unwrap_or(false) {
                            *guard = None;
                        }
                    }
                }
                self.fill_pool(depth);
            }
        }
    }
}

impl<'a> Clone for Tasks<'a> {
    fn clone(&self) -> Tasks<'a> {
        Tasks {
            game: self.game,

            roots: self.roots.clone(),
            pool: self.pool.clone(),
            pool_size: self.pool_size,
            max_pool_size: self.max_pool_size,
            pool_yielded: self.pool_yielded,

            max_duration: self.max_duration,
            sigma: self.sigma,

            tree: self.tree.iter().map(|handle| {
                // Clones a treehandle: it tries to lock the mutex and to copy the underlying data, but otherwise sets it to None
                // and wraps it all back up in a Mutex.
                TreeHandle(handle.0, Arc::new(Mutex::new(handle.1.try_lock().ok().map(|guard| *guard).flatten())), handle.2)
            }).collect(),
            recyclable: self.recyclable,
            index: self.index,
        }
    }
}

impl<'a> Iterator for Tasks<'a> {
    type Item = (TreeNode<'static>, TreeHandle);

    /**
        Returns the next tasks.
    **/
    fn next(&mut self) -> Option<Self::Item> {
        if self.sigma >= self.max_duration {
            return None
        }

        if self.recyclable {
            let res = self.get_cached(self.index);
            self.index += 1;
            res
        } else {
            self.pop_cached()
        }
    }
}
