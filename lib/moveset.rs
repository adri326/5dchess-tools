use crate::{game::*, moves::*, resolve::*};

// TODO: optional boards

pub struct MovesetIter<'a> {
    game: &'a Game,
    virtual_boards: &'a Vec<Board>,
    info: GameInfo,
    moves: Vec<Vec<(Move, Vec<Board>, GameInfo, i32)>>,
    pub moves_considered: usize,
    permutation_stack: Vec<Vec<Move>>,
    max_moves: usize,
}

pub fn generate_movesets<'a>(
    game: &'a Game,
    virtual_boards: &'a Vec<Board>,
    info: &'a GameInfo,
    moves: Vec<Vec<(Move, Vec<Board>, GameInfo, i32)>>,
) -> MovesetIter<'a> {
    MovesetIter {
        game,
        virtual_boards,
        info: info.clone(),
        max_moves: moves.iter().map(|m| m.len()).max().unwrap_or(0),
        moves,
        moves_considered: 0,
        permutation_stack: vec![],
    }
}

impl<'a> Iterator for MovesetIter<'a> {
    type Item = Vec<Move>;
    fn next(&mut self) -> Option<Vec<Move>> {
        match self.permutation_stack.pop() {
            Some(moveset) => Some(moveset),
            None => {
                if self.moves_considered <= self.max_moves {
                    loop {
                        self.moves_considered += 1;
                        let new_moves = self
                            .moves
                            .iter()
                            .enumerate()
                            .filter(|(_i, m)| m.len() >= self.moves_considered)
                            .map(|(i, m)| (i, self.moves_considered - 1))
                            .collect::<Vec<_>>();

                        generate_combinations(self, new_moves);

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

fn generate_combinations<'a>(iter: &mut MovesetIter<'a>, new_moves: Vec<(usize, usize)>) {
    for (i, nm) in new_moves.into_iter() {
        let pre_combinations = if i > 0 {
            generate_pre_combinations(iter, i, 0)
        } else {
            vec![vec![]]
        };
        let post_combinations = if i < iter.moves.len() - 1 {
            generate_post_combinations(iter, i, iter.moves.len() - 1)
        } else {
            vec![vec![]]
        };
        for pre in pre_combinations.into_iter() {
            for post in post_combinations.iter().cloned() {
                commit_combination(
                    iter,
                    pre.iter()
                        .cloned()
                        .chain(vec![(i, nm)].into_iter())
                        .chain(post.into_iter())
                        .map(|(i, m)| {
                            (
                                iter.moves[i][m].0.clone(),
                                iter.moves[i][m].2.clone(),
                                false,
                            )
                        })
                        .collect::<Vec<_>>(),
                );
            }
        }
    }
}

fn generate_pre_combinations<'a>(
    iter: &mut MovesetIter<'a>,
    max: usize,
    current: usize,
) -> Vec<Vec<(usize, usize)>> {
    if current == max - 1 {
        return (0..(iter.moves[current].len().min(iter.moves_considered - 1)))
            .map(|n| vec![(current, n)])
            .collect();
    } else {
        let mut res: Vec<Vec<(usize, usize)>> = Vec::new();
        for v in generate_pre_combinations(iter, max, current + 1) {
            for x in 0..(iter.moves[current].len().min(iter.moves_considered - 1)) {
                let mut v2 = v.clone();
                v2.push((current, x));
                res.push(v2);
            }
        }
        return res;
    }
}

fn generate_post_combinations<'a>(
    iter: &mut MovesetIter<'a>,
    min: usize,
    current: usize,
) -> Vec<Vec<(usize, usize)>> {
    if current == min + 1 {
        return (0..(iter.moves[current].len().min(iter.moves_considered - 1)))
            .map(|n| vec![(current, n)])
            .collect();
    } else {
        let mut res: Vec<Vec<(usize, usize)>> = Vec::new();
        for v in generate_post_combinations(iter, min, current - 1) {
            for x in 0..(iter.moves[current].len().min(iter.moves_considered)) {
                let mut v2 = v.clone();
                v2.push((current, x));
                res.push(v2);
            }
        }
        return res;
    }
}

fn commit_combination<'a>(iter: &mut MovesetIter<'a>, combination: Vec<(Move, GameInfo, bool)>) {
    // NOTE: the (Move, GameInfo, bool) stand for "move", "move's generated info" (used for differentiating jumps from branching moves) and "move locked" (`true` to prevent recursive iterations from cancelling its value out)
    let normal_moves = combination
        .iter()
        .filter(|(m, _i, _a)| m.src.0 == m.dst.0 && m.src.1 == m.dst.1)
        .collect::<Vec<_>>();
    let jumping_moves = combination
        .iter()
        .filter(|(m, i, _a)| {
            i.max_timeline == iter.info.max_timeline
                && i.min_timeline == iter.info.min_timeline
                && (m.src.0 != m.dst.0 || m.src.1 != m.dst.1)
        })
        .collect::<Vec<_>>();
    let branching_moves = combination
        .iter()
        .filter(|(m, i, _a)| {
            (i.max_timeline != iter.info.max_timeline || i.min_timeline != iter.info.min_timeline)
                && (m.src.0 != m.dst.0 || m.src.1 != m.dst.1)
        })
        .collect::<Vec<_>>();

    for (k, (jumping_move, info, locked)) in jumping_moves.iter().enumerate() {
        if *locked {
            continue;
        }
        if let Some(target_move) = combination
            .iter()
            .find(|(m, _i, _a)| m.src.0 == jumping_move.dst.0 && m.src.1 == jumping_move.dst.1)
        {
            commit_combination(
                iter,
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
        iter.permutation_stack.push(x);
    }
}
