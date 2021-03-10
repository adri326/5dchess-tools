//! Tree search algorithms and utilities
use crate::prelude::*;

pub mod schedule;
pub use schedule::{Tasks};

pub mod dfs;
pub use dfs::*;

/**
    A node in a tree search
**/
#[derive(Clone, Debug)]
pub struct TreeNode<'a> {
    pub partial_game: PartialGame<'a>,
    pub path: Vec<Moveset>,
    pub branches: usize,
}

impl<'a> PartialEq for TreeNode<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl<'a> Eq for TreeNode<'a> {}

impl<'a> TreeNode<'a> {
    pub fn new(partial_game: PartialGame<'a>, path: Vec<Moveset>, branches: usize) -> Self {
        Self {
            partial_game,
            path,
            branches,
        }
    }

    pub fn extend(parent: &TreeNode<'a>, moveset: Moveset, partial_game: PartialGame<'a>) -> Self {
        let mut new_path = parent.path.clone();
        new_path.push(moveset);
        let branches = parent.branches + partial_game.info.len_timelines() - parent.partial_game.info.len_timelines();

        Self {
            partial_game,
            path: new_path,
            branches,
        }
    }
}

impl TreeNode<'static> {
    pub fn empty(game: &Game) -> Self {
        Self {
            partial_game: no_partial_game(game),
            path: vec![],
            branches: 0,
        }
    }
}

/**
    A node in a tree search
**/
#[derive(Clone, Debug)]
pub struct EvalNode {
    pub path: Vec<Moveset>,
}

impl EvalNode {
    pub fn new(path: Vec<Moveset>) -> Self {
        Self {
            path
        }
    }
}

impl PartialEq for EvalNode {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for EvalNode {}

impl<'a> From<TreeNode<'a>> for EvalNode {
    fn from(node: TreeNode<'a>) -> EvalNode {
        EvalNode {
            path: node.path
        }
    }
}

impl<'a> From<&TreeNode<'a>> for EvalNode {
    fn from(node: &TreeNode<'a>) -> EvalNode {
        EvalNode {
            path: node.path.clone()
        }
    }
}
