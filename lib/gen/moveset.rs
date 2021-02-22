use super::*;
use crate::check::*;
use itertools::Itertools;
use std::convert::TryFrom;
use std::time::{Duration, Instant};
use std::borrow::Cow;

/**
    An iterator that yields all of the valid movesets.
    Some (most) of them will not shift the present, some will be illegal, etc.
    All of the movesets are however valid, which means that the moveset can be executed, but not submitted.
**/
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

// From a set of boards or the position
impl<'a> GenMovesetIter<'a, BoardIter<'a>> {
    // TODO: rename `new` to `with_boards` and uncomment the following:
    // pub fn new(game: &'a Game, partial_game: &'a PartialGame<'a>) -> Self {
    //     Self {
    //         _game: game,
    //         partial_game,
    //         states: vec![None; boards.len()],
    //         boards: partial_game
    //             .own_boards(game)
    //             .filter_map(|board| CacheMoves::try_from((board, game, partial_game)).ok())
    //             .collect(),
    //         done: false,
    //     }
    // }

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

// TODO: deprecate this
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

/// Maps ℕ ∪ {∅} to ℕ, with f(∅) = 0 and f(n) = succ(n)
fn inc_option_usize(x: Option<usize>) -> usize {
    match x {
        None => 0,
        Some(y) => y + 1,
    }
}

/// Permutes moves within a moveset; returned by `GenMovesetIter`.
/// If you wish to list all movesets, you should `Iterator::flatten` `GenMovesetIter`.
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

/// Length of `GenLegalMovesetIter::attackers` at which a trim is triggered
const ATTACKERS_THRESHOLD: usize = 100;

/// Length to trim `GenLegalMovesetIter::attackers` to when a trim is triggered
const ATTACKERS_PREFERED_LEN: usize = 20;

/// Number of trims after which it is considered unnecessary to record any more attackers
const ATTACKERS_MAX_TRIM: usize = 20;

/**
    An iterator, similar to GenMovesetIter, which only yields legal movesets.
    It does its best to look for legal movesets as fast a possible.

    You should prefer using this rather than `GenMovesetIter` if you are only interested by the legal movesets, as `GenLegalMovesetIter`
    implements checkmate-specific optimizations to the search.

    It implements the following optimizations:

    - Permutation-free first pass (better performances in some worst-case non-mate scenarios)
    - Attacker pool (between 10% and 70% speedup, average 50%); can be customized by changing the values of the `ATTACKERS_*` constants
    - Board caching and minimal number of clones
    - Lazy move, moveset, permutation and checking move generation
**/
#[derive(Clone)]
pub struct GenLegalMovesetIter<'a> {
    // Base state
    game: &'a Game,
    partial_game: Cow<'a, PartialGame<'a>>,

    pub done: bool,
    first_pass: bool,

    boards: Vec<CacheMovesBoards<'a, FilterLegalMove<'a, Sigma<BoardIter<'a>>>>>,
    /// A variable-basis state counter, with a special value (None) for "no move"
    states: Vec<Option<usize>>,

    // Timing
    sigma: Duration,
    max_duration: Duration,

    // Permutation
    physical_moves: Vec<(usize, usize)>,
    non_physical_iter: Option<itertools::structs::Permutations<std::vec::IntoIter<(usize, usize)>>>,
    non_physical_moves: Option<Vec<(usize, usize)>>,

    // Attacker cache
    attackers: Vec<(Coords, usize)>,
    attackers_trim_count: usize,
}

