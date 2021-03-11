use crate::prelude::*;

/**
    Verifies that no branching moveset is done.
**/
#[derive(Copy, Clone)]
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

impl Goal for NoBranching {
    #[inline]
    fn verify<'b>(
        &self,
        _path: &'b [Moveset],
        _game: &'b Game,
        partial_game: &'b PartialGame<'b>,
        _max_depth: Option<usize>,
    ) -> Option<bool> {
        Some(
            partial_game.info.min_timeline() == self.min_timeline
                && partial_game.info.max_timeline() == self.max_timeline,
        )
    }
}

/**
    Verifies that no branching moveset is done.
**/
#[derive(Copy, Clone)]
pub struct MaxBranching {
    pub min_timeline: Layer,
    pub max_timeline: Layer,

    pub max_branches: usize,
}

impl MaxBranching {
    pub fn new(info: &Info, max_branches: usize) -> Self {
        MaxBranching {
            min_timeline: info.min_timeline(),
            max_timeline: info.max_timeline(),

            max_branches
        }
    }
}

impl Goal for MaxBranching {
    #[inline]
    fn verify<'b>(
        &self,
        _path: &'b [Moveset],
        _game: &'b Game,
        partial_game: &'b PartialGame<'b>,
        _max_depth: Option<usize>,
    ) -> Option<bool> {
        let branches = self.min_timeline - partial_game.info.min_timeline()
            + partial_game.info.max_timeline() - self.max_timeline;
        Some(branches as usize <= self.max_branches)
    }
}
