use super::*;
use itertools::Itertools;
use std::convert::TryFrom;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct GenMovesetIter<'a, I>
where
    I: Iterator<Item = Move>,
{
    _game: &'a Game,
    partial_game: &'a PartialGame<'a>,
    boards: Vec<CacheMoves<I>>,
    states: Vec<Option<usize>>,
    done: bool,
}

// From BoardOr as GenMoves
impl<'a> GenMovesetIter<'a, BoardIter<'a>> {
    /** Creates a new GenMovesetIter from a set of boards. **/
    pub fn new(boards: Vec<&'a Board>, game: &'a Game, partial_game: &'a PartialGame<'a>) -> Self {
        Self {
            _game: game,
            partial_game,
            states: vec![None; boards.len()],
            boards: boards
                .into_iter()
                .filter_map(|board| CacheMoves::try_from((board, game, partial_game)).ok())
                .collect(),
            done: false,
        }
    }
}

// From a set of iterators
impl<'a, I> GenMovesetIter<'a, I>
where
    I: Iterator<Item = Move>,
{
    /** Creates a new GenMovesetIter from a set of move iterators **/
    pub fn from_iters(iters: Vec<I>, game: &'a Game, partial_game: &'a PartialGame<'a>) -> Self {
        Self {
            _game: game,
            partial_game,
            states: vec![None; iters.len()],
            boards: iters
                .into_iter()
                .map(|iter| CacheMoves::new(iter))
                .collect(),
            done: false,
        }
    }

    /** Creates a new GenMovesetIter from a set of cached move iterators **/
    pub fn from_cached_iters(
        iters: Vec<CacheMoves<I>>,
        game: &'a Game,
        partial_game: &'a PartialGame<'a>,
    ) -> Self {
        Self {
            _game: game,
            partial_game,
            states: vec![None; iters.len()],
            boards: iters,
            done: false,
        }
    }
}

pub type GenMovesetPreFilter<'a> = GenMovesetIter<'a, FilterLegalMove<'a, BoardIter<'a>>>;

// TODO: turn this into a proper, legal moveset generator (deprecating that of utils.rs)
/** Creates a new GenMovesetIter with the moves pre-filtered. **/
pub fn generate_movesets_prefilter<'a>(
    boards: Vec<&'a Board>,
    game: &'a Game,
    partial_game: &'a PartialGame<'a>,
) -> GenMovesetPreFilter<'a> {
    GenMovesetIter {
        _game: game,
        partial_game,
        states: vec![None; boards.len()],
        boards: boards
            .into_iter()
            .filter_map(move |board| {
                Some(FilterLegalMove {
                    iter: board.generate_moves(game, partial_game)?,
                    game,
                    partial_game,
                })
            })
            .map(|iter| CacheMoves::new(iter))
            .collect(),
        done: false,
    }
}

impl<'a, I> GenMovesetIter<'a, I>
where
    I: Iterator<Item = Move>,
{
    /** Increments the `states` **/
    fn inc(&mut self) {
        if self.done {
            return;
        }

        let mut index = 0;
        while index < self.boards.len() {
            let n_state = inc_option_usize(self.states[index]);
            self.states[index] = Some(n_state);
            if let None = self.boards[index].get(n_state) {
                self.states[index] = None;
                index += 1;
                if index >= self.boards.len() {
                    self.done = true;
                }
            } else {
                break;
            }
        }
    }
}

impl<'a, I> Iterator for GenMovesetIter<'a, I>
where
    I: Iterator<Item = Move>,
{
    type Item = PermMovesetIter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let mut physical_moves: Vec<Move> = Vec::with_capacity(self.boards.len());
        let mut non_physical_moves: Vec<Move> = Vec::with_capacity(self.boards.len());

        for (index, state) in self.states.iter().enumerate() {
            match state {
                Some(n) => {
                    if let Some(m) = self.boards[index].get(*n) {
                        if m.is_jump() {
                            non_physical_moves.push(m);
                        } else {
                            physical_moves.push(m);
                        }
                    } else {
                        debug_assert!(false, "Expected self.boards[index].get(n) to return true; this is likely an erronerous state.");
                    }
                }
                None => {}
            }
        }

        self.inc();

        Some(PermMovesetIter::new(
            physical_moves,
            non_physical_moves,
            self.partial_game,
        ))
    }
}

fn inc_option_usize(x: Option<usize>) -> usize {
    match x {
        None => 0,
        Some(y) => y + 1,
    }
}

#[derive(Clone)]
pub struct PermMovesetIter<'a> {
    pub physical: Vec<Move>,
    pub non_physical_iter: itertools::structs::Permutations<std::vec::IntoIter<Move>>,
    pub info: &'a Info,
}

impl<'a> PermMovesetIter<'a> {
    pub fn new(
        mut physical: Vec<Move>,
        non_physical: Vec<Move>,
        partial_game: &'a PartialGame<'a>,
    ) -> Self {
        physical.shrink_to_fit();

        let length = non_physical.len();

        Self {
            physical,
            non_physical_iter: non_physical.into_iter().permutations(length),
            info: &partial_game.info,
        }
    }
}

impl<'a> Iterator for PermMovesetIter<'a> {
    type Item = Result<Moveset, MovesetValidityErr>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.non_physical_iter.next() {
            Some(mut non_physical) => {
                let mut res = self.physical.clone();
                res.append(&mut non_physical);
                Some(Moveset::try_from((res, self.info)))
            }
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.non_physical_iter.size_hint()
    }
}

