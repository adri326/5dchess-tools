use super::*;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq)]
pub enum PartialGameStorage {
    Shallow(Vec<Option<Board>>, Vec<Option<Board>>),
    Deep(BoardArray),
}

/** Represents a "partial game state": the game state that a branch of a tree search algorithm needs to store.
    This structure is recursive to allow for recursive algorithms to perform better (by reducing the number of clones needed).
    It can be used in a non-recursive way by ommitting the `parent` field.
**/
#[derive(Clone, Debug, PartialEq)]
pub struct PartialGame<'a> {
    boards: PartialGameStorage,
    pub info: Info,
    pub parent: Option<&'a PartialGame<'a>>,
}

impl<'a> PartialGame<'a> {
    /** Creates a new PartialGame instance.
        Use this function if you are making use of the recursive data structure or you are initializing a new partial game state
    **/
    #[inline]
    pub fn new(
        boards: PartialGameStorage,
        info: Info,
        parent: Option<&'a PartialGame<'a>>,
    ) -> Self {
        Self {
            boards,
            info,
            parent,
        }
    }

    /** Creates an empty PartialGame instance. **/
    #[inline]
    pub fn empty(
        info: Info,
        parent: Option<&'a PartialGame<'a>>,
    ) -> Self {
        Self {
            boards: PartialGameStorage::Shallow(
                vec![None; info.timelines_white.len()],
                vec![None; info.timelines_black.len()]
            ),
            info,
            parent,
        }
    }

    /**
        Overwrites the `boards` field.
    **/
    #[inline]
    pub fn set_boards(&mut self, boards: PartialGameStorage) {
        self.boards = boards;
    }

    pub fn insert(&mut self, board: Board) {
        match &mut self.boards {
            PartialGameStorage::Shallow(boards_white, boards_black) => {
                if board.l() >= 0 {
                    let index = board.l() as usize;
                    if boards_white.len() <= index {
                        while boards_white.len() < index {
                            boards_white.push(None);
                        }
                        boards_white.push(Some(board));
                    } else if boards_white[index].is_none() {
                        boards_white[index] = Some(board);
                    } else {
                        self.deepen();
                        self.insert(board);
                    }
                } else {
                    let index = -board.l() as usize - 1;
                    if boards_black.len() <= index {
                        while boards_black.len() < index {
                            boards_black.push(None);
                        }
                        boards_black.push(Some(board));
                    } else if boards_black[index].is_none() {
                        boards_black[index] = Some(board);
                    } else {
                        self.deepen();
                        self.insert(board);
                    }
                }
            }
            PartialGameStorage::Deep(boards) => {
                boards.insert_board(board);
            }
        }
    }

