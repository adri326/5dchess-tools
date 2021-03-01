//! Tree search algorithms and utilities

pub mod schedule;
pub use schedule::{Tasks, DEFAULT_POOL_SIZE};

pub mod dfs;
pub use dfs::*;

/**
    A node in a tree search
**/
#[derive(Clone, Debug)]
pub struct TreeNode<'a> {
    pub partial_game: crate::prelude::PartialGame<'a>,
    pub path: Vec<crate::prelude::Moveset>,
}

impl<'a> PartialEq for TreeNode<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl<'a> Eq for TreeNode<'a> {}

/**
    A node in a tree search
**/
#[derive(Clone, Debug)]
pub struct EvalNode {
    pub path: Vec<crate::prelude::Moveset>,
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
