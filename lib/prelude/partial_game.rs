use super::*;
use std::collections::hash_map::{HashMap, Keys};
use std::hash::{Hash, Hasher};

/** Represents a "partial game state": the game state that a branch of a tree search algorithm needs to store.
    This structure is recursive to allow for recursive algorithms to perform better (by reducing the number of clones needed).
    It can be used in a non-recursive way by ommitting the `parent` field.

    The "boards" store a hashmap over an arbitrary data structure B. You may put any extension of `Board` in it!
**/
#[derive(Debug)]
pub struct PartialGame<'a, B: Clone + AsRef<Board>> {
    pub boards: HashMap<(Layer, Time), B>,
    pub info: Info,
    pub parent: Option<&'a PartialGame<'a, B>>,
}

impl<'a, B: Clone + AsRef<Board>> PartialGame<'a, B> {
    /** Creates a new PartialGame instance.
        Use this function if you are making use of the recursive data structure or you are initializing a new partial game state
    **/
    #[inline]
    pub fn new(
        boards: HashMap<(Layer, Time), B>,
        info: Info,
        parent: Option<&'a PartialGame<'a, B>>,
    ) -> Self {
        Self {
            boards,
            info,
            parent,
        }
    }

    /** Merges an already-existing PartialGame instance with a set of additional boards and info,
        yielding a new partial game state with all of the new boards.
        Use this function if you are using this data structure in a non-recursive way.
    **/
    pub fn merge(&self, boards: HashMap<(Layer, Time), B>, info: Info) -> Self {
        let mut tmp_boards = self.boards.clone();
        tmp_boards.extend(boards.into_iter());
        Self {
            boards: tmp_boards,
            info,
            parent: self.parent,
        }
    }

    #[inline]
    pub fn insert(&mut self, board: B) {
        self.boards
            .insert((board.as_ref().l(), board.as_ref().t()), board);
    }

    /** Merges all of the parent `PartialGame`s into itself, returning a now-`'static` PartialGame.
        This function clones boards, so do not use this function in hot-path code!
    **/
    pub fn flatten(self) -> PartialGame<'static, B> {
        let mut boards = self.boards;
        if let Some(parent) = self.parent {
            for board in parent.iter() {
                boards.insert((board.as_ref().l(), board.as_ref().t()), board.clone());
            }
            PartialGame {
                boards,
                info: self.info,
                parent: None,
            }
        } else {
            PartialGame {
                boards,
                info: self.info,
                parent: None,
            }
        }
    }

    /** Returns an iterator over all of the boards contained within that partial game state and its parents.
        That iterator yields objects of type `&B`.
        If you only wish to yield an iterator over the boards in this layer of partial game state, use `iter_shallow` instead.
    **/
    #[inline]
    pub fn iter(&'a self) -> PartialGameIter<'a, B> {
        PartialGameIter {
            partial_game: self,
            iter: self.boards.keys(),
        }
    }

    /** Returns an iterator over all of the boards contained within that partial game state, ignoring its parents.
        That iterator yields objects of type `&B`.
        If you also with to yield the boards of this partial game state's parent, use `iter` instead.
    **/
    #[inline]
    pub fn iter_shallow(&'a self) -> impl Iterator<Item = &'a B> {
        self.boards.values()
    }

    pub fn get_board<'b>(&'b self, coords: (Layer, Time)) -> Option<&'b B> {
        match self.boards.get(&coords) {
            Some(b) => Some(b),
            None => match self.parent {
                Some(parent) => parent.get_board(coords),
                None => None,
            },
        }
    }

    #[inline]
    pub fn get_board_with_game<'b>(
        &'b self,
        game: &'b Game,
        coords: (Layer, Time),
    ) -> Option<BoardOr<'b, B>> {
        match game.get_board(coords) {
            Some(b) => Some(BoardOr::Board(b)),
            None => self.get_board(coords).map(|b| BoardOr::B(b)),
        }
    }

    pub fn get_with_game<'b>(&'b self, game: &'b Game, coords: Coords) -> Tile {
        self.get_board_with_game(game, coords.non_physical())
            .map(|b| b.get(coords.physical()))
            .into()
    }

    #[inline]
    pub fn own_boards<'b>(&'b self, game: &'b Game) -> impl Iterator<Item = BoardOr<'b, B>> + 'b {
        self.info
            .timelines_white
            .iter()
            .chain(self.info.timelines_black.iter())
            .filter_map(move |tl| self.get_board_with_game(game, (tl.index, tl.last_board)))
            .filter(move |b| b.white() == self.info.active_player)
    }

    #[inline]
    pub fn opponent_boards<'b>(
        &'b self,
        game: &'b Game,
    ) -> impl Iterator<Item = BoardOr<'b, B>> + 'b {
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

    pub fn populate(&mut self, game: &'a Game) -> Option<()>
    where for<'b> B: PopulateBoard<'b, B>
    {
        for (_index, board) in &mut self.boards {
            board.populate(game, self.parent)?;
        }

        Some(())
    }
}

#[inline]
pub fn no_partial_game<'a>(game: &Game) -> PartialGame<'static, Board> {
    PartialGame::new(HashMap::new(), game.info.clone(), None)
}

impl<'a, B: Clone + AsRef<Board>> From<&'_ Game> for PartialGame<'a, B> {
    fn from(game: &'_ Game) -> Self {
        Self {
            boards: HashMap::new(),
            info: game.info.clone(),
            parent: None,
        }
    }
}

pub struct PartialGameIter<'a, B: Clone + AsRef<Board>> {
    pub partial_game: &'a PartialGame<'a, B>,
    pub iter: Keys<'a, (Layer, Time), B>,
}

impl<'a, B: Clone + AsRef<Board>> Iterator for PartialGameIter<'a, B> {
    type Item = &'a B;

    fn next(&'_ mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(coords) => Some(&self.partial_game.boards[coords]),
            None => match self.partial_game.parent {
                Some(parent) => {
                    self.partial_game = parent;
                    self.iter = self.partial_game.boards.keys();
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

impl<'a, B: Clone + AsRef<Board> + Hash> Hash for PartialGame<'a, B> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.info.hash(state);
        for board in self.iter() {
            board.hash(state);
        }
    }
}

impl<'a, 'b, B, C> std::convert::TryFrom<(&'b PartialGame<'b, B>, &'a Game)> for PartialGame<'a, C>
where
    B: Clone + AsRef<Board>,
    C: Clone + AsRef<Board>,
    for<'c> C: From<(B, &'c Game, &'c PartialGame<'c, C>)>,
{
    type Error = ();

    fn try_from((partial_game, game): (&'b PartialGame<'b, B>, &'a Game)) -> Result<Self, ()> {
        let mut res = Self {
            boards: HashMap::with_capacity(partial_game.boards.len()),
            info: partial_game.info.clone(),
            parent: match partial_game.parent {
                Some(_p) => return Err(()),
                None => None
            }
        };

        for (_index, board) in partial_game.boards.iter() {
            let new_board = C::from((board.clone(), game, &res));
            res.boards.insert((board.as_ref().l(), board.as_ref().t()), new_board);
        }

        Ok(res)
    }
}
