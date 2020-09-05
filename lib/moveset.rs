use crate::{game::*, moves::*, resolve::*};

// TODO: optional boards

/**
    An iterator over movesets. Movesets are lazily yielded, based on the sorting done on `moves`.
**/
#[allow(dead_code)]
pub struct MovesetIter<'a> {
    game: &'a Game,
    virtual_boards: &'a Vec<&'a Board>,
    info: GameInfo,
    moves: Vec<Vec<(Move, Vec<Board>, GameInfo, i32)>>,
    pub moves_considered: usize,
    permutation_stack: Vec<Vec<Move>>,
    max_moves: usize,
    pub max_moves_considered: usize,    // 0 for ∞
    pub max_movesets_considered: usize, // 0 for ∞
    pub movesets_considered: usize,
}

impl<'a> Iterator for MovesetIter<'a> {
    type Item = Vec<Move>;
    fn next(&mut self) -> Option<Vec<Move>> {
        self.movesets_considered += 1;
        if self.max_movesets_considered > 0
            && self.movesets_considered > self.max_movesets_considered
            || self.max_moves_considered > 0 && self.moves_considered > self.max_moves_considered
        {
            return None;
        }

        match self.permutation_stack.pop() {
            Some(moveset) => Some(moveset),
            None => {
                if self.moves_considered <= self.max_moves {
                    loop {
                        self.moves_considered += 1;

                        if self.max_moves_considered > 0
                            && self.moves_considered > self.max_moves_considered
                        {
                            return None;
                        }

                        let new_moves = self
                            .moves
                            .iter()
                            .enumerate()
                            .filter(|(_i, m)| m.len() >= self.moves_considered)
                            .map(|(i, _m)| (i, self.moves_considered - 1))
                            .collect::<Vec<_>>();

                        self.generate_combinations(new_moves);

                        if self.permutation_stack.len() > 0 {
                            return Some(self.permutation_stack.pop().unwrap());
                        }
                        if self.moves_considered > self.max_moves {
                            break;
                        }
                    }
                }
                None
            }
        }
    }
}

impl<'a> MovesetIter<'a> {
    /**
    Generates a new MovesetIter. Assumes that `moves` was already sorted.
    **/
    pub fn new(
        game: &'a Game,
        virtual_boards: &'a Vec<&'a Board>,
        info: &'a GameInfo,
        moves: Vec<Vec<(Move, Vec<Board>, GameInfo, i32)>>,
    ) -> Self {
        MovesetIter {
            game,
            virtual_boards,
            info: info.clone(),
            max_moves: moves.iter().map(|m| m.len()).max().unwrap_or(0),
            moves,
            moves_considered: 0,
            permutation_stack: vec![],
            max_movesets_considered: 0,
            max_moves_considered: 0,
            movesets_considered: 0,
        }
    }

    /**
    Called by `next()` when the `permutation_stack` empties out.
    It increases `moves_considered` and generates the combinations made using the new, considered moves.
    **/
    pub fn generate_combinations(&mut self, new_moves: Vec<(usize, usize)>) {
        for (i, nm) in new_moves.into_iter() {
            let pre_combinations = if i > 0 {
                self.generate_pre_combinations(i, 0)
            } else {
                vec![vec![]]
            };
            let post_combinations = if i < self.moves.len() - 1 {
                self.generate_post_combinations(i, self.moves.len() - 1)
            } else {
                vec![vec![]]
            };
            for pre in pre_combinations.into_iter() {
                for post in post_combinations.iter().cloned() {
                    self.commit_combination(
                        pre.iter()
                            .cloned()
                            .chain(vec![(i, nm)].into_iter())
                            .chain(post.into_iter())
                            .map(|(i, m)| {
                                (
                                    self.moves[i][m].0.clone(),
                                    self.moves[i][m].2.clone(),
                                    false,
                                )
                            })
                            .collect::<Vec<_>>(),
                    );
                }
            }
        }
    }

    /**
    Combines moves preceding the current, new, appended move
    **/
    fn generate_pre_combinations(
        &mut self,
        max: usize,
        current: usize,
    ) -> Vec<Vec<(usize, usize)>> {
        if current == max - 1 {
            return (0..(self.moves[current].len().min(self.moves_considered - 1)))
                .map(|n| vec![(current, n)])
                .collect();
        } else {
            let mut res: Vec<Vec<(usize, usize)>> = Vec::new();
            for v in self.generate_pre_combinations(max, current + 1) {
                for x in 0..(self.moves[current].len().min(self.moves_considered - 1)) {
                    let mut v2 = v.clone();
                    v2.push((current, x));
                    res.push(v2);
                }
            }
            return res;
        }
    }

