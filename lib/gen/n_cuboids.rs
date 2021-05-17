#[allow(unused_imports)]
use crate::prelude::*;

#[allow(unused_imports)]
use super::*;

#[allow(unused_imports)]
use crate::check::*;

use std::collections::HashMap;
use std::time::Duration;

type HC = HashMap<Layer, Vec<usize>>;

pub fn hc_contains(hypercuboid: HC, point: HashMap<Layer, usize>) -> bool {
    for (l, axis) in hypercuboid.iter() {
        let value = point[l];
        if axis.iter().find(|index| **index == value).is_none() {
            return false;
        }
    }
    true
}

/**
    Given a cross section, cut it out of a hypercuboid, returning a list of disjoint hypercuboids that are
    subsets of the original.
**/
fn cut(mut hypercuboid: HC, sections: &HashMap<Layer, Vec<usize>>) -> Vec<HC> {
    let mut res: Vec<HC> = Vec::with_capacity(sections.len());

    for section in sections {
        let (with_hc, without_hc) = split(hypercuboid, section);
        res.push(without_hc);
        hypercuboid = with_hc;
    }

    res
}

/**
    split a hypercuboid into two
**/
fn split(hypercuboid: HC, section: (&Layer, &Vec<usize>)) -> (HC, HC) {
    let mut with_hc = hypercuboid.clone();
    let mut without_hc = hypercuboid;
    with_hc
        .get_mut(&section.0)
        .unwrap()
        .retain(|index| section.1.iter().find(|target| *target == index).is_some());
    without_hc
        .get_mut(&section.0)
        .unwrap()
        .retain(|index| section.1.iter().find(|target| *target == index).is_none());

    (with_hc, without_hc)
}

/**
    Variant of split, which mutates `hypercuboid` in place
**/
fn split_without(hypercuboid: &mut HC, section: (&Layer, &Vec<usize>)) {
    hypercuboid
        .get_mut(&section.0)
        .unwrap()
        .retain(|index| section.1.iter().find(|target| *target == index).is_none());
}

