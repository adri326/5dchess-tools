/*
    Structures and functions related to the game's state.
*/

use super::*;

/**
    Structure holding the main game state (boards, width, height, timeline information, current player, etc.)
**/
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    pub boards: BoardArray,
    pub width: Physical,
    pub height: Physical,
    pub info: Info,
}

impl Game {
    /// Creates a new `Game` instance.
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

    /// Gets the board stored within the current `Game` instance at `(l, t)`
    #[inline]
    pub fn get_board(&self, (l, t): (Layer, Time)) -> Option<&Board> {
        self.boards.get_board((l, t))
    }

    /// Gets a mutable reference to the board stored within the current `Game` instance at `(l, t)`
    #[inline]
    pub fn get_board_mut(&mut self, (l, t): (Layer, Time)) -> Option<&mut Board> {
        self.boards.get_board_mut((l, t))
    }

    /// Gets the board stored within the current `Game` instance at `(l, t)`, without checking that the coordinates are valid.
    /// Only use this if you know that the board is present
    #[inline]
    pub fn get_board_unchecked(&self, (l, t): (Layer, Time)) -> &Board {
        self.boards.get_board_unchecked((l, t))
    }

    /// Gets the tile at the coordinates `(l, t, x, y)`
    #[inline]
    pub fn get(&self, Coords(l, t, x, y): Coords) -> Tile {
        self.get_board((l, t)).map(|b| b.get((x, y))).into()
    }

    /// Gets the tile at the coordinates `(l, t, x, y)`, without checking that the coordinates are valid.
    /// Only use this if you know that the corresponding board is present and that the `(x, y)` coordinates lie within that board.
    #[inline]
    pub fn get_unchecked(&self, Coords(l, t, x, y): Coords) -> Tile {
        self.boards.get_board_unchecked((l, t)).get_unchecked((x, y))
    }

    /// Inserts the given board into the game. The coordinates of it are deduced from that board's coordinates.
    #[inline]
    pub fn insert_board(&mut self, board: Board) {
        self.boards.insert_board(board)
    }

    /// Iterate over the boards within that game instance
    #[inline]
    pub fn iter_boards<'a>(&'a self) -> impl Iterator<Item=((Layer, Time), &'a Board)> {
        self.boards.iter_boards()
    }

    /// Returns an iterator yielding mutable references of the boards within that game instance.
    #[inline]
    pub fn iter_boards_mut<'a>(&'a mut self) -> impl Iterator<Item=((Layer, Time), &'a mut Board)> {
        self.boards.iter_boards_mut()
    }
}

/// A growable, contiguous structure holding a set of boards.
/// Indexing is done in O(1) time and is much faster than what a hashmap can do.
/// Appending a board at the end of a timeline is done in O(1) time in the best scenario.
/// Creating a new timeline is done in O(n) time, with `n` being the number of boards already present in the `BoardArray`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardArray {
    // [(0; 0), (-1; 0), (1; 0), ..., (0; 1), (-1; 0), (1; 0), ...]
    boards: Vec<Option<Board>>,
    timeline_width: usize,
    max_time: isize,
    min_time: isize,
}

impl BoardArray {
    /// Creates a new, empty BoardArray instance
    pub fn empty() -> Self {
        Self {
            boards: vec![None],
            timeline_width: 0,
            max_time: 0,
            min_time: 0,
        }
    }

    /// Grows the internal vector so that a board in (~l, t) would fit in it.
    /// l corresponds to a value of `Layer` mapped to ℕ
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

    /// Gets the index for a board at `(l, t)`; returns `(index, true)` if the index is found and `(0, false)` otherwise.
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

    /// Gets the index for a board at `(l, t)`, ignoring whether or not that index is out of bounds
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

    /// Gets the board at `(l, t)`, returning `None` if that board is absent
    #[inline]
    pub fn get_board(&self, (l, t): (Layer, Time)) -> Option<&Board> {
        let (index, success) = self.get_index(l, t);

        if success {
            self.boards[index].as_ref()
        } else {
            None
        }
    }

    /// Gets a mutable reference to the board at `(l, t)`, returning `None` if that board is absent
    #[inline]
    pub fn get_board_mut(&mut self, (l, t): (Layer, Time)) -> Option<&mut Board> {
        let (index, success) = self.get_index(l, t);

        if success {
            self.boards[index].as_mut()
        } else {
            None
        }
    }

    /// Gets the board at `(l, t)`; panics if the board is not found
    #[inline]
    pub fn get_board_unchecked(&self, (l, t): (Layer, Time)) -> &Board {
        let index = self.get_index_unchecked(l, t);

        self.boards[index].as_ref().unwrap()
    }

    /// Inserts a board; if the board already fits, then
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

    /// Returns an iterator over the boards
    #[inline]
    pub fn iter_boards<'a>(&'a self) -> impl Iterator<Item=((Layer, Time), &'a Board)> {
        self.boards.iter().filter_map(|b| {
            match b {
                Some(board) => Some(((board.l(), board.t()), board)),
                None => None
            }
        })
    }

    /// Returns an iterator over the boards, yielding mutable references to these boards
    #[inline]
    pub fn iter_boards_mut<'a>(&'a mut self) -> impl Iterator<Item=((Layer, Time), &'a mut Board)> {
        self.boards.iter_mut().filter_map(|b| {
            match b {
                Some(board) => Some(((board.l(), board.t()), board)),
                None => None
            }
        })
    }

    /// Returns an iterator over the internal board buffer
    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, Option<Board>> {
        self.boards.iter()
    }

    /// Turns the internal board buffer into an iterator
    pub fn into_iter(self) -> std::vec::IntoIter<Option<Board>> {
        self.boards.into_iter()
    }
}
