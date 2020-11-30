use super::game::*;
use std::collections::HashMap;
use std::rc::Rc;

/// A generic "virtual boardset"; it is used to hold the state of a branch (while searching the move tree) without making unnecessary and expensive copies of `Game`
/// The way the additional, "virtual" boards (as part of the state of a branch) are stored is implementation-dependent.
pub trait VirtualBoardset<'a> {
    /// Crates a new virtual boardset from a game instance and a set of boards
    fn new(game: &'a Game, info: GameInfo, boards: Vec<Board>) -> Self;

    /// Returns the board at (l, t) or None if none was found.
    /// If `(l, t)` corresponds to a board within the game instance, then that board should be returned.
    fn get_board<'b>(&'b self, l: i32, t: isize) -> Option<&'b Board>;

    /// Returns the tile at (l, t) or None if the board isn't found or the tile is out of bounds
    fn get(&self, l: i32, t: isize, x: u8, y: u8) -> Option<Piece> {
        self.get_board(l, t).map(|b| b.get(x, y)).flatten()
    }

    /// Returns an iterator over the virtual boards stored within the virtual boardset
    fn virtual_boards<'b>(&'b self) -> Box<dyn Iterator<Item=Board> + '_>;

    /// Returns the underlying Game instance
    fn game(&self) -> &'a Game;

    fn info(&self) -> GameInfo;

    /// Appends a set of boards to a boardset.
    /// It is expected that `∀l ∀t, (a.get_board(l, t) is Some) => (push(a, ...).get_board(l, t) is Some)` (ie. `a ⊂ push(a, ...)`)
    fn push(&self, info: GameInfo, boards: Vec<Board>) -> Self;
}

pub fn empty<'b>(game: &'b Game) -> EmptyVirtualBoardset<'b> {
    EmptyVirtualBoardset::new(game, game.info, vec![])
}

#[derive(Debug, Clone)]
pub struct SimpleVirtualBoardset<'a> {
    pub game: &'a Game,
    pub info: GameInfo,
    pub virtual_boards: HashMap<(i32, isize), Board>,
}

impl<'a> VirtualBoardset<'a> for SimpleVirtualBoardset<'a> {
    fn new(game: &'a Game, info: GameInfo, boards: Vec<Board>) -> Self {
        let mut res = Self {
            game,
            info,
            virtual_boards: HashMap::with_capacity(boards.len()),
        };

        for board in boards.into_iter() {
            res.virtual_boards.insert((board.l, board.t), board);
        }

        res
    }

    fn get_board<'b>(&'b self, l: i32, t: isize) -> Option<&'b Board> {
        self.game.get_board(l, t).or_else(|| {
            self.virtual_boards.get(&(l, t))
        })
    }

    fn push<'b>(&'b self, info: GameInfo, boards: Vec<Board>) -> Self {
        let mut res = self.clone();

        res.info = info;
        for board in boards.into_iter() {
            res.virtual_boards.insert((board.l, board.t), board);
        }

        res
    }

    fn game(&self) -> &'a Game {
        self.game
    }

    fn info(&self) -> GameInfo {
        self.info
    }

    fn virtual_boards<'b>(&'b self) -> Box<dyn Iterator<Item=Board> + '_> {
        Box::new(self.virtual_boards.values().cloned())
    }
}

#[derive(Debug, Clone)]
pub struct RecursiveVirtualBoardsetRaw<'a> {
    pub game: &'a Game,
    pub info: GameInfo,
    pub virtual_boards: HashMap<(i32, isize), Board>,
    pub parent: Option<Rc<RecursiveVirtualBoardsetRaw<'a>>>,
}

#[derive(Debug, Clone)]
pub struct RecursiveVirtualBoardset<'a>(Rc<RecursiveVirtualBoardsetRaw<'a>>);

impl<'a> VirtualBoardset<'a> for RecursiveVirtualBoardset<'a> {
    fn new(game: &'a Game, info: GameInfo, boards: Vec<Board>) -> Self {
        let mut res = RecursiveVirtualBoardsetRaw {
            game,
            info,
            virtual_boards: HashMap::with_capacity(boards.len()),
            parent: None,
        };

        for board in boards.into_iter() {
            res.virtual_boards.insert((board.l, board.t), board);
        }

        RecursiveVirtualBoardset(Rc::new(res))
    }

    fn get_board<'b>(&'b self, l: i32, t: isize) -> Option<&'b Board> {
        self.0.game.get_board(l, t).or_else(|| {
            self.0.virtual_boards.get(&(l, t))
        })
    }

    fn push(&self, info: GameInfo, boards: Vec<Board>) -> Self {
        let mut res = RecursiveVirtualBoardsetRaw {
            game: self.0.game,
            info: info,
            virtual_boards: HashMap::with_capacity(boards.len()),
            parent: Some(self.0.clone()),
        };

        for board in boards.into_iter() {
            res.virtual_boards.insert((board.l, board.t), board);
        }

        RecursiveVirtualBoardset(Rc::new(res))
    }

    fn game(&self) -> &'a Game {
        self.0.game
    }

    fn info(&self) -> GameInfo {
        self.0.info
    }

    fn virtual_boards<'b>(&'b self) -> Box<dyn Iterator<Item=Board> + '_> {
        Box::new(RecursiveVirtualBoardsetIter {
            rvb: self.0.clone(),
            vec: collect(self.0.clone()),
            index: 0,
        })
    }
}