#[derive(Clone, Debug)]
pub enum AxisLoc {
    Physical(Board, Move),
    Arrive(Board, Move),
    Leave(Board, Coords),
    Pass(Layer, Option<Time>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Sec {
    Single(HashMap<Layer, Vec<usize>>),
    MatchesOne((Layer, Vec<usize>), HashMap<Layer, Vec<usize>>),
}

impl From<(&Layer, &Vec<usize>)> for Sec {
    fn from((l, v): (&Layer, &Vec<usize>)) -> Self {
        let mut hm = HashMap::new();
        hm.insert(*l, v.clone());
        Sec::Single(hm)
    }
}

#[derive(Clone, Debug)]
pub struct Search<'a> {
    // Info (parts of it):
    game: &'a Game,
    partial_game: &'a PartialGame<'a>,
    n_playable: usize,
    n_branching_axes: Layer,

    elements: Vec<AxisLoc>,
    cuboids: Vec<HC>, // Union of cuboids
}

impl<'a> Search<'a> {
    pub fn new(
        game: &'a Game,
        partial_game: &'a PartialGame<'a>,
        max_duration: Option<Duration>,
    ) -> Option<Self> {
        let max_duration = max_duration.unwrap_or(Duration::new(u64::MAX, 1_000_000_000 - 1));
        let mut n_branching_axes = 0;
        let mut branching_axis = Vec::new();
        let mut axes: HashMap<Layer, Vec<usize>> = HashMap::new();
        let mut elements: Vec<AxisLoc> = Vec::new();
        let mut n_playable = 0;

        for board in partial_game.own_boards(game) {
            axes.insert(board.l(), vec![elements.len()]);
            elements.push(AxisLoc::Pass(board.l(), Some(board.t())));
            n_playable += 1;
        }

        for board in partial_game.own_boards(game) {
            let mut has_leaving = false;
            for mv in board.generate_moves(game, partial_game)? {
                if mv.is_jump() {
                    let new_leaving_board = mv.generate_source_board(game, partial_game)?;
                    let new_arriving_board = mv.generate_target_board(game, partial_game)?;

                    // Prevent duplicates
                    if axes[&mv.from.1.l()]
                        .iter()
                        .find(|index| match &elements[**index] {
                            AxisLoc::Leave(_, coords) => mv.from.1 == *coords,
                            _ => false,
                        })
                        .is_none()
                    {
                        if let Some(axis) = axes.get_mut(&mv.from.1.l()) {
                            axis.push(elements.len());
                            elements.push(AxisLoc::Leave(new_leaving_board, mv.from.1));
                        } else {
                            panic!("Invalid layer: {}", mv.from.1.l());
                        }
                    }

                    if partial_game
                        .info
                        .get_timeline(mv.to.1.l())
                        .unwrap()
                        .last_board
                        == mv.to.1.t()
                    {
                        if let Some(axis) = axes.get_mut(&mv.to.1.l()) {
                            axis.push(elements.len());
                            elements.push(AxisLoc::Arrive(new_arriving_board.clone(), mv));
                        } else {
                            panic!("Invalid layer: {}", mv.to.1.l());
                        }
                    }
                    branching_axis.push(elements.len());
                    elements.push(AxisLoc::Arrive(new_arriving_board, mv));
                    has_leaving = true;
                } else {
                    let new_board = mv.generate_source_board(game, partial_game)?;
                    if let Some(axis) = axes.get_mut(&mv.from.1.l()) {
                        axis.push(elements.len());
                        elements.push(AxisLoc::Physical(new_board, mv));
                    } else {
                        panic!("Invalid layer: {}", mv.from.1.l());
                    }
                }
            }
            if has_leaving {
                n_branching_axes += 1;
            }
        }

        // [1; n_branching_axes]
        for n in 1..=n_branching_axes {
            let new_l = if partial_game.info.active_player {
                partial_game.info.max_timeline() + n
            } else {
                partial_game.info.min_timeline() - n
            };

            axes.insert(new_l, branching_axis.clone());
            axes.get_mut(&new_l).unwrap().push(elements.len());
            elements.push(AxisLoc::Pass(new_l, None));
        }

        Some(Self {
            game,
            partial_game,
            n_playable,
            n_branching_axes,
            elements,
            cuboids: vec![axes],
        })
    }

    pub fn remove(&mut self, section: Sec) {
        if let Some(tail_hc) = self.cuboids.pop() {
            match section {
                Sec::Single(sec) => {
                    let res = cut(tail_hc, &sec);
                    // >>= sanity info
                    for hc in res.into_iter() {
                        if let Some(hc) = self.sanity(hc) {
                            self.cuboids.push(hc);
                        }
                    }
                }
                Sec::MatchesOne(leave, branches) => {
                    let (leaving, not_leaving) = split(tail_hc, (&leave.0, &leave.1));
                    let mut no_branches = leaving;
                    let mut res = Vec::new(); // All of the hypercuboids with the leaving point and without the branches
                    let mut exclude = not_leaving; // Hypercuboid with no leave and no branches
                    for j in branches {
                        // exactlyOne
                        let (with_jump, no_jump) = split(no_branches, (&j.0, &j.1));
                        no_branches = no_jump;
                        for x in res.iter_mut() {
                            split_without(x, (&j.0, &j.1));
                        }
                        res.push(with_jump);
                        // foldl' call
                        exclude = split(exclude, (&j.0, &j.1)).1;
                    }
                    res.push(exclude);
                    // >>= sanity info
                    for hc in res.into_iter() {
                        if let Some(hc) = self.sanity(hc) {
                            self.cuboids.push(hc);
                        }
                    }
                }
            }
        } else {
            panic!("No cuboids!");
        }
    }

