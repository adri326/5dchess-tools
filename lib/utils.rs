use crate::prelude::moveset::FilterByStrategy;
use crate::strategies::legal::OptLegalMove;
use crate::*;
use std::time::{Duration, Instant};

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