pub struct RecursiveVirtualBoardsetIter<'a> {
    pub rvb: Rc<RecursiveVirtualBoardsetRaw<'a>>,
    pub vec: Vec<(i32, isize)>,
    pub index: usize,
}

impl<'a> Iterator for RecursiveVirtualBoardsetIter<'a> {
    type Item = Board;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.vec.len() - self.index, None)
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        match self.vec.get(self.index - 1) {
            Some(key) => Some(self.rvb.virtual_boards[key].clone()),
            None => {
                match self.rvb.parent.clone() {
                    Some(parent) => {
                        self.rvb = parent.clone();
                        self.vec = collect(parent);
                        self.index = 0;
                        self.next()
                    },
                    None => None
                }
            }
        }
    }
}

fn collect<'a>(hm: Rc<RecursiveVirtualBoardsetRaw<'a>>) -> Vec<(i32, isize)> {
    let mut res = Vec::with_capacity(hm.virtual_boards.len());
    for k in hm.virtual_boards.keys() {
        res.push(*k);
    }
    res
}

pub struct EmptyVirtualBoardset<'a>(&'a Game, GameInfo);

impl<'a> VirtualBoardset<'a> for EmptyVirtualBoardset<'a> {
    fn new(game: &'a Game, info: GameInfo, _boards: Vec<Board>) -> Self {
        Self(game, info)
    }

    fn get_board<'b>(&'b self, l: i32, t: isize) -> Option<&'b Board> {
        self.0.get_board(l, t)
    }

    fn push(&self, info: GameInfo, _boards: Vec<Board>) -> Self {
        Self(self.0, info)
    }

    fn game(&self) -> &'a Game {
        self.0
    }

    fn info(&self) -> GameInfo {
        self.1
    }

    fn virtual_boards<'b>(&'b self) -> Box<dyn Iterator<Item=Board> + '_> {
        Box::new(std::iter::empty())
    }
}

impl<'a> From<EmptyVirtualBoardset<'a>> for SimpleVirtualBoardset<'a> {
    fn from(empty: EmptyVirtualBoardset<'a>) -> Self {
        Self::new(empty.0, empty.1, vec![])
    }
}

impl<'a> From<&'a EmptyVirtualBoardset<'a>> for SimpleVirtualBoardset<'a> {
    fn from(empty: &'a EmptyVirtualBoardset<'a>) -> Self {
        Self::new(empty.0, empty.1, vec![])
    }
}

impl<'a> From<EmptyVirtualBoardset<'a>> for RecursiveVirtualBoardset<'a> {
    fn from(empty: EmptyVirtualBoardset<'a>) -> Self {
        <Self as VirtualBoardset>::new(empty.0, empty.1, vec![])
    }
}

impl<'a, 'b> From<&'b EmptyVirtualBoardset<'a>> for RecursiveVirtualBoardset<'a> {
    fn from(empty: &'b EmptyVirtualBoardset<'a>) -> Self {
        <Self as VirtualBoardset>::new(empty.0, empty.1, vec![])
    }
}

impl<'a> From<SimpleVirtualBoardset<'a>> for RecursiveVirtualBoardset<'a> {
    fn from(simple: SimpleVirtualBoardset<'a>) -> Self {
        RecursiveVirtualBoardset(Rc::new(RecursiveVirtualBoardsetRaw {
            game: simple.game,
            info: simple.info,
            virtual_boards: simple.virtual_boards,
            parent: None,
        }))
    }
}

impl<'a, 'b> From<&'b SimpleVirtualBoardset<'a>> for RecursiveVirtualBoardset<'a> {
    fn from(simple: &'b SimpleVirtualBoardset<'a>) -> Self {
        RecursiveVirtualBoardset(Rc::new(RecursiveVirtualBoardsetRaw {
            game: simple.game,
            info: simple.info,
            virtual_boards: simple.virtual_boards.clone(),
            parent: None,
        }))
    }
}

impl<'a> From<RecursiveVirtualBoardset<'a>> for SimpleVirtualBoardset<'a> {
    fn from(rec: RecursiveVirtualBoardset<'a>) -> Self {
        let mut res = SimpleVirtualBoardset {
            game: rec.game(),
            info: rec.info(),
            virtual_boards: HashMap::with_capacity(rec.0.virtual_boards.len())
        };

        for vb in rec.virtual_boards() {
            res.virtual_boards.insert((vb.l, vb.t), vb.clone());
        }

        res
    }
}
