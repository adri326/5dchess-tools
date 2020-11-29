use super::game::*;
use std::collections::HashMap;

/// A generic "virtual boardset"; it is used to hold the state of a branch (while searching the move tree) without making unnecessary and expensive copies of `Game`
/// The way the additional, "virtual" boards (as part of the state of a branch) are stored is implementation-dependent.
pub trait VirtualBoardset<'a> {
    /// Crates a new virtual boardset from a game instance and a set of boards
    fn new(game: &'a Game, boards: Vec<Board>) -> Self;

    /// Returns the board at (l, t) or None if none was found.
    /// If `(l, t)` corresponds to a board within the game instance, then that board should be returned.
    fn get_board(&'a self, l: i32, t: isize) -> Option<&'a Board>;

    /// Returns the tile at (l, t) or None if the board isn't found or the tile is out of bounds
    fn get(&'a self, l: i32, t: isize, x: u8, y: u8) -> Option<Piece> {
        self.get_board(l, t).map(|b| b.get(x, y)).flatten()
    }

    /// Returns an iterator over the virtual boards stored within the virtual boardset
    fn virtual_boards(&'a self) -> Box<dyn Iterator<Item=&'a Board> + '_>;

    /// Returns the underlying Game instance
    fn game(&'a self) -> &'a Game;

    /// Appends a set of boards to a boardset.
    /// It is expected that `∀l ∀t, (a.get_board(l, t) is Some) => (push(a, ...).get_board(l, t) is Some)` (ie. `a ⊂ push(a, ...)`)
    fn push(&'a self, boards: Vec<Board>) -> Self;
}

pub fn empty<'b>(game: &'b Game) -> EmptyVirtualBoardset<'b> {
    EmptyVirtualBoardset::new(game, vec![])
}

#[derive(Debug, Clone)]
pub struct SimpleVirtualBoardset<'a> {
    pub game: &'a Game,
    pub virtual_boards: HashMap<(i32, isize), Board>,
}

impl<'a> VirtualBoardset<'a> for SimpleVirtualBoardset<'a> {
    fn new(game: &'a Game, boards: Vec<Board>) -> Self {
        let mut res = Self {
            game,
            virtual_boards: HashMap::with_capacity(boards.len()),
        };

        for board in boards.into_iter() {
            res.virtual_boards.insert((board.l, board.t), board);
        }

        res
    }

    fn get_board(&'a self, l: i32, t: isize) -> Option<&'a Board> {
        self.game.get_board(l, t).or_else(|| {
            self.virtual_boards.get(&(l, t))
        })
    }

    fn push(&'a self, boards: Vec<Board>) -> Self {
        let mut res = self.clone();
        for board in boards.into_iter() {
            res.virtual_boards.insert((board.l, board.t), board);
        }

        res
    }

    fn game(&'a self) -> &'a Game {
        self.game
    }

    fn virtual_boards(&'a self) -> Box<dyn Iterator<Item=&'a Board> + '_> {
        Box::new(self.virtual_boards.values())
    }
}

#[derive(Debug, Clone)]
pub struct RecursiveVirtualBoardset<'a> {
    pub game: &'a Game,
    pub virtual_boards: HashMap<(i32, isize), Board>,
    pub parent: Option<&'a RecursiveVirtualBoardset<'a>>,
}

impl<'a> VirtualBoardset<'a> for RecursiveVirtualBoardset<'a> {
    fn new(game: &'a Game, boards: Vec<Board>) -> Self {
        let mut res = Self {
            game,
            virtual_boards: HashMap::with_capacity(boards.len()),
            parent: None,
        };

        for board in boards.into_iter() {
            res.virtual_boards.insert((board.l, board.t), board);
        }

        res
    }

    fn get_board(&'a self, l: i32, t: isize) -> Option<&'a Board> {
        self.game.get_board(l, t).or_else(|| {
            self.virtual_boards.get(&(l, t))
        })
    }

    fn push(&'a self, boards: Vec<Board>) -> Self {
        let mut res = RecursiveVirtualBoardset {
            game: self.game,
            virtual_boards: HashMap::with_capacity(boards.len()),
            parent: Some(self),
        };

        for board in boards.into_iter() {
            res.virtual_boards.insert((board.l, board.t), board);
        }

        res
    }

    fn game(&'a self) -> &'a Game {
        self.game
    }

    fn virtual_boards(&'a self) -> Box<dyn Iterator<Item=&'a Board> + '_> {
        Box::new(RecursiveVirtualBoardsetIter {
            rvb: Some(self),
            iter: self.virtual_boards.values(),
        })
    }
}

pub struct RecursiveVirtualBoardsetIter<'a> {
    pub rvb: Option<&'a RecursiveVirtualBoardset<'a>>,
    pub iter: std::collections::hash_map::Values<'a, (i32, isize), Board>,
}

impl<'a> Iterator for RecursiveVirtualBoardsetIter<'a> {
    type Item = &'a Board;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.iter.size_hint().0, None)
    }

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(b) => Some(b),
            None => {
                if self.rvb.is_some() {
                    self.rvb = self.rvb.unwrap().parent;
                    if let Some(rvb) = self.rvb {
                        self.iter = rvb.virtual_boards.values();
                    }
                    self.next()
                } else {
                    None
                }
            }
        }
    }
}

pub struct EmptyVirtualBoardset<'a>(&'a Game);

impl<'a> VirtualBoardset<'a> for EmptyVirtualBoardset<'a> {
    fn new(game: &'a Game, _boards: Vec<Board>) -> Self {
        Self(game)
    }

    fn get_board(&'a self, l: i32, t: isize) -> Option<&'a Board> {
        self.0.get_board(l, t)
    }

    fn push(&'a self, _boards: Vec<Board>) -> Self {
        Self(self.0)
    }

    fn game(&'a self) -> &'a Game {
        self.0
    }

    fn virtual_boards(&'a self) -> Box<dyn Iterator<Item=&'a Board> + '_> {
        Box::new(std::iter::empty())
    }
}

impl<'a> From<EmptyVirtualBoardset<'a>> for SimpleVirtualBoardset<'a> {
    fn from(empty: EmptyVirtualBoardset<'a>) -> Self {
        Self::new(empty.0, vec![])
    }
}

impl<'a> From<&'a EmptyVirtualBoardset<'a>> for SimpleVirtualBoardset<'a> {
    fn from(empty: &'a EmptyVirtualBoardset<'a>) -> Self {
        Self::new(empty.0, vec![])
    }
}

impl<'a> From<EmptyVirtualBoardset<'a>> for RecursiveVirtualBoardset<'a> {
    fn from(empty: EmptyVirtualBoardset<'a>) -> Self {
        Self::new(empty.0, vec![])
    }
}

impl<'a> From<&'a EmptyVirtualBoardset<'a>> for RecursiveVirtualBoardset<'a> {
    fn from(empty: &'a EmptyVirtualBoardset<'a>) -> Self {
        Self::new(empty.0, vec![])
    }
}

impl<'a> From<SimpleVirtualBoardset<'a>> for RecursiveVirtualBoardset<'a> {
    fn from(simple: SimpleVirtualBoardset<'a>) -> Self {
        Self {
            game: simple.game,
            virtual_boards: simple.virtual_boards,
            parent: None,
        }
    }
}

impl<'a> From<&'a SimpleVirtualBoardset<'a>> for RecursiveVirtualBoardset<'a> {
    fn from(simple: &'a SimpleVirtualBoardset<'a>) -> Self {
        Self {
            game: simple.game,
            virtual_boards: simple.virtual_boards.clone(),
            parent: None,
        }
    }
}
