use super::*;
use itertools::Itertools;
use std::convert::TryFrom;

pub struct GenMovesetIter<'a, B, I>
where
    I: Iterator<Item=Move>,
    B: Clone + AsRef<Board> + 'a,
    for<'b> &'b B: GenMoves<'b, B>,
{
    _game: &'a Game,
    partial_game: &'a PartialGame<'a, B>,
    boards: Vec<CacheMoves<I>>,
    states: Vec<Option<usize>>,
    done: bool,
}

impl<'a, B> GenMovesetIter<'a, B, <BoardOr<'a, B> as GenMoves<'a, B>>::Iter>
where
    B: Clone + AsRef<Board> + 'a,
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

pub struct FilterByStrategy<'a, B, I, S>
where
    B: Clone + AsRef<Board> + 'a,
    for<'b> &'b B: GenMoves<'b, B>,
    I: Iterator<Item=Move>,
    S: Strategy<'a, B, From=Move, To=bool>,
{
    iter: I,
    game: &'a Game,
    partial_game: &'a PartialGame<'a, B>,
    _strategy: std::marker::PhantomData<S>,
}

impl<'a, B, I, S> Iterator for FilterByStrategy<'a, B, I, S>
where
    B: Clone + AsRef<Board> + 'a,
    for<'b> &'b B: GenMoves<'b, B>,
    I: Iterator<Item=Move>,
    S: Strategy<'a, B, From=Move, To=bool>,
{
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(mv) => {
                    if S::apply(mv, self.game, self.partial_game)? {
                        return Some(mv)
                    }
                },
                None => return None
            }
        }
    }
}

/** Creates a new GenMovesetIter from a set of boards and a `Move` → `bool` strategy. **/
pub fn generate_movesets_with_strategy<'a, B, S>(
    boards: Vec<BoardOr<'a, B>>,
    game: &'a Game,
    partial_game: &'a PartialGame<'a, B>,
) -> GenMovesetIter<'a, B, FilterByStrategy<'a, B, <BoardOr<'a, B> as GenMoves<'a, B>>::Iter, S>>
where
    B: Clone + AsRef<Board> + 'a,
    for<'b> &'b B: GenMoves<'b, B>,
    S: Strategy<'a, B, From=Move, To=bool>,
{
    GenMovesetIter {
        _game: game,
        partial_game,
        states: vec![None; boards.len()],
        boards: boards
            .into_iter()
            .filter_map(|borb| Some(FilterByStrategy {
                iter: borb.generate_moves(game, partial_game)?,
                game,
                partial_game,
                _strategy: std::marker::PhantomData,
            }))
            .map(|iter| CacheMoves::new(iter))
            .collect(),
        done: false,
    }
}

impl<'a, B, I> GenMovesetIter<'a, B, I>
where
    I: Iterator<Item=Move>,
    B: Clone + AsRef<Board> + 'a,
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
    I: Iterator<Item=Move>,
    B: Clone + AsRef<Board> + 'a,
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
        B: Clone + AsRef<Board> + 'a,
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
