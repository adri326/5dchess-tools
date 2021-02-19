/*
    Structures and functions related to the game's state.
*/

use super::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Game {
    boards: HashMap<(Layer, Time), Board>,
    pub width: Physical,
    pub height: Physical,
    pub info: Info,
}

impl Game {
    pub fn new(
        width: Physical,
        height: Physical,
        even_timelines: bool,
        timelines_white: Vec<TimelineInfo>,
        timelines_black: Vec<TimelineInfo>,
    ) -> Self {
        Game {
            boards: HashMap::new(),
            width,
            height,
            info: Info::new(even_timelines, timelines_white, timelines_black),
        }
    }

    #[inline]
    pub fn get_board(&self, (l, t): (Layer, Time)) -> Option<&Board> {
        self.boards.get(&(l, t))
    }

    #[inline]
    pub fn get_board_mut(&mut self, (l, t): (Layer, Time)) -> Option<&mut Board> {
        self.boards.get_mut(&(l, t))
    }

    #[inline]
    pub fn get_board_unchecked(&self, (l, t): (Layer, Time)) -> &Board {
        &self.boards[&(l, t)]
    }

    #[inline]
    pub fn get(&self, Coords(l, t, x, y): Coords) -> Tile {
        self.boards.get(&(l, t)).map(|b| b.get((x, y))).into()
    }

    #[inline]
    pub fn get_unchecked(&self, Coords(l, t, x, y): Coords) -> Tile {
        self.boards[&(l, t)].get_unchecked((x, y))
    }

    #[inline]
    pub fn insert_board(&mut self, board: Board) {
        let coords = (board.l(), board.t());
        self.boards.insert(coords, board);
    }

    #[inline]
    pub fn iter_boards<'a>(&'a self) -> impl Iterator<Item=(&(Layer, Time), &'a Board)> {
        self.boards.iter()
    }

    #[inline]
    pub fn iter_boards_mut<'a>(&'a mut self) -> impl Iterator<Item=(&(Layer, Time), &'a mut Board)> {
        self.boards.iter_mut()
    }
}
