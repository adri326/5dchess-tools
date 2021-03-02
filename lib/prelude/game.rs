/*
    Structures and functions related to the game's state.
*/

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    pub boards: BoardArray,
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
            boards: BoardArray::empty(),
            width,
            height,
            info: Info::new(even_timelines, timelines_white, timelines_black),
        }
    }

    #[inline]
    pub fn get_board(&self, (l, t): (Layer, Time)) -> Option<&Board> {
        self.boards.get_board((l, t))
    }

    #[inline]
    pub fn get_board_mut(&mut self, (l, t): (Layer, Time)) -> Option<&mut Board> {
        self.boards.get_board_mut((l, t))
    }

    #[inline]
    pub fn get_board_unchecked(&self, (l, t): (Layer, Time)) -> &Board {
        self.boards.get_board_unchecked((l, t))
    }

    #[inline]
    pub fn get(&self, Coords(l, t, x, y): Coords) -> Tile {
        self.get_board((l, t)).map(|b| b.get((x, y))).into()
    }

    #[inline]
    pub fn get_unchecked(&self, Coords(l, t, x, y): Coords) -> Tile {
        self.boards.get_board_unchecked((l, t)).get_unchecked((x, y))
    }

    #[inline]
    pub fn insert_board(&mut self, board: Board) {
        self.boards.insert_board(board)
    }

    #[inline]
    pub fn iter_boards<'a>(&'a self) -> impl Iterator<Item=((Layer, Time), &'a Board)> {
        self.boards.iter_boards()
    }

    #[inline]
    pub fn iter_boards_mut<'a>(&'a mut self) -> impl Iterator<Item=((Layer, Time), &'a mut Board)> {
        self.boards.iter_boards_mut()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardArray {
    // [(0; 0), (-1; 0), (1; 0), ..., (0; 1), (-1; 0), (1; 0), ...]
    boards: Vec<Option<Board>>,
    timeline_width: usize,
    max_time: isize,
    min_time: isize,
}

impl BoardArray {
    pub fn empty() -> Self {
        Self {
            boards: vec![None],
            timeline_width: 0,
            max_time: 0,
            min_time: 0,
        }
    }

    fn resize_boards(&mut self, l: usize, t: Time) {
        if l >= self.timeline_width * 2 + 1 || t < self.min_time {
            // We need to resize it all :)
            let timeline_width = l;
            let w = 2 * timeline_width + 1;
            let min_time = std::cmp::min(self.min_time, t);
            let max_time = std::cmp::max(self.max_time, t);

            let new_boards: Vec<Option<Board>> = vec![None; ((max_time - min_time) as usize + 1) * (2 * timeline_width + 1)];

            let old_boards = std::mem::replace(&mut self.boards, new_boards);

            for board in old_boards {
                if let Some(board) = board {
                    let l = if board.l() < 0 {
                        (-2 * board.l()) as usize + 1
                    } else {
                        2 * board.l() as usize
                    };

                    let t = (board.t() - min_time) as usize;

                    self.boards[l + t * w] = Some(board);
                }
            }

            self.timeline_width = timeline_width;
            self.min_time = min_time;
            self.max_time = max_time;
        } else {
            // We only need to extend the boards
            while t > self.max_time {
                self.max_time = (self.max_time - self.min_time + 1) * 2 + self.min_time;
            }
            self.boards.resize(((self.max_time - self.min_time) as usize + 1) * (2 * self.timeline_width + 1), None);
        }
    }

    #[inline]
    fn get_index(&self, l: Layer, t: Time) -> (usize, bool) {
        let l: usize = if l < 0 {
            (-2 * l) as usize + 1
        } else {
            2 * l as usize
        };

        let w: usize = self.timeline_width * 2 + 1;

        if l >= w || t < self.min_time || t > self.max_time {
            return (0, false)
        }

        let t: usize = (t - self.min_time) as usize;

        (l + t * w, true)
    }

    #[inline]
    fn get_index_unchecked(&self, l: Layer, t: Time) -> usize {
        let l: usize = if l < 0 {
            (-2 * l) as usize + 1
        } else {
            2 * l as usize
        };

        let w: usize = self.timeline_width * 2 + 1;

        let t: usize = (t - self.min_time) as usize;

        l + t * w
    }

    #[inline]
    pub fn get_board(&self, (l, t): (Layer, Time)) -> Option<&Board> {
        let (index, success) = self.get_index(l, t);

        if success {
            self.boards[index].as_ref()
        } else {
            None
        }
    }

    #[inline]
    pub fn get_board_mut(&mut self, (l, t): (Layer, Time)) -> Option<&mut Board> {
        let (index, success) = self.get_index(l, t);

        if success {
            self.boards[index].as_mut()
        } else {
            None
        }
    }

    #[inline]
    pub fn get_board_unchecked(&self, (l, t): (Layer, Time)) -> &Board {
        let index = self.get_index_unchecked(l, t);

        self.boards[index].as_ref().unwrap()
    }

    #[inline]
    pub fn insert_board(&mut self, board: Board) {
        let l: usize = if board.l() < 0 {
            (-2 * board.l()) as usize + 1
        } else {
            2 * board.l() as usize
        };

        if l >= self.timeline_width * 2 + 1 || board.t() < self.min_time || board.t() > self.max_time {
            self.resize_boards(l, board.t());
        }

        let w: usize = self.timeline_width * 2 + 1;
        let t: usize = (board.t() - self.min_time) as usize;

        self.boards[l + t * w] = Some(board);
    }

    #[inline]
    pub fn iter_boards<'a>(&'a self) -> impl Iterator<Item=((Layer, Time), &'a Board)> {
        self.boards.iter().filter_map(|b| {
            match b {
                Some(board) => Some(((board.l(), board.t()), board)),
                None => None
            }
        })
    }

    #[inline]
    pub fn iter_boards_mut<'a>(&'a mut self) -> impl Iterator<Item=((Layer, Time), &'a mut Board)> {
        self.boards.iter_mut().filter_map(|b| {
            match b {
                Some(board) => Some(((board.l(), board.t()), board)),
                None => None
            }
        })
    }

    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, Option<Board>> {
        self.boards.iter()
    }
}
