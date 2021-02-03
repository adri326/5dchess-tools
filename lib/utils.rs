use crate::prelude::moveset::FilterByStrategy;
use crate::strategies::legal::OptLegalMove;
use crate::*;
use rand::seq::SliceRandom;
use std::time::{Duration, Instant};

// Legality checker

/**
    Iterator returned by `list_legal_movesets` and `list_legal_movesets_filter_strategy`.

    Returns generated movesets and their corresponding, generated partial game, if they are legal movesets.
    A strategy can also be provided to filter the moves.
**/
#[derive(Clone)]
pub struct LegalMovesetsIter<'a, S>
where
    S: Strategy<'a, From = Move, To = bool>,
{
    pub game: &'a Game,
    pub partial_game: &'a PartialGame<'a>,
    pub moveset_iter:
        std::iter::Flatten<GenMovesetIter<'a, FilterByStrategy<'a, BoardIter<'a>, S>>>,
    pub duration: Option<Duration>,
    pub sigma: Duration,
}

pub fn list_legal_movesets_filter_strategy<'a, S>(
    game: &'a Game,
    partial_game: &'a PartialGame<'a>,
    strategy: S,
    duration: Option<Duration>,
) -> LegalMovesetsIter<'a, S>
where
    S: Strategy<'a, From = Move, To = bool>,
{
    LegalMovesetsIter {
        game,
        partial_game,
        moveset_iter: generate_movesets_filter_strategy(
            partial_game.own_boards(game).collect(),
            game,
            partial_game,
            strategy,
        )
        .flatten(),
        duration,
        sigma: Duration::new(0, 0),
    }
}

pub fn list_legal_movesets<'a>(
    game: &'a Game,
    partial_game: &'a PartialGame<'a>,
    duration: Option<Duration>,
) -> LegalMovesetsIter<'a, OptLegalMove>
where
{
    list_legal_movesets_filter_strategy::<OptLegalMove>(
        game,
        partial_game,
        OptLegalMove::new(),
        duration,
    )
}

impl<'a, S> LegalMovesetsIter<'a, S>
where
    S: Strategy<'a, From = Move, To = bool>,
{
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

impl<'a, S> Iterator for LegalMovesetsIter<'a, S>
where
    S: Strategy<'a, From = Move, To = bool>,
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

            match self.moveset_iter.next() {
                Some(Ok(ms)) => match ms.generate_partial_game(self.game, self.partial_game) {
                    Some(new_partial_game) => {
                        if !is_illegal(self.game, &new_partial_game)? {
                            break Some((ms, new_partial_game));
                        }
                    }
                    None => {}
                },
                Some(Err(_)) => {}
                None => break None,
            }
        };

        self.sigma += start.elapsed();
        res
    }
}

// Goals

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
    pub depth: usize,
}

pub fn list_legal_movesets_filter_strategy_goal<'a, 'b, S, G>(
    game: &'a Game,
    partial_game: &'a PartialGame<'a>,
    goal: &'b G,
    strategy: S,
    duration: Option<Duration>,
    depth: usize,
) -> ApplyGoals<'a, 'b, G, LegalMovesetsIter<'a, S>>
where
    'a: 'b,
    S: Strategy<'a, From = Move, To = bool>,
    G: Goal,
{
    ApplyGoals::new(
        list_legal_movesets_filter_strategy(game, partial_game, strategy, duration),
        goal,
        game,
        duration,
        depth,
    )
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
        depth: usize,
    ) -> Self {
        Self {
            iterator,
            goal,
            game,
            sigma: Duration::new(0, 0),
            duration,
            depth,
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
                    match self.goal.verify(&ms, self.game, &partial_game, self.depth) {
                        Some(true) => break Some((ms, partial_game)),
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

// Random movesets, useful for MCTS

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RandomLegalMovesetReason {
    Checkmate,
    Stalemate,
    Error,
    TimeoutCheckmate,
    TimeoutStalemate,
}

pub fn random_legal_moveset_filter_strategy<'a, S>(
    game: &'a Game,
    partial_game: &'a PartialGame<'a>,
    strategy: S,
    duration: Duration,
) -> Result<(Moveset, PartialGame<'a>), RandomLegalMovesetReason>
where
    for<'b> S: Strategy<'b, From = Move, To = bool>,
{
    let mut rng = rand::thread_rng();
    let mut active_boards: Vec<&Board> = partial_game
        .own_boards(game)
        .filter(|b| b.active(&partial_game.info))
        .collect();
    let mut inactive_boards: Vec<&Board> = partial_game
        .own_boards(game)
        .filter(|b| !b.active(&partial_game.info))
        .collect();
    active_boards.shuffle(&mut rng);
    inactive_boards.shuffle(&mut rng);
    let boards: Vec<&Board> = active_boards
        .into_iter()
        .chain(inactive_boards.into_iter())
        .collect();
    let mut iters = Vec::new();

    for board in boards {
        let mut moves: Vec<_> = FilterByStrategy::new(
            board
                .generate_moves(game, partial_game)
                .ok_or(RandomLegalMovesetReason::Error)?,
            game,
            partial_game,
            strategy.clone(),
        )
        .collect();
        moves.shuffle(&mut rng);
        iters.push(CacheMoves::new(moves.into_iter()));
    }

    let mut iter = SigmaFilter::new(
        GenMovesetIter::from_cached_iters(iters, game, partial_game)
            .filter_timed(|_iter| true, duration)
            .flatten()
            .map(|ms| {
                let res = match ms {
                    Ok(ms) => match ms.generate_partial_game(game, partial_game) {
                        Some(new_partial_game) => Some((ms, new_partial_game)),
                        None => None,
                    },
                    Err(_) => None,
                };
                res
            })
            .filter_timed(|x| x.is_some(), duration),
        |opt| {
            let (_ms, new_partial_game) = opt.as_ref().unwrap();
            !is_illegal(game, &new_partial_game).unwrap_or(true)
        },
        duration,
    );

    match iter.next() {
        Some(Some(x)) => Ok(x),
        _ => {
            if iter.timed_out() {
                match generate_idle_boards(game, partial_game) {
                    Some(idle_partial_game) => match is_in_check(game, &idle_partial_game) {
                        Some(true) => Err(RandomLegalMovesetReason::TimeoutCheckmate),
                        Some(false) => Err(RandomLegalMovesetReason::TimeoutStalemate),
                        None => Err(RandomLegalMovesetReason::Error),
                    },
                    None => Err(RandomLegalMovesetReason::Error),
                }
            } else {
                match generate_idle_boards(game, partial_game) {
                    Some(idle_partial_game) => match is_in_check(game, &idle_partial_game) {
                        Some(true) => Err(RandomLegalMovesetReason::Checkmate),
                        Some(false) => Err(RandomLegalMovesetReason::Stalemate),
                        None => Err(RandomLegalMovesetReason::Error),
                    },
                    None => Err(RandomLegalMovesetReason::Error),
                }
            }
        }
    }
}

pub fn random_legal_moveset<'a>(
    game: &'a Game,
    partial_game: &'a PartialGame<'a>,
    duration: Option<Duration>,
) -> Result<(Moveset, PartialGame<'a>), RandomLegalMovesetReason>
where
{
    random_legal_moveset_filter_strategy(
        game,
        partial_game,
        OptLegalMove::new(),
        duration.unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1)),
    )
}