    /** Merges all of the parent `PartialGame`s into itself, returning a now-`'static` PartialGame.
        This function clones boards, so do not use this function in hot-path code!
    **/
    pub fn flatten(mut self) -> PartialGame<'static> {
        if let Some(parent) = self.parent {
            // Kind of an oxymoron
            self.deepen();

            let mut boards = match self.boards {
                PartialGameStorage::Deep(boards) => boards,
                PartialGameStorage::Shallow(_, _) => {
                    unsafe {
                        std::hint::unreachable_unchecked()
                    }
                }
            };

            for board in parent.iter() {
                boards.insert_board(board.clone());
            }

            PartialGame {
                boards: PartialGameStorage::Deep(boards),
                info: self.info,
                parent: None,
            }
        } else {
            PartialGame {
                boards: self.boards,
                info: self.info,
                parent: None,
            }
        }
    }

    /** If `self.boards` is of PartialGameStorage::Shallow, then turns it into a PartialGameStorage::Deep; otherwise does nothing. **/
    pub fn deepen(&mut self) {
        match self.boards {
            PartialGameStorage::Shallow(_, _) => {},
            PartialGameStorage::Deep(_) => return,
        }

        let boards: PartialGameStorage = PartialGameStorage::Deep(BoardArray::empty());
        let boards = std::mem::replace(&mut self.boards, boards);

        let target = match &mut self.boards {
            PartialGameStorage::Shallow(_, _) => {
                unreachable!();
            },
            PartialGameStorage::Deep(boards) => boards,
        };

        match boards {
            PartialGameStorage::Shallow(boards_white, boards_black) => {
                for board in boards_white {
                    if let Some(board) = board {
                        target.insert_board(board);
                    }
                }
                for board in boards_black {
                    if let Some(board) = board {
                        target.insert_board(board);
                    }
                }
            },
            PartialGameStorage::Deep(_) => {
                unreachable!();
            }
        }
    }

    /** Returns an iterator over all of the boards contained within that partial game state and its parents.
        That iterator yields objects of type `&Board`.
        If you only wish to yield an iterator over the boards in this layer of partial game state, use `iter_shallow` instead.
    **/
    #[inline]
    pub fn iter(&'a self) -> PartialGameIter<'a> {
        PartialGameIter {
            partial_game: self,
            iter: self.boards.iter(),
        }
    }

    /** Returns an iterator over all of the boards contained within that partial game state, ignoring its parents.
        That iterator yields objects of type `&Board`.
        If you also with to yield the boards of this partial game state's parent, use `iter` instead.
    **/
    #[inline]
    pub fn iter_shallow(&'a self) -> impl Iterator<Item = &'a Board> {
        self.boards.iter()
    }

    pub fn get_board<'b>(&'b self, coords: (Layer, Time)) -> Option<&'b Board> {
        match &self.boards {
            PartialGameStorage::Shallow(boards_white, boards_black) => {
                if let Some(tl) = self.info.get_timeline(coords.0) {
                    if coords.1 == tl.last_board {
                        if coords.0 >= 0 {
                            match &boards_white[coords.0 as usize] {
                                Some(board) => return Some(board),
                                None => {}
                            }
                        } else {
                            match &boards_black[-coords.0 as usize - 1] {
                                Some(board) => return Some(board),
                                None => {}
                            }
                        }
                    }
                    match self.parent {
                        Some(parent) => parent.get_board(coords),
                        None => None,
                    }
                } else {
                    // Change this to look for a timeline in the parent if timelines can be collapsed/deleted
                    None
                }
            }
            PartialGameStorage::Deep(boards) => {
                match boards.get_board(coords) {
                    Some(b) => Some(b),
                    None => match self.parent {
                        Some(parent) => parent.get_board(coords),
                        None => None,
                    },
                }
            }
        }
    }

    #[inline]
    pub fn get_board_with_game<'b>(
        &'b self,
        game: &'b Game,
        coords: (Layer, Time),
    ) -> Option<&'b Board> {
        match game.get_board(coords) {
            Some(b) => Some(b),
            None => self.get_board(coords),
        }
    }

    #[inline]
    pub fn get_with_game<'b>(&'b self, game: &'b Game, coords: Coords) -> Tile {
        match self.get_board_with_game(game, coords.non_physical()) {
            Some(board) => board.get(coords.physical()),
            None => Tile::Void,
        }
    }

    #[inline]
    pub fn own_boards<'b>(&'b self, game: &'b Game) -> impl Iterator<Item = &'b Board> + 'b {
        self.info
            .timelines_white
            .iter()
            .chain(self.info.timelines_black.iter())
            .filter_map(move |tl| self.get_board_with_game(game, (tl.index, tl.last_board)))
            .filter(move |b| b.white() == self.info.active_player)
    }

    #[inline]
    pub fn opponent_boards<'b>(&'b self, game: &'b Game) -> impl Iterator<Item = &'b Board> + 'b {
        self.info
            .timelines_white
            .iter()
            .chain(self.info.timelines_black.iter())
            .filter_map(move |tl| self.get_board_with_game(game, (tl.index, tl.last_board)))
            .filter(move |b| b.white() != self.info.active_player)
    }

    #[inline]
    pub fn is_last_board(&self, coords: (Layer, Time)) -> Option<bool> {
        match self.info.get_timeline(coords.0) {
            Some(tl) => Some(tl.last_board == coords.1),
            None => None,
        }
    }
}

#[inline]
pub fn no_partial_game<'a>(game: &Game) -> PartialGame<'static> {
    PartialGame::empty(game.info.clone(), None)
}

impl<'a> From<&'_ Game> for PartialGame<'a> {
    #[inline]
    fn from(game: &'_ Game) -> Self {
        PartialGame::empty(
            game.info.clone(),
            None,
        )
    }
}

// Iterator for PartialGameStorage

pub enum PartialGameStorageIter<'a> {
    Shallow(std::iter::Chain<std::slice::Iter<'a, Option<Board>>, std::slice::Iter<'a, Option<Board>>>),
    Deep(std::slice::Iter<'a, Option<Board>>)
}

impl PartialGameStorage {
    #[inline]
    pub fn iter<'a>(&'a self) -> PartialGameStorageIter<'a> {
        match self {
            PartialGameStorage::Shallow(boards_white, boards_black) => {
                PartialGameStorageIter::Shallow(boards_white.iter().chain(boards_black.iter()))
            },
            PartialGameStorage::Deep(boards) => {
                PartialGameStorageIter::Deep(boards.iter())
            }
        }
    }
}

impl<'a> Iterator for PartialGameStorageIter<'a> {
    type Item = &'a Board;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            PartialGameStorageIter::Shallow(iter) => {
                for b in iter {
                    if let Some(board) = b {
                        return Some(board)
                    }
                }
                None
            },
            PartialGameStorageIter::Deep(iter) => {
                for b in iter {
                    if let Some(board) = b {
                        return Some(board)
                    }
                }
                None
            }
        }
    }
}

// Recursive iterator for PartialGame

pub struct PartialGameIter<'a> {
    pub partial_game: &'a PartialGame<'a>,
    pub iter: PartialGameStorageIter<'a>,
}

impl<'a> Iterator for PartialGameIter<'a> {
    type Item = &'a Board;

    #[inline]
    fn next(&'_ mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(board) => Some(board),
            None => match self.partial_game.parent {
                Some(parent) => {
                    self.partial_game = parent;
                    self.iter = self.partial_game.boards.iter();
                    self.next()
                }
                None => None,
            },
        }
    }

    fn size_hint(&'_ self) -> (usize, Option<usize>) {
        if let None = self.partial_game.parent {
            self.iter.size_hint()
        } else {
            (self.iter.size_hint().0, None)
        }
    }
}

impl<'a> Hash for PartialGame<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.info.hash(state);
        for board in self.iter() {
            board.hash(state);
        }
    }
}
