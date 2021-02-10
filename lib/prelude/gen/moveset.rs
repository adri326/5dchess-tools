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

    boards: Vec<CacheMoves<FilterLegalMove<'a, Sigma<BoardIter<'a>>>>>,
    /// A variable-basis state counter, with a special value (None) for "no move"
    states: Vec<Option<usize>>,

    /// A stack of partial games, containing the pre-generated moves for boards[MIN_CACHING_DEPTH..]
    /// It grows in reverse compared to `boards` and `states`; see the comments in `inc` for details about its structure
    current_partial_games: Vec<PartialGame<'a>>,

    // Timing
    sigma: Duration,
    max_duration: Duration,

    // Permutation
    physical_moves: Vec<Move>,
    remaining_physical_moves: Vec<Move>,
    non_physical_iter: Option<itertools::structs::Permutations<std::vec::IntoIter<(Move, bool)>>>,
}

const MIN_CACHING_DEPTH: usize = 2;

// This was probably a mistake :/
// TODO: there are some shady, unnecessary calls to Board::generate_partial_game, where the timeline's last_board was incremented twice.
// I have changed generate_partial_game so that it doesn't blindly increment it anymore, but these calls should be tracked down.

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
        let current_partial_games: Vec<PartialGame<'a>> =
            Vec::with_capacity(if states.len() >= MIN_CACHING_DEPTH {
                states.len() - MIN_CACHING_DEPTH
            } else {
                0
            });

        Self {
            game,
            partial_game,

            done: false,

            boards,
            states,

            current_partial_games,

            sigma: start.elapsed(),
            max_duration,

            physical_moves: vec![],
            remaining_physical_moves: vec![],
            non_physical_iter: None,
        }
    }

    fn get_new_partial_game(&self) -> PartialGame<'a> {
        self.current_partial_games
            .last()
            .cloned()
            .unwrap_or(PartialGame::empty(
                self.partial_game.info.clone(),
                Some(self.partial_game),
            ))
    }

    /** Increments the `states`; also updates `current_partial_games`, `physical_moves` and `non_physical_iter` **/
    fn inc(&mut self) {
        let mut index: usize = 0;
        let start = Instant::now();
        if self.done {
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

        if index >= MIN_CACHING_DEPTH && index < self.boards.len() {
            // Loop to update the PartialGame stack and look for move legality
            'l: loop {
                if self.done || start.elapsed() + self.sigma > self.max_duration {
                    self.sigma += start.elapsed();
                    return;
                }

                // Re-construct the PartialGame stack...

                // We wish to have, if index = 4:
                // P₀, P₁, (P₂), X, X, _, _
                // Where P₃ is the new, generated partial game, X are partial games to be removed and _ are partial games that aren't on the stack
                // because of MIN_CACHING_DEPTH.
                // If index = 3:
                // P₀, P₁, P₂, (P₃), X, _, _

                // Note that the partial game stack grows in reverse order; if MIN_CACHING_DEPTH = 2, it'd look like that:
                // Board    = [A, B, C, D, E, F, G]
                // Partials = [_, _, 4, 3, 2, 1, 0]

                // Add the missing partial games (might happen on some rare carries; I just can't wrap my head around it, though)
                while self.current_partial_games.len() + index + 1 < self.states.len() {
                    let new_partial_game = self.get_new_partial_game();
                    self.current_partial_games.push(new_partial_game);
                }
                // Remove the outdated partial games
                while self.current_partial_games.len() + index >= self.boards.len() {
                    self.current_partial_games.pop(); // Be gone!
                }

                // We now add a new partial game. The remaining partial games after that are handled once we exit this loop

                if self.done || start.elapsed() + self.sigma > self.max_duration {
                    self.sigma += start.elapsed();
                    return;
                }

                let mut new_partial_game = self.get_new_partial_game();

                // Try to add the corresponding source board
                if let Some(i) = self.states[index] {
                    if let Some(mv) = self.boards[index].get(i) {
                        mv.generate_partial_game(
                            self.game,
                            self.partial_game,
                            &mut new_partial_game,
                            PartialGameGenKind::Source,
                        );
                    }
                }

                if let Some(false) = is_in_check(self.game, &new_partial_game) {
                    self.current_partial_games.push(new_partial_game);

                    // Everything okay, exit the loop
                    break;
                } else {
                    // The position is illegal; no subsequent moves using self.boards[self.states[index]] can be legal, thus we increment
                    // self.states[index] (and do carries if need be)
                    // No need to update self.states before index, as they should all be set to None already

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
                            // Carry done, loop back to update the partial game stack
                            continue 'l;
                        }
                    }
                }
            }

            // End of the loop; we add the missing partial games if need be
            // TODO: maybe don't have this? It'll be done if we need to update the stack anyways
            while self.current_partial_games.len() + MIN_CACHING_DEPTH <= self.states.len() {
                let new_partial_game = self.get_new_partial_game();
                self.current_partial_games.push(new_partial_game);
            }
        }

        // Update physical_moves, remaining_physical_moves and non_physical_iter
        let mut physical_moves = Vec::new();
        let mut remaining_physical_moves = Vec::new();
        let mut non_physical_moves = Vec::new();
        for index in 0..self.states.len() {
            if let Some(state) = self.states[index] {
                if let Some(mv) = self.boards[index].get(state) {
                    if mv.is_jump() {
                        non_physical_moves.push((mv, index >= MIN_CACHING_DEPTH));
                    } else if index < MIN_CACHING_DEPTH {
                        remaining_physical_moves.push(mv);
                    } else {
                        physical_moves.push(mv);
                    }
                }
            }
        }
        self.physical_moves = physical_moves;
        self.remaining_physical_moves = remaining_physical_moves;

        let length = non_physical_moves.len();
        self.non_physical_iter = Some(non_physical_moves.into_iter().permutations(length));

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

            if let Some(iter) = &mut self.non_physical_iter {
                if let Some(perm) = iter.next() {
                    let mut moves = self.physical_moves.clone();
                    let mut only_target: Vec<bool> = vec![true; moves.len()];

                    for mv in &self.remaining_physical_moves {
                        moves.push(mv.clone());
                        only_target.push(false);
                    }

                    for (mv, target) in perm {
                        only_target.push(target);
                        moves.push(mv);
                    }

                    if let Ok(ms) = Moveset::new(moves, &self.partial_game.info) {
                        let mut new_partial_game = self.get_new_partial_game();

                        // Generate the remaining boards
                        for i in 0..ms.moves().len() {
                            let mv = &ms.moves()[i];
                            if only_target[i] {
                                mv.generate_partial_game(
                                    self.game,
                                    self.partial_game,
                                    &mut new_partial_game,
                                    PartialGameGenKind::Target,
                                );
                            } else {
                                mv.generate_partial_game(
                                    self.game,
                                    self.partial_game,
                                    &mut new_partial_game,
                                    PartialGameGenKind::Both,
                                );
                            }
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
