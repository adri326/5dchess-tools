use crate::prelude::moveset::FilterByStrategy;
use crate::strategies::legal::OptLegalMove;
use crate::*;
use std::time::{Duration, Instant};

// Legality checker

/**
    Iterator returned by `list_legal_movesets` and `list_legal_movesets_filter_strategy`.

    Returns generated movesets and their corresponding, generated partial game, if they are legal movesets.
    A strategy can also be provided to filter the moves.
**/
pub struct LegalMovesetsIter<'a, S, B>
where
    S: Strategy<'a, B, From = Move, To = bool>,
    B: Clone + AsRef<Board> + AsMut<Board> + 'a,
    for<'b> &'b B: GenMoves<'b, B>,
    for<'b> B: From<(Board, &'b Game, &'b PartialGame<'b, B>)>,
{
    pub game: &'a Game,
    pub partial_game: &'a PartialGame<'a, B>,
    pub moveset_iter: std::iter::Flatten<GenMovesetIter<
        'a,
        B,
        FilterByStrategy<'a, B, <BoardOr<'a, B> as GenMoves<'a, B>>::Iter, S>,
    >>,
    pub duration: Option<Duration>,
    pub sigma: Duration,
    _phantom: std::marker::PhantomData<*const S>,
}

pub fn list_legal_movesets_filter_strategy<'a, S, B>(
    game: &'a Game,
    partial_game: &'a PartialGame<'a, B>,
    duration: Option<Duration>,
) -> LegalMovesetsIter<'a, S, B>
where
    S: Strategy<'a, B, From = Move, To = bool>,
    B: Clone + AsRef<Board> + AsMut<Board> + 'a,
    for<'b> &'b B: GenMoves<'b, B>,
    for<'b> B: From<(Board, &'b Game, &'b PartialGame<'b, B>)>,
{
    LegalMovesetsIter {
        game,
        partial_game,
        moveset_iter: generate_movesets_filter_strategy(
            partial_game.own_boards(game).collect(),
            game,
            partial_game,
        ).flatten(),
        duration,
        sigma: Duration::new(0, 0),
        _phantom: std::marker::PhantomData,
    }
}

pub fn list_legal_movesets<'a, B>(
    game: &'a Game,
    partial_game: &'a PartialGame<'a, B>,
    duration: Option<Duration>,
) -> LegalMovesetsIter<'a, OptLegalMove, B>
where
    B: Clone + AsRef<Board> + AsMut<Board> + 'a,
    for<'b> &'b B: GenMoves<'b, B>,
    for<'b> B: From<(Board, &'b Game, &'b PartialGame<'b, B>)>,
{
    list_legal_movesets_filter_strategy::<OptLegalMove, B>(
        game,
        partial_game,
        duration
    )
}

impl<'a, S, B> LegalMovesetsIter<'a, S, B>
where
    S: Strategy<'a, B, From = Move, To = bool>,
    B: Clone + AsRef<Board> + AsMut<Board> + 'a,
    for<'b> &'b B: GenMoves<'b, B>,
    for<'b> B: From<(Board, &'b Game, &'b PartialGame<'b, B>)>,
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
            None => None
        }
    }

    pub fn timed_out(&self) -> bool {
        match self.duration {
            Some(duration) => self.sigma > duration,
            None => false,
        }
    }
}

impl<'a, S, B> Iterator for LegalMovesetsIter<'a, S, B>
where
    S: Strategy<'a, B, From = Move, To = bool>,
    B: Clone + AsRef<Board> + AsMut<Board> + 'a,
    for<'b> &'b B: GenMoves<'b, B>,
    for<'b> B: From<(Board, &'b Game, &'b PartialGame<'b, B>)>,
{
    type Item = (Moveset, PartialGame<'a, B>);

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
                Some(Ok(ms)) => {
                    match ms.generate_partial_game(self.game, self.partial_game) {
                        Some(new_partial_game) => {
                            if !is_illegal(self.game, &new_partial_game)? {
                                break Some((ms, new_partial_game))
                            }
                        },
                        None => {}
                    }
                }
                Some(Err(_)) => {},
                None => break None,
            }
        };

        self.sigma += start.elapsed();
        res
    }
}


// Goals

pub struct ApplyGoals<'a, 'b, B, G, I>
where
    'a: 'b,
    B: Clone + AsRef<Board> + 'a,
    G: Goal<B>,
    I: Iterator<Item=(Moveset, PartialGame<'a, B>)>,
{
    pub iterator: I,
    pub goal: &'b G,
    pub game: &'a Game,
    pub sigma: Duration,
    pub duration: Option<Duration>,
    pub depth: usize,
}

pub fn list_legal_movesets_filter_strategy_goal<'a, 'b, S, G, B>(
    game: &'a Game,
    partial_game: &'a PartialGame<'a, B>,
    goal: &'b G,
    duration: Option<Duration>,
    depth: usize,
) -> ApplyGoals<'a, 'b, B, G, LegalMovesetsIter<'a, S, B>>
where
    'a: 'b,
    S: Strategy<'a, B, From = Move, To = bool>,
    G: Goal<B>,
    B: Clone + AsRef<Board> + AsMut<Board> + 'a,
    for<'c> &'c B: GenMoves<'c, B>,
    for<'c> B: From<(Board, &'c Game, &'c PartialGame<'c, B>)>,
{
    ApplyGoals::new(
        list_legal_movesets_filter_strategy(game, partial_game, duration),
        goal,
        game,
        duration,
        depth
    )
}

impl<'a, 'b, B, G, I> ApplyGoals<'a, 'b, B, G, I>
where
    'a: 'b,
    B: Clone + AsRef<Board> + 'a,
    G: Goal<B>,
    I: Iterator<Item=(Moveset, PartialGame<'a, B>)>,
{
    pub fn new(iterator: I, goal: &'b G, game: &'a Game, duration: Option<Duration>, depth: usize) -> Self {
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
            None => None
        }
    }

    pub fn timed_out(&self) -> bool {
        match self.duration {
            Some(duration) => self.sigma > duration,
            None => false,
        }
    }
}

impl<'a, 'b, B, G, I> Iterator for ApplyGoals<'a, 'b, B, G, I>
where
    'a: 'b,
    B: Clone + AsRef<Board>,
    G: Goal<B>,
    I: Iterator<Item=(Moveset, PartialGame<'a, B>)>,
{
    type Item = (Moveset, PartialGame<'a, B>);

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
                        Some(false) => {},
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