    pub fn sanity(&self, mut hypercuboid: HC) -> Option<HC> {
        // if any null hc
        for l in self.partial_game.info.min_timeline()..=self.partial_game.info.max_timeline() {
            let axis = hypercuboid.get(&l)?;
            if axis.len() == 0 {
                return None;
            }
        }

        // foldr
        let mut b = false;
        for offset in 1..=self.n_branching_axes {
            let l = if self.partial_game.info.active_player {
                self.partial_game.info.max_timeline() + offset
            } else {
                self.partial_game.info.min_timeline() - offset
            };

            let axis = hypercuboid.get_mut(&l)?;
            // (n:ns), where n is head
            let head = axis.pop()?;
            if let AxisLoc::Pass(_, _) = self.elements[head] {
                // (b && isPass(snd n)) -...-> else
                if !b {
                    axis.push(head);
                }
            } else {
                b = true;
                axis.push(head);
            }

            if axis.len() == 0 {
                return None;
            }
        }

        Some(hypercuboid)
    }

    pub fn find_problems(&self, point: HashMap<Layer, usize>) -> Option<Sec> {
        let mut cell: HashMap<Layer, &AxisLoc> = HashMap::new();
        for (k, v) in &point {
            cell.insert(*k, &self.elements[*v]);
        }
        self.arrives_match_leaves(&cell, &point)
            .or_else(|| self.jump_order_consistent(&cell, &point))
            .or_else(|| self.test_present(&cell, &point))
            .or_else(|| self.find_check(&cell, &point)) //'newState' is only used by this, so can be computed within the function
    }
    /**
    Helper function for arrives_match_leaves
    */
    pub fn make_sec(&self, leaving: Coords, point: &HashMap<Layer, usize>) -> Sec {
        let mut others: HashMap<Layer, Vec<usize>> = HashMap::new();
        for (i, c) in self.elements.iter().enumerate() {
            match c {
                AxisLoc::Arrive(_, m) => {
                    if m.from.1 == leaving {
                        match others.get_mut(&m.to.1.l()) {
                            None => {
                                others.insert(m.to.1.l(), Vec::new());
                                others.get_mut(&m.to.1.l()).unwrap().push(i)
                            }
                            Some(v) => v.push(i),
                        };
                    }
                }
                _ => {}
            }
        }
        return Sec::MatchesOne(
            (leaving.l(), vec![*point.get(&leaving.l()).unwrap()]),
            others,
        );
    }
    pub fn arrives_match_leaves(
        &self,
        cell: &HashMap<Layer, &AxisLoc>,
        point: &HashMap<Layer, usize>,
    ) -> Option<Sec> {
        let jumps: Vec<(&Layer, &Coords)> = cell
            .iter()
            .filter_map(|(l, al)| match al {
                AxisLoc::Arrive(_, m) => Some((l, &m.from.1)),
                _ => None,
            })
            .collect();

        //TODO: check no jumps share a source

        for (_, jsrc) in jumps {
            match cell.get(&jsrc.l()) {
                Some(AxisLoc::Leave(_, lsrc)) => {
                    if lsrc != jsrc {
                        return Some(self.make_sec(*jsrc, point));
                    }
                }
                _ => return Some(self.make_sec(*jsrc, point)),
            }
        }
        // TODO: check that there are no unmatched Leaves
        None
    }
    pub fn jump_order_consistent(
        &self,
        cell: &HashMap<Layer, &AxisLoc>,
        point: &HashMap<Layer, usize>,
    ) -> Option<Sec> {
        //TODO
        None
    }
    pub fn test_present(
        &self,
        cell: &HashMap<Layer, &AxisLoc>,
        point: &HashMap<Layer, usize>,
    ) -> Option<Sec> {
        //TODO
        None
    }
    pub fn find_check(
        &self,
        cell: &HashMap<Layer, &AxisLoc>,
        point: &HashMap<Layer, usize>,
    ) -> Option<Sec> {
        //TODO
        // If moves which will always give check have already been eliminated,
        //   then we only need to consider checks which involve at least 2 new boards.
        // Note that neither the royal piece nor the piece that gives check need to be on the new boards
        //   if the new boards allow a piece to move through them
        None
    }
}
