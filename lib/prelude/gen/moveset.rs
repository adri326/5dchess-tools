use super::*;
use std::convert::TryFrom;

pub struct GenMovesetIter<'a, B>
where
    B: Clone + AsRef<Board> + 'a,
    &'a B: GenMoves<'a, B>,
{
    _game: &'a Game,
    partial_game: &'a PartialGame<'a, B>,
    boards: Vec<CacheMoves<'a, B, BoardOr<'a, B>>>,
    states: Vec<Option<usize>>,
    done: bool,
}

impl<'a, B> GenMovesetIter<'a, B>
where
    B: Clone + AsRef<Board> + 'a,
    &'a B: GenMoves<'a, B>,
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
                .filter_map(|borb| CacheMoves::new(borb, game, partial_game))
                .collect(),
            done: false,
        }
    }

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

impl<'a, B> Iterator for GenMovesetIter<'a, B>
where
    B: Clone + AsRef<Board> + 'a,
    &'a B: GenMoves<'a, B>,
{
    type Item = Result<Moveset, MovesetValidityErr>;

    fn next(&mut self) -> Option<Result<Moveset, MovesetValidityErr>> {
        if self.done {
            return None;
        }

        let mut res_vec: Vec<Move> = Vec::with_capacity(self.boards.len());

        for (index, state) in self.states.iter().enumerate() {
            match state {
                Some(n) => {
                    if let Some(m) = self.boards[index].get(*n) {
                        res_vec.push(m);
                    } else {
                        debug_assert!(false, "Expected self.boards[index].get(n) to return true; this is likely an erronerous state.");
                    }
                }
                None => {}
            }
        }

        self.inc();

        Some(Moveset::try_from((res_vec, &self.partial_game.info)))
    }
}

fn inc_option_usize(x: Option<usize>) -> usize {
    match x {
        None => 0,
        Some(y) => y + 1,
    }
}
