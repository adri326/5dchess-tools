use super::*;
use itertools::Itertools;
use std::convert::TryFrom;

pub struct GenMovesetIter<'a, B, I>
where
    I: Iterator<Item = Move>,
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
{
    _game: &'a Game,
    partial_game: &'a PartialGame<'a, B>,
    boards: Vec<CacheMoves<I>>,
    states: Vec<Option<usize>>,
    done: bool,
}

// From a set of iterators
impl<'a, B, I> GenMovesetIter<'a, B, I>
where
    I: Iterator<Item = Move>,
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
{
    /** Creates a new GenMovesetIter from a set of move iterators **/
    pub fn from_iters(iters: Vec<I>, game: &'a Game, partial_game: &'a PartialGame<'a, B>) -> Self {
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
}

// From a set of CacheMoves
impl<'a, B, I> GenMovesetIter<'a, B, I>
where
    I: Iterator<Item = Move>,
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
{
    /** Creates a new GenMovesetIter from a set of cached move iterators **/
    pub fn from_cached_iters(
        iters: Vec<CacheMoves<I>>,
        game: &'a Game,
        partial_game: &'a PartialGame<'a, B>,
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

// From BoardOr as GenMoves
impl<'a, B> GenMovesetIter<'a, B, <BoardOr<'a, B> as GenMoves<'a, B>>::Iter>
where
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
{
    /** Creates a new GenMovesetIter from a set of boards. **/
    pub fn new(
        boards: Vec<BoardOr<'a, B>>,
        game: &'a Game,
        partial_game: &'a PartialGame<'a, B>,
    ) -> Self {
        Self {
            _game: game,
            partial_game,
            states: vec![None; boards.len()],
            boards: boards
                .into_iter()
                .filter_map(|borb| CacheMoves::try_from((borb, game, partial_game)).ok())
                .collect(),
            done: false,
        }
    }
}

/**
    Iterator that filters moves from a parent iterator using a given strategy.
**/
pub struct FilterByStrategy<'a, B, I, S>
where
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
    I: Iterator<Item = Move>,
    S: Strategy<'a, B, From = Move, To = bool>,
{
    iter: I,
    game: &'a Game,
    partial_game: &'a PartialGame<'a, B>,
    strategy: S,
}

impl<'a, B, I, S> Iterator for FilterByStrategy<'a, B, I, S>
where
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
    I: Iterator<Item = Move>,
    S: Strategy<'a, B, From = Move, To = bool>,
{
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(mv) => {
                    if self.strategy.apply(mv, self.game, self.partial_game)? {
                        return Some(mv);
                    }
                }
                None => return None,
            }
        }
    }
}

// From a filter strategy
/** Creates a new GenMovesetIter from a set of boards and a `Move` → `bool` strategy. **/
pub fn generate_movesets_filter_strategy<'a, S, B>(
    boards: Vec<BoardOr<'a, B>>,
    game: &'a Game,
    partial_game: &'a PartialGame<'a, B>,
    strategy: S,
) -> GenMovesetIter<'a, B, FilterByStrategy<'a, B, <BoardOr<'a, B> as GenMoves<'a, B>>::Iter, S>>
where
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
    S: Strategy<'a, B, From = Move, To = bool>,
{
    GenMovesetIter {
        _game: game,
        partial_game,
        states: vec![None; boards.len()],
        boards: boards
            .into_iter()
            .filter_map(move |borb| {
                Some(FilterByStrategy {
                    iter: borb.generate_moves(game, partial_game)?,
                    game,
                    partial_game,
                    strategy: strategy.clone(),
                })
            })
            .map(|iter| CacheMoves::new(iter))
            .collect(),
        done: false,
    }
}

// From an iterator strategy
/** Creates a new GenMovesetIter from a set of boards and a `BoardOr<B>` → `Iterator<Item=Move>` strategy. **/
pub fn generate_movesets_iterator_strategy<'a, S, B>(
    boards: Vec<BoardOr<'a, B>>,
    game: &'a Game,
    partial_game: &'a PartialGame<'a, B>,
    strategy: S,
) -> GenMovesetIter<'a, B, S::To>
where
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
    S: Strategy<'a, B, From = BoardOr<'a, B>>,
    <S as Strategy<'a, B>>::To: Iterator<Item = Move>,
{
    GenMovesetIter {
        _game: game,
        partial_game,
        states: vec![None; boards.len()],
        boards: boards
            .into_iter()
            .filter_map(|borb| strategy.apply(borb, game, partial_game))
            .map(|iter| CacheMoves::new(iter))
            .collect(),
        done: false,
    }
}

impl<'a, B, I> GenMovesetIter<'a, B, I>
where
    I: Iterator<Item = Move>,
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
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

impl<'a, B, I> Iterator for GenMovesetIter<'a, B, I>
where
    I: Iterator<Item = Move>,
    B: Clone + AsRef<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
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

pub struct PermMovesetIter<'a> {
    pub physical: Vec<Move>,
    pub non_physical_iter: itertools::structs::Permutations<std::vec::IntoIter<Move>>,
    pub info: &'a Info,
}

impl<'a> PermMovesetIter<'a> {
    pub fn new<B>(
        mut physical: Vec<Move>,
        non_physical: Vec<Move>,
        partial_game: &'a PartialGame<'a, B>,
    ) -> Self
    where
        B: Clone + AsRef<Board>,
    {
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
