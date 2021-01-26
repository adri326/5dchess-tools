use crate::prelude::*;

/**
    Verifies that no branching moveset is done. You can further optimize the search by also giving it as strategy `NoTimeTravel`.
**/
pub struct NoBranching {
    pub min_timeline: Layer,
    pub max_timeline: Layer,
}

impl NoBranching {
    pub fn new(info: &Info) -> Self {
        NoBranching {
            min_timeline: info.min_timeline(),
            max_timeline: info.max_timeline(),
        }
    }
}

impl<B: Clone + AsRef<Board>> Goal<B> for NoBranching {
    fn verify<'b>(
        &self,
        _moveset: &'b Moveset,
        _game: &'b Game,
        partial_game: &'b PartialGame<'b, B>,
        _depth: usize,
    ) -> Option<bool> {
        Some(
            partial_game.info.min_timeline() == self.min_timeline
                && partial_game.info.max_timeline() == self.max_timeline,
        )
    }
}
