use crate::gen::*;
use crate::prelude::*;
use crate::eval::Eval;
use super::{TreeNode, EvalNode};
use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// TODO: have some sort of tree structure to keep track of negamax

// Actors would make this *much* easier to implement
#[derive(Debug)]
pub struct TreeHandle(usize, Arc<Mutex<Option<Eval>>>, Option<usize>);

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

    let game = Arc::new(parse::parse(/* ... */));

    let tasks = Tasks::new(game, 64, Some(Duration::new(10, 0)));

    for task in tasks {
        println!("{:?}", task.path); // This line will be, if possible, reached at least 64 times
    }
    ```
**/
pub struct Tasks {
    game: Arc<Game>,

    // backlog: Vec<TreeNode<'static>>,
    pool: VecDeque<(TreeNode<'static>, Option<usize>)>,
    pool_size: usize,
    pool_yielded: usize,

    // SAFETY: `Tasks::gen` cannot be shared with any other structure that outlives this `Tasks` reference,
    // unless a reference to `Tasks::game` is also passed along
    gen: GenLegalMovesetIter<'static>,
    current_path: Vec<Moveset>,
    parent_index: Option<usize>,

    max_duration: Duration,
    sigma: Duration,

    backlog: Vec<TreeNode<'static>>,
    pub tree: Vec<TreeHandle>,
    recyclable: bool,
}

impl Tasks {
    pub fn new(
        game: Arc<Game>,
        pool_size: usize,
        max_duration: Option<Duration>,
    ) -> Self {
        let pool = VecDeque::with_capacity(pool_size);

        let root_partial_game = Cow::Owned(no_partial_game(&game));

        // pool.push_back(TreeNode {
        //     partial_game: no_partial_game(&game),
        //     path: vec![],
        // });

        unsafe {
            // SAFETY: we are extracting a &'static reference from game
            // this is safe iff Arc::strong_count(game) > 0 remains true while gen is used,
            // which is guaranteed by the stored, strong reference within self.game
            Self {
                game: Arc::clone(&game),

                pool,
                pool_size,
                pool_yielded: 0,

                gen: GenLegalMovesetIter::new(&*Arc::as_ptr(&game), root_partial_game, max_duration),
                current_path: Vec::new(),
                parent_index: None,

                max_duration: max_duration.unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1)),
                sigma: Duration::new(0, 0),

                backlog: vec![],
                tree: vec![],
                recyclable: true,
            }
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

        let game = Arc::new(parse::parse(/* ... */));

        let tasks = Tasks::new(game, 64, Some(Duration::new(10, 0)));

        while !tasks.done {
            if tasks.pool_len() == 0 {
                tasks.refill_pool();
            } else {
                println!("{:?}", tasks.next_cached().unwrap().path);
            }
        }
        ```
    **/
    pub fn refill_pool(&mut self) {
        let start = Instant::now();
        while self.pool.len() < self.pool_size {
            if self.gen.done && self.pool_yielded < self.pool_size {
                match self.pool.pop_front() {
                    Some((head, parent_index)) => {
                        self.current_path = head.path.clone();

                        if self.recyclable || parent_index.is_none() {
                            self.backlog.push(head.clone());
                        }

                        self.tree.push(TreeHandle(self.tree.len(), Arc::new(Mutex::new(None)), parent_index));
                        self.parent_index = Some(self.tree.len() - 1);

                        unsafe {
                            // SAFETY: we are extracting a &'static reference from self.game
                            // this is safe iff Arc::strong_count(self.game) > 0 remains true while gen is used,
                            // which is guaranteed by the stored, strong reference within self.game
                            self.gen = GenLegalMovesetIter::new(
                                &*Arc::as_ptr(&self.game),
                                Cow::Owned(head.partial_game),
                                Some(self.max_duration - self.sigma - start.elapsed()),
                            );
                        }
                    }
                    None => return,
                }
            } else if self.gen.done {
                return
            } else {
                match self.gen.next() {
                    Some((ms, partial_game)) => {
                        let mut path = self.current_path.clone();
                        path.push(ms);
                        // SAFETY: it is critical that partial_game.flatten() be called here, so that partial_game.parent == None
                        let partial_game = partial_game.flatten();
                        assert!(partial_game.parent.is_none());
                        let branches = partial_game.info.len_timelines() - self.game.info.len_timelines();
                        self.pool.push_back((TreeNode::new(
                            partial_game,
                            path,
                            branches
                        ), self.parent_index));
                        self.pool_yielded += 1;
                    }
                    None => {
                        if self.gen.timed_out() {
                            return
                        }
                    }
                }
            }
        }
        self.sigma += start.elapsed();
    }

    /**
        Returns the underlying reference to `Game`.
    **/
    pub fn game(&self) -> &Arc<Game> {
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
        self.gen.done && self.pool.len() == 0
        || self.gen.timed_out()
        || self.sigma >= self.max_duration
    }

    /**
        Returns the next element of the underlying pool.
        You should most likely use `Tasks::next` instead!
    **/
    pub fn next_cached(&mut self) -> Option<(TreeNode<'static>, TreeHandle)> {
        match self.pool.pop_front() {
            Some((elem, parent_index)) => {
                let handle = TreeHandle(self.tree.len(), Arc::new(Mutex::new(None)), parent_index);

                if self.recyclable || parent_index.is_none() {
                    self.backlog.push(elem.clone());
                }

                self.tree.push(TreeHandle(handle.0, Arc::clone(&handle.1), handle.2));
                Some((elem, handle))
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
                    // println!("  {:?}", self.backlog[handle.0].path);

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
    }

    pub fn best_move(&self) -> Option<(EvalNode, Eval)> {
        let mut best_move: Option<&TreeNode> = None;
        let mut best_score: Eval = f32::NEG_INFINITY;

        for handle in self.tree.iter() {
            if handle.2.is_none() {
                if let Some(value) = *handle.1.lock().unwrap() {
                    if -value > best_score {
                        best_score = -value;
                        best_move = Some(&self.backlog[handle.0]);
                    }
                }
            }
        }

        best_move.map(|bm| (bm.into(), best_score))
    }
}

impl Clone for Tasks {
    fn clone(&self) -> Tasks {
        Tasks {
            game: Arc::clone(&self.game),

            pool: self.pool.clone(),
            pool_size: self.pool_size,
            pool_yielded: self.pool_yielded,

            // SAFETY: Tasks::game is also passed along above this line
            gen: self.gen.clone(),
            current_path: self.current_path.clone(),
            parent_index: self.parent_index,

            max_duration: self.max_duration,
            sigma: self.sigma,

            backlog: self.backlog.clone(),
            tree: self.tree.iter().map(|handle| {
                // Clones a treehandle: it tries to lock the mutex and to copy the underlying data, but otherwise sets it to None
                // and wraps it all back up in a Mutex.
                TreeHandle(handle.0, Arc::new(Mutex::new(handle.1.try_lock().ok().map(|guard| *guard).flatten())), handle.2)
            }).collect(),
            recyclable: self.recyclable,
        }
    }
}

impl Iterator for Tasks {
    type Item = (TreeNode<'static>, TreeHandle);

    /**
        Returns the next tasks.
    **/
    fn next(&mut self) -> Option<Self::Item> {
        if self.pool.len() == 0 && !self.done() {
            self.refill_pool();
        }

        self.next_cached()
    }
}
