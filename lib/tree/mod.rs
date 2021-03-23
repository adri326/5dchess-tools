//! Tree search algorithms and utilities
use crate::prelude::*;

pub mod schedule;
pub use schedule::{Tasks};

pub mod dfs;
pub use dfs::*;

/// The number of nodes before movesets including future or inactive boards are discarded
pub const APPROX_MIN_NODES: usize = 16;

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

#[derive(Clone, Copy, PartialEq)]
pub struct TasksOptions<G: Goal> {
    pub n_threads: u32,
    pub pool_size: usize,
    pub max_pool_size: usize,
    pub goal: G,
    pub max_duration: Option<std::time::Duration>,
    pub approx: bool,
}

impl<G: Goal> TasksOptions<G> {
    pub fn n_threads(mut self, value: u32) -> Self {
        self.n_threads = value;
        self
    }

    pub fn pool_size(mut self, value: usize) -> Self {
        self.pool_size = value;
        self
    }

    pub fn max_pool_size(mut self, value: usize) -> Self {
        self.max_pool_size = value;
        self
    }

    pub fn max_duration(mut self, value: Option<std::time::Duration>) -> Self {
        self.max_duration = value;
        self
    }

    pub fn goal<G2: Goal>(self, value: G2) -> TasksOptions<G2> {
        (value, self).into()
    }

    pub fn approx(mut self, value: bool) -> Self {
        self.approx = value;
        self
    }
}

impl<G: Goal> From<G> for TasksOptions<G> {
    fn from(goal: G) -> Self {
        Self {
            n_threads: 1,
            pool_size: 32,
            max_pool_size: 10000,
            goal,
            max_duration: None,
            approx: false,
        }
    }
}

impl<G: Goal, H: Goal> From<(G, TasksOptions<H>)> for TasksOptions<G> {
    fn from((goal, options): (G, TasksOptions<H>)) -> Self {
        Self {
            n_threads: options.n_threads,
            pool_size: options.pool_size,
            max_pool_size: options.max_pool_size,
            goal,
            max_duration: options.max_duration,
            approx: options.approx,
        }
    }
}

impl Default for TasksOptions<ContinueGoal> {
    fn default() -> Self {
        Self {
            n_threads: 1,
            pool_size: 32,
            max_pool_size: 10000,
            goal: ContinueGoal,
            max_duration: None,
            approx: false,
        }
    }
}