/**
    An iterator, similar to GenMovesetIter, which only yields legal movesets.
    It does its best to look for legal movesets as fast a possible. To help it, you can put the non-check boards first.
**/
#[derive(Clone)]
pub struct GenLegalMovesetIter<'a> {
    // Base state
    game: &'a Game,
    partial_game: &'a PartialGame<'a>,

    done: bool,
    first_pass: bool,
    iter_tampered: bool,

    boards: Vec<CacheMoves<FilterLegalMove<'a, Sigma<BoardIter<'a>>>>>,
    /// A variable-basis state counter, with a special value (None) for "no move"
    states: Vec<Option<usize>>,

    // Timing
    sigma: Duration,
    max_duration: Duration,

    // Permutation
    physical_moves: Vec<Move>,
    non_physical_iter: Option<itertools::structs::Permutations<std::vec::IntoIter<Move>>>,
}

impl<'a> GenLegalMovesetIter<'a> {
    pub fn new(
        game: &'a Game,
        partial_game: &'a PartialGame<'a>,
        max_duration: Option<Duration>,
    ) -> Self {
        let max_duration = max_duration.unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1));
        let start = Instant::now();
        let boards = partial_game
            .own_boards(game)
            .filter_map(|board| {
                Some(CacheMoves::new(FilterLegalMove::new(
                    board
                        .generate_moves(game, partial_game)?
                        .sigma(max_duration),
                    game,
                    partial_game,
                )))
            })
            .collect::<Vec<_>>();
        let states = vec![None; boards.len()];

        Self {
            game,
            partial_game,

            done: false,
            first_pass: true,
            iter_tampered: true,

            boards,
            states,

            sigma: start.elapsed(),
            max_duration,

            physical_moves: vec![],
            non_physical_iter: None,
        }
    }

    /** Increments the `states`; also updates `current_partial_games`, `physical_moves` and `non_physical_iter` **/
    fn inc(&mut self) {
        let mut index: usize = 0;
        let start = Instant::now();
        if self.done && !self.first_pass || self.sigma > self.max_duration {
            return;
        }

        // Increment the state once
        while index < self.boards.len() {
            let n_state = inc_option_usize(self.states[index]);
            self.states[index] = Some(n_state);

            if let None = self.boards[index].get(n_state) {
                self.states[index] = None;
                index += 1;
                if index >= self.boards.len() {
                    self.done = true;
                }
            } else {
                break;
            }
        }

        if self.done {
            if self.first_pass {
                self.first_pass = false;
                self.iter_tampered = false;
                self.done = false;
                self.states = vec![None; self.boards.len()];
                self.sigma += start.elapsed();
                self.non_physical_iter = None;
                return self.inc()
            } else {
                self.sigma += start.elapsed();
                return
            }
        }


        // Update physical_moves, remaining_physical_moves and non_physical_iter
        let mut physical_moves = Vec::new();
        let mut non_physical_moves = Vec::new();
        for index in 0..self.states.len() {
            if let Some(state) = self.states[index] {
                if let Some(mv) = self.boards[index].get(state) {
                    if mv.is_jump() {
                        non_physical_moves.push(mv);
                    } else {
                        physical_moves.push(mv);
                    }
                }
            }
        }

        let length = non_physical_moves.len();
        if self.first_pass || length > 1 {
            self.physical_moves = physical_moves;

            let mut iter = non_physical_moves.into_iter().permutations(length);
            if !self.first_pass {
                iter.next();
            }

            self.non_physical_iter = Some(iter);
            self.iter_tampered = false;
        } else {
            self.non_physical_iter = None;
        }

        self.sigma += start.elapsed();
    }

    pub fn timed_out(&self) -> bool {
        self.sigma >= self.max_duration
    }
}

impl<'a> Iterator for GenLegalMovesetIter<'a> {
    type Item = (Moveset, PartialGame<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let start = Instant::now();

        if self.done {
            return None;
        }

        Some(loop {
            if self.done || start.elapsed() + self.sigma > self.max_duration {
                self.sigma += start.elapsed();
                return None;
            }

            while self.first_pass && self.iter_tampered {
                self.inc();
            }

            if let Some(iter) = &mut self.non_physical_iter {
                self.iter_tampered = true;
                if let Some(mut perm) = iter.next() {
                    let mut moves = self.physical_moves.clone();
                    moves.append(&mut perm);

                    if let Ok(ms) = Moveset::new(moves, &self.partial_game.info) {
                        let mut new_partial_game = PartialGame::empty(self.partial_game.info.clone(), Some(self.partial_game));

                        // Generate the remaining boards
                        for i in 0..ms.moves().len() {
                            let mv = &ms.moves()[i];

                            mv.generate_partial_game(
                                self.game,
                                self.partial_game,
                                &mut new_partial_game,
                                PartialGameGenKind::Both,
                            );
                        }

                        new_partial_game.info.recalculate_present();
                        if new_partial_game.info.active_player
                            != self.partial_game.info.active_player
                        {
                            if let Some(false) = is_illegal(self.game, &new_partial_game) {
                                self.sigma += start.elapsed();
                                break (ms, new_partial_game);
                            }
                        }
                    }
                } else {
                    self.inc();
                }
            } else {
                self.inc();
            }
        })
    }
}
