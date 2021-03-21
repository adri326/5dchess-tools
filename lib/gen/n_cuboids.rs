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
            return false
        }
    }
    true
}

fn cut(mut hypercuboid: HC, sections: HashMap<Layer, Vec<usize>>) -> Vec<HC> {
    let mut res: Vec<HC> = Vec::with_capacity(sections.len());

    for section in sections {
        let (with_hc, without_hc) = split(hypercuboid, section);
        res.push(without_hc);
        hypercuboid = with_hc;
    }

    res
}

fn split(hypercuboid: HC, section: (Layer, Vec<usize>)) -> (HC, HC) {
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

#[derive(Clone)]
pub enum AxisLoc {
    Physical(Board, Move),
    Arrive(Board, Move),
    Leave(Board, Coords),
    Pass(Layer, Option<Time>),
}

pub struct Search<'a> {
    // Info (parts of it):
    game: &'a Game,
    partial_game: &'a PartialGame<'a>,

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

        for board in partial_game.own_boards(game) {
            axes.insert(board.l(), vec![elements.len()]);
            elements.push(AxisLoc::Pass(board.l(),Some(board.t())) );
        }

        for board in partial_game.own_boards(game) {
            let mut has_leaving = false;
            for mv in board.generate_moves(game, partial_game)? {
                if mv.is_jump() {
                    let new_leaving_board = mv.generate_source_board(game, partial_game)?;
                    let new_arriving_board = mv.generate_target_board(game, partial_game)?;

                    // Prevent duplicates
                    if axes[&mv.from.1.l()].iter().find(|index| {
                        match &elements[**index] {
                            AxisLoc::Leave(_, coords) => mv.from.1 == *coords,
                            _ => false
                        }
                    }).is_none() {
                        if let Some(axis) = axes.get_mut(&mv.from.1.l()) {
                            axis.push(elements.len());
                            elements.push(AxisLoc::Leave(new_leaving_board, mv.from.1));
                        } else {
                            panic!("Invalid layer: {}", mv.from.1.l());
                        }
                    }

                    if partial_game.info.get_timeline(mv.to.1.l()).unwrap().last_board == mv.to.1.t() {
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
            elements.push(AxisLoc::Pass(new_l,None));
        }

        Some(Self {
            game,
            partial_game,
            elements,
            cuboids: vec![axes],
        })
    }

    // TODO: remove
}