impl<'a> GenLegalMovesetIter<'a> {
    /**
        Creates a new instance of GenLegalMovesetIter.
    **/
    pub fn new(
        game: &'a Game,
        partial_game: Cow<'a, PartialGame<'a>>,
        max_duration: Option<Duration>,
    ) -> Self {
        let max_duration = max_duration.unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1));
        let start = Instant::now();
        let mut boards: Vec<_> = Vec::new();

        match &partial_game {
            Cow::Borrowed(partial_game) => {
                for board in partial_game.own_boards(game) {
                    if let Some(moves) = board.generate_moves(game, partial_game) {
                        boards.push(CacheMovesBoards::new(
                            FilterLegalMove::new(
                                moves.sigma(max_duration),
                                game,
                                partial_game,
                            ),
                            game,
                            partial_game,
                        ));
                    }
                }
            }
            Cow::Owned(partial_game) => unsafe {
                // This is sound iff `CacheMoves::consume` guarantees that `res.done == true`, hence the assertion
                let partial_game: *const PartialGame<'a> = &*partial_game;
                for board in partial_game.as_ref().unwrap().own_boards(game) {
                    if let Some(moves) = board.generate_moves(game, &*partial_game) {
                        let mut res = CacheMovesBoards::new(
                            FilterLegalMove::new(
                                moves.sigma(max_duration),
                                game,
                                &*partial_game,
                            ),
                            game,
                            &*partial_game,
                        );
                        res.consume();

                        assert!(res.done());

                        boards.push(res);
                    }
                }
            }
        }

        let states = vec![None; boards.len()];

        Self {
            game,
            partial_game,

            done: false,
            first_pass: true,

            boards,
            states,

            sigma: start.elapsed(),
            max_duration,

            physical_moves: vec![],
            non_physical_iter: None,
            non_physical_moves: None,

            attackers: Vec::with_capacity(ATTACKERS_THRESHOLD),
            attackers_trim_count: 0,
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
                self.done = false;
                self.states = vec![None; self.boards.len()];
                self.sigma += start.elapsed();
                self.non_physical_iter = None;
                self.non_physical_moves = None;
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
                        non_physical_moves.push((index, state));
                    } else {
                        physical_moves.push((index, state));
                    }
                }
            }
        }

        let length = non_physical_moves.len();
        if self.first_pass {
            self.physical_moves = physical_moves;
            self.non_physical_moves = Some(non_physical_moves);
        } else if length > 1 {
            self.physical_moves = physical_moves;

            let mut iter = non_physical_moves.into_iter().permutations(length);
            iter.next();

            self.non_physical_iter = Some(iter);
        } else {
            self.non_physical_iter = None;
        }

        self.sigma += start.elapsed();
    }

    /** Whether or not the iterator stopped yielding because it timed out. **/
    pub fn timed_out(&self) -> bool {
        self.sigma >= self.max_duration
    }

    /**
        Takes a permutation, assembles the set of moves and boards, updates `attackers` if need be.
        Returns `Some((moveset, position))` if `apply(G, moveset)` is legal, `None` otherwise.
    **/
    #[inline]
    fn yield_perm(&mut self, perm: Vec<(usize, usize)>) -> Option<(Moveset, PartialGame<'a>)> {
        let mut moves = Vec::with_capacity(self.physical_moves.len() + perm.len());
        let mut boards = Vec::with_capacity(self.physical_moves.len() + perm.len());

        for &(index, state) in &self.physical_moves {
            moves.push(self.boards[index].get_cached(state).unwrap());
            boards.push(self.boards[index].get_board_cached(state).unwrap());
        }

        for (index, state) in perm {
            moves.push(self.boards[index].get_cached(state).unwrap());
            boards.push(self.boards[index].get_board_cached(state).unwrap());
        }

        if let Ok(ms) = Moveset::new(moves, &self.partial_game.info) {
            let mut new_partial_game = match &self.partial_game {
                Cow::Borrowed(partial_game) => PartialGame::empty(self.partial_game.info.clone(), Some(partial_game)),
                Cow::Owned(partial_game) => {
                    let mut res = partial_game.clone();
                    res.deepen();
                    res
                }
            };

            // Generate the remaining boards
            for (mv, (source, target)) in ms.moves().iter().zip(boards.into_iter()) {
                mv.insert_source_board(
                    &mut new_partial_game,
                    source.into_owned(),
                );

                match target {
                    std::borrow::Cow::Borrowed(Some(target)) => {
                        mv.insert_target_board(
                            &mut new_partial_game,
                            target.clone(),
                        );
                    }
                    std::borrow::Cow::Owned(Some(target)) => {
                        mv.insert_target_board(
                            &mut new_partial_game,
                            target,
                        );
                    }
                    _ => {}
                }
            }

            new_partial_game.info.recalculate_present();
            if new_partial_game.info.active_player
                != self.partial_game.info.active_player
            {
                // Quick glance at whether or not one of the known attackers is checking us
                for (attacker, value) in &mut self.attackers {
                    if let Tile::Piece(piece) = new_partial_game.get_with_game(self.game, *attacker) {
                        if piece.white == new_partial_game.info.active_player {
                            let piece_position = PiecePosition::new(piece, *attacker);

                            for mv in piece_position.generate_moves(self.game, &new_partial_game).unwrap() {
                                if let Some(target) = mv.to.0 {
                                    if target.is_royal() && target.white != new_partial_game.info.active_player {
                                        *value += 1;
                                        return None
                                    }
                                }
                            }
                        }
                    }
                }

                // Thorough check verification
                let illegal = if self.game.width <= MAX_BITBOARD_WIDTH as Physical {
                    is_illegal_bitboard(self.game, &new_partial_game)
                } else {
                    is_illegal(self.game, &new_partial_game)
                };
                match illegal {
                    Some((false, _)) => return Some((ms, new_partial_game)),
                    Some((true, None)) => {}
                    Some((true, Some(mv))) => {
                        self.insert_attacker(mv.from.1);
                    }
                    None => return None
                }
            }
        }

        None
    }

    /**
        Updates `attackers`, given the coordinates of such an attacker.
    **/
    #[inline]
    fn insert_attacker(&mut self, coords: Coords) {
        // Skip trimming and updating the array after a set number of trims
        if self.attackers_trim_count >= ATTACKERS_MAX_TRIM {
            return
        }
        if self.attackers.len() > 0 {
            for attacker in &mut self.attackers {
                if attacker.0 == coords {
                    attacker.1 += 1;
                    return
                }
            }
        }
        self.attackers.push((coords, 1));

        // Too many attackers: trimming
        if self.attackers.len() >= ATTACKERS_THRESHOLD {
            self.attackers.sort_unstable_by(
                #[inline]
                |a: &(Coords, usize), b: &(Coords, usize)| -> std::cmp::Ordering {
                    b.1.partial_cmp(&a.1).unwrap()
                }
            );
            self.attackers.truncate(ATTACKERS_PREFERED_LEN);
            self.attackers_trim_count += 1;
        }
    }
}

impl<'a> Iterator for GenLegalMovesetIter<'a> {
    type Item = (Moveset, PartialGame<'a>);

    /**
        Yields the next legal moveset, if any. If `None` is yielded, check whether or not it was because of a timeout,
        using `GenLegalMovesetIter::timed_out`!
    **/
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

            if self.first_pass {
                if let Some(perm) = std::mem::replace(&mut self.non_physical_moves, None) {
                    match self.yield_perm(perm) {
                        Some(res) => {
                            self.sigma += start.elapsed();
                            break res
                        }
                        None => {},
                    }
                }
                self.inc();
            } else {
                if let Some(iter) = &mut self.non_physical_iter {
                    if let Some(perm) = iter.next() {
                        match self.yield_perm(perm) {
                            Some(res) => {
                                self.sigma += start.elapsed();
                                break res
                            }
                            None => {},
                        }
                    } else {
                        self.inc();
                    }
                } else {
                    self.inc();
                }
            }
        })
    }
}
