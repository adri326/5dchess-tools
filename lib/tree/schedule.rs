use crate::gen::*;
use crate::prelude::*;
use super::TreeNode;
use std::borrow::Cow;
use std::sync::Arc;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

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
#[derive(Clone)]
pub struct Tasks {
    game: Arc<Game>,

    pool: VecDeque<TreeNode<'static>>,
    pool_size: usize,
    pool_yielded: usize,

    // SAFETY: `Tasks::gen` cannot be shared with any other structure that outlives this `Tasks` reference,
    // unless a reference to `Tasks::game` is also passed along
    gen: GenLegalMovesetIter<'static>,
    current_path: Vec<Moveset>,

    max_duration: Duration,
    sigma: Duration,
}

impl Tasks {
    pub fn new(
        game: Arc<Game>,
        pool_size: usize,
        max_duration: Option<Duration>,
    ) -> Self {
        let mut pool: VecDeque<TreeNode<'static>> = VecDeque::with_capacity(pool_size);

        let root_partial_game = Cow::Owned(no_partial_game(&game));

        pool.push_back(TreeNode {
            partial_game: no_partial_game(&game),
            path: vec![],
        });

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

                max_duration: max_duration.unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1)),
                sigma: Duration::new(0, 0),
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
                    Some(head) => {
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
                        self.pool.push_back(TreeNode {
                            partial_game,
                            path,
                        });
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
    pub fn next_cached(&mut self) -> Option<TreeNode<'static>> {
        self.pool.pop_front()
    }
}

impl Iterator for Tasks {
    type Item = TreeNode<'static>;

    /**
        Returns the next tasks.
    **/
    fn next(&mut self) -> Option<Self::Item> {
        if self.pool.len() == 0 && !self.done() {
            self.refill_pool();
        }

        self.pool.pop_front()
    }
}
