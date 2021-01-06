use super::*;
use std::borrow::Borrow;
use std::collections::hash_map::{HashMap, Keys};
use std::fmt;
use std::hash::Hash;

/** Represents a "partial game state": the game state that a branch of a tree search algorithm needs to store.
    This structure is recursive to allow for recursive algorithms to perform better (by reducing the number of clones needed).
    It can be used in a non-recursive way by ommitting the `parent` field.

    The "boards" store a hashmap over an arbitrary data structure B. You may put any extension of `Board` in it!
**/
pub struct PartialGame<'a, B: Clone + 'a> {
    pub boards: HashMap<(Layer, Time), B>,
    pub info: Info,
    pub parent: Option<&'a PartialGame<'a, B>>,
}

impl<'a, B: Clone + 'a> PartialGame<'a, B> {
    /** Creates a new PartialGame instance.
        Use this function if you are making use of the recursive data structure or you are initializing a new partial game state
    **/
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

    /** Toggles the `active_player` variable **/
    pub fn swap_player(mut self) -> Self {
        self.info.active_player = !self.info.active_player;
        self
    }

    /** Returns an iterator over all of the boards contained within that partial game state and its parents.
        That iterator yields objects of type `PartialGameRef`, which implement `AsRef<B>`.
        If you only wish to yield an iterator over the boards in this layer of partial game state, use `iter_shallow` instead.
    **/
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
}

impl<'a, B: Clone + 'a> From<&'_ Game> for PartialGame<'a, B> {
    fn from(game: &'_ Game) -> Self {
        Self {
            boards: HashMap::new(),
            info: game.info.clone(),
            parent: None,
        }
    }
}

pub struct PartialGameIter<'a, B: Clone + 'a> {
    pub partial_game: &'a PartialGame<'a, B>,
    pub iter: Keys<'a, (Layer, Time), B>,
}

impl<'a, B: Clone + 'a> Iterator for PartialGameIter<'a, B> {
    type Item = PartialGameRef<'a, B>;

    fn next(&'_ mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(coords) => Some(PartialGameRef {
                coords: *coords,
                partial_game: self.partial_game,
            }),
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

pub struct PartialGameRef<'a, B: Clone + 'a> {
    pub coords: (Layer, Time),
    pub partial_game: &'a PartialGame<'a, B>,
}

impl<'a, B: Clone + 'a> AsRef<B> for PartialGameRef<'a, B> {
    fn as_ref<'b>(&'b self) -> &'b B {
        &self.partial_game.boards[&self.coords]
    }
}

impl<'a, B: Clone + fmt::Debug + 'a> fmt::Debug for PartialGameRef<'a, B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("PartialGameRef")
            .field(self.as_ref() as &B)
            .finish()
    }
}

impl<'a, B: Clone + 'a> Clone for PartialGameRef<'a, B> {
    /** Does NOT clone the underlying partial_game **/
    fn clone(&self) -> Self {
        PartialGameRef {
            coords: self.coords,
            partial_game: self.partial_game,
        }
    }
}
