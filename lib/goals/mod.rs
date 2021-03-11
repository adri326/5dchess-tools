use crate::prelude::*;
use std::time::{Instant, Duration};

/*? A set of non-essential goals that you may use to filter the tree searches.
Following is a list of the submodules and what they include:

- [`misc`](./misc.rs): Contains miscellaneous goals that do not fit in the other categories.
*/

pub mod branch;

#[derive(Clone)]
pub struct ApplyGoals<'a, 'b, G, I>
where
    'a: 'b,
    G: Goal,
    I: Iterator<Item = (Moveset, PartialGame<'a>)>,
{
    pub iterator: I,
    pub goal: &'b G,
    pub game: &'a Game,
    pub sigma: Duration,
    pub duration: Option<Duration>,
    pub max_depth: Option<usize>,
}

impl<'a, 'b, G, I> ApplyGoals<'a, 'b, G, I>
where
    'a: 'b,
    G: Goal,
    I: Iterator<Item = (Moveset, PartialGame<'a>)>,
{
    pub fn new(
        iterator: I,
        goal: &'b G,
        game: &'a Game,
        duration: Option<Duration>,
        max_depth: Option<usize>,
    ) -> Self {
        Self {
            iterator,
            goal,
            game,
            sigma: Duration::new(0, 0),
            duration,
            max_depth,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.sigma
    }

    pub fn remaining(&self) -> Option<Duration> {
        match self.duration {
            Some(duration) => {
                if self.sigma > duration {
                    Some(Duration::new(0, 0))
                } else {
                    Some(duration - self.sigma)
                }
            }
            None => None,
        }
    }

    pub fn timed_out(&self) -> bool {
        match self.duration {
            Some(duration) => self.sigma > duration,
            None => false,
        }
    }
}

impl<'a, 'b, G, I> Iterator for ApplyGoals<'a, 'b, G, I>
where
    'a: 'b,
    G: Goal,
    I: Iterator<Item = (Moveset, PartialGame<'a>)>,
{
    type Item = (Moveset, PartialGame<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let start = Instant::now();

        let res = loop {
            match self.duration {
                Some(duration) => {
                    if self.sigma + start.elapsed() > duration {
                        break None;
                    }
                }
                None => {}
            }

            match self.iterator.next() {
                Some((ms, partial_game)) => {
                    let mut arr = vec![ms];
                    match self.goal.verify(&arr, self.game, &partial_game, self.max_depth) {
                        Some(true) => {
                            let ms = arr.pop().unwrap();
                            break Some((ms, partial_game))
                        }
                        Some(false) => {}
                        None => break None,
                    }
                }
                None => break None,
            }
        };

        self.sigma += start.elapsed();
        res
    }
}
