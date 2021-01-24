use crate::prelude::*;

/**
    Verifies that no branching moveset is done until `depth > max_depth`. If `max_depth` is `0`, then the condition holds indefinitely.
    If `max_depth == 0`, then you can further optimize the search by also giving it as strategy `NoTimeTravel`.
**/
pub struct NoBranching {
    pub min_timeline: Layer,
    pub max_timeline: Layer,
    pub max_depth: usize,
}

impl NoBranching {
    pub fn new(info: &Info, max_depth: usize) -> Self {
        NoBranching {
            min_timeline: info.min_timeline(),
            max_timeline: info.max_timeline(),
            max_depth,
        }
    }
}

impl<B: Clone + AsRef<Board>> Goal<B> for NoBranching {
    fn verify<'b>(
        &self,
        _moveset: &'b Moveset,
        _game: &'b Game,
        partial_game: &'b PartialGame<'b, B>,
        depth: usize
    ) -> Option<bool> {
        if depth > self.max_depth {
            Some(true)
        } else {
            Some(
                partial_game.info.min_timeline() == self.min_timeline
                && partial_game.info.max_timeline() == self.max_timeline
            )
        }
    }
}
