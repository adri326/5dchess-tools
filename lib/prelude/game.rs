/*
    Structures and functions related to the game's state.
*/

use std::collections::HashMap;
use super::*;

#[derive(Debug)]
pub struct Game {
    pub boards: HashMap<(Layer, Time), Board>,
    pub width: Physical,
    pub height: Physical,
    pub info: Info,
}

impl Game {
    pub fn new(width: Physical, height: Physical, even_timelines: bool, timelines_white: Vec<TimelineInfo>, timelines_black: Vec<TimelineInfo>) -> Self {
        Game {
            boards: HashMap::new(),
            width,
            height,
            info: Info::new(even_timelines, timelines_white, timelines_black),
        }
    }

    pub fn get_board(&self, (l, t): (Layer, Time)) -> Option<&Board> {
        self.boards.get(&(l, t))
    }

    pub fn get_board_unchecked(&self, (l, t): (Layer, Time)) -> &Board {
        &self.boards[&(l, t)]
    }

    pub fn get(&self, Coords(l, t, x, y): Coords) -> Option<Piece> {
        self.boards.get(&(l, t)).map(|b| b.get((x, y))).flatten()
    }

    pub fn get_unchecked(&self, Coords(l, t, x, y): Coords) -> Option<Piece> {
        self.boards[&(l, t)].get_unchecked((x, y))
    }
}