    /**
    Combines moves postceding the current, new, appended move
    **/
    fn generate_post_combinations(
        &mut self,
        min: usize,
        current: usize,
    ) -> Vec<Vec<(usize, usize)>> {
        if current == min + 1 {
            return (0..(self.moves[current].len().min(self.moves_considered - 1)))
                .map(|n| vec![(current, n)])
                .collect();
        } else {
            let mut res: Vec<Vec<(usize, usize)>> = Vec::new();
            for v in self.generate_post_combinations(min, current - 1) {
                for x in 0..(self.moves[current].len().min(self.moves_considered)) {
                    let mut v2 = v.clone();
                    v2.push((current, x));
                    res.push(v2);
                }
            }
            return res;
        }
    }

    /**
    Appends a combination and its derived permutations to `permutation_stack`.
    **/
    fn commit_combination(&mut self, combination: Vec<(Move, GameInfo, bool)>) {
        // NOTE: the (Move, GameInfo, bool) stand for "move", "move's generated info" (used for differentiating jumps from branching moves) and "move locked" (`true` to prevent recursive iterations from cancelling its value out)
        let normal_moves = combination
            .iter()
            .filter(|(m, _i, _a)| m.src.0 == m.dst.0 && m.src.1 == m.dst.1)
            .collect::<Vec<_>>();
        let jumping_moves = combination
            .iter()
            .filter(|(m, i, _a)| {
                i.max_timeline == self.info.max_timeline
                    && i.min_timeline == self.info.min_timeline
                    && (m.src.0 != m.dst.0 || m.src.1 != m.dst.1)
            })
            .collect::<Vec<_>>();
        let branching_moves = combination
            .iter()
            .filter(|(m, i, _a)| {
                (i.max_timeline != self.info.max_timeline
                    || i.min_timeline != self.info.min_timeline)
                    && (m.src.0 != m.dst.0 || m.src.1 != m.dst.1)
            })
            .collect::<Vec<_>>();

        for (k, (jumping_move, _info, locked)) in jumping_moves.iter().enumerate() {
            if *locked {
                continue;
            }
            if let Some(target_move) = combination
                .iter()
                .find(|(m, _i, _a)| m.src.0 == jumping_move.dst.0 && m.src.1 == jumping_move.dst.1)
            {
                self.commit_combination(
                    normal_moves
                        .clone()
                        .into_iter()
                        .cloned()
                        .chain(branching_moves.clone().into_iter().cloned())
                        .chain(jumping_moves.clone().into_iter().cloned().enumerate().map(
                            |(k2, (m, i, a))| {
                                if k2 <= k {
                                    (m, i, true)
                                } else {
                                    (m, i, a)
                                }
                            },
                        ))
                        .filter(|(x, _, _)| {
                            x.src.0 != target_move.0.src.0 || x.src.1 != target_move.0.src.1
                        })
                        .collect::<Vec<_>>(),
                );
            }
        }

        for permutation in permute::permutations_of(
            &jumping_moves
                .into_iter()
                .chain(branching_moves.into_iter())
                .collect::<Vec<_>>(),
        ) {
            let x = permutation
                .cloned()
                .chain(normal_moves.clone().into_iter())
                .map(|(m, _i, _a)| m.clone())
                .collect::<Vec<_>>();
            if x.len() == 0 {
                panic!("No move!");
            }
            self.permutation_stack.push(x);
        }
    }

    /**
    Lazily applies the `score_moveset` function to the movesets and filters out the illegal movesets
    **/
    pub fn score(self) -> impl Iterator<Item = (Vec<Move>, Vec<Board>, GameInfo, f32)> + 'a {
        // NOTE: I still don't quite understand why this doesn't fail to compile
        let game = self.game;
        let virtual_boards = self.virtual_boards;
        let info = self.info;

        self.map(move |ms| {
            score_moveset(
                &game,
                &virtual_boards,
                &info,
                get_opponent_boards(game, virtual_boards, &info).into_iter(),
                ms,
            )
        })
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
    }
}
