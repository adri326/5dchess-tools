use crate::gen::*;
use crate::prelude::*;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct TreeNode {
    pub partial_game: PartialGame<'static>,
    pub path: Vec<Moveset>,
}

pub struct Tasks<'a> {
    pub game: &'a Game,
    pub partial_game: &'a PartialGame<'a>,

    pool: VecDeque<TreeNode>,
    pool_size: usize,

    gen: GenLegalMovesetIter<'a>,
    current_path: Vec<Moveset>,

    max_duration: Duration,
    start: Instant,
}

impl<'a> Tasks<'a> {
    pub fn new(
        game: &'a Game,
        partial_game: &'a PartialGame<'a>,
        pool_size: usize,
        max_duration: Option<Duration>,
    ) -> Self {
        let mut pool: VecDeque<TreeNode> = VecDeque::with_capacity(pool_size);

        pool.push_back(TreeNode {
            partial_game: no_partial_game(game),
            path: vec![],
        });

        Self {
            game,
            partial_game,

            pool,
            pool_size,

            gen: GenLegalMovesetIter::new(game, Cow::Borrowed(partial_game), max_duration),
            current_path: Vec::new(),

            max_duration: max_duration.unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1)),
            start: Instant::now(),
        }
    }

    pub fn refill_pool(&mut self) {
        while self.pool.len() < self.pool_size {
            if self.gen.done {
                match self.pool.pop_front() {
                    Some(head) => {
                        self.gen = GenLegalMovesetIter::new(
                            self.game,
                            Cow::Owned(head.partial_game),
                            Some(self.max_duration - self.start.elapsed()),
                        );
                    }
                    None => return,
                }
            } else {
                match self.gen.next() {
                    Some((ms, partial_game)) => {
                        let mut path = self.current_path.clone();
                        path.push(ms);
                        self.pool.push_back(TreeNode {
                            partial_game: partial_game.flatten(),
                            path,
                        })
                    }
                    None => {
                        if self.gen.timed_out() || self.start.elapsed() > self.max_duration {
                            return;
                        }
                    }
                }
            }
        }
    }
}
