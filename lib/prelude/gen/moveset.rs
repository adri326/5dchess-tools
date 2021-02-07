use super::*;
use itertools::Itertools;
use std::convert::TryFrom;

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

pub type GenMovesetPreFilter<'a> = GenMovesetIter<
    'a,
    FilterLegalMove<
        'a,
        BoardIter<'a>>,
>;

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
