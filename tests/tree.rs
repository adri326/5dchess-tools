use chess5dlib::parse::test::{read_and_parse};
use chess5dlib::{
    prelude::*,
    mate::*,
    tree::*,
    eval::*,
    eval::value::PieceValues,
};
use std::time::Duration;
#[macro_use]
extern crate lazy_static;

const N_THREADS: u32 = 4;
lazy_static! {
    pub static ref OPTIONS: TasksOptions<TrueGoal, FalseGoal> = TasksOptions::default().n_threads(N_THREADS).max_duration(Some(Duration::new(20, 0)));
}


#[test]
fn test_dfs_rook_tactics_1() {
    let game = read_and_parse("tests/games/puzzles/rook-tactics-1.json");
    let partial_game = no_partial_game(&game);

    let solution = Moveset::new(vec![Move::new(&game, &partial_game, Coords(0, 4, 4, 0), Coords(0, 4, 4, 4)).unwrap()], &game.info);
    assert!(solution.is_ok());
    let solution = solution.unwrap();

    let node = TreeNode {
        partial_game,
        path: vec![],
        branches: 0,
    };

    let res = dfs(&game, node, 1, Some(Duration::new(10, 0)), NoEvalFn::new(), TrueGoal, FalseGoal, false);
    assert!(res.is_some(), "dfs timed out or errored out on rook-tactics-1!");
    assert_eq!(res, dfs_schedule(&game, 1, NoEvalFn::new(), *OPTIONS, false), "dfs_schedule to return the same value as dfs");
    let (node, value) = res.unwrap();
    assert_eq!(node.path.len(), 1);
    assert_eq!(node.path[0], solution);
    assert_eq!(value, f32::INFINITY);

    let partial_game = no_partial_game(&game);

    let new_partial_game = node.path[0].generate_partial_game(&game, &partial_game).unwrap();
    match is_mate(&game, &new_partial_game, None) {
        Mate::TimeoutCheckmate | Mate::TimeoutStalemate => {
            unreachable!();
        }
        Mate::Error => {
            panic!("is_mate errored out while re-analyzing the best move of dfs!");
        }
        Mate::Checkmate => {
            // Ok
        }
        x => {
            panic!("is_mate found that the best move of dfs isn't checkmate! Got {:?}", x);
        }
    }
}

#[test]
fn test_dfs_rook_tactics_2() {
    let game = read_and_parse("tests/games/puzzles/rook-tactics-2.json");
    let partial_game = no_partial_game(&game);

    let solution_1 = Moveset::new(vec![Move::new(&game, &partial_game, Coords(0, 2, 1, 0), Coords(0, 2, 5, 0)).unwrap()], &game.info);
    assert!(solution_1.is_ok());
    let solution_1 = solution_1.unwrap();

    let solution_2 = Moveset::new(vec![Move::new(&game, &partial_game, Coords(0, 2, 1, 0), Coords(0, 2, 1, 4)).unwrap()], &game.info);
    assert!(solution_2.is_ok());
    let solution_2 = solution_2.unwrap();

    let node = TreeNode {
        partial_game,
        path: vec![],
        branches: 0,
    };

    let res = dfs(&game, node.clone(), 3, Some(Duration::new(30, 0)), NoEvalFn::new(), TrueGoal, FalseGoal, false);
    assert!(res.is_some(), "dfs timed out or errored out on rook-tactics-2!");
    let dfs_res = dfs_schedule(&game, 3, NoEvalFn::new(), *OPTIONS, false).expect("dfs_schedule should return Some(...)");
    assert!(dfs_res.0.path[0] == solution_1 || dfs_res.0.path[0] == solution_2, "dfs_schedule should return a valid solution");
    let iddfs_res = iddfs(&game, node, Some(Duration::new(30, 0)), NoEvalFn::new(), TrueGoal, FalseGoal, false).expect("iddfs should return Some(...)");
    assert!(iddfs_res.0.path[0] == solution_1 || iddfs_res.0.path[0] == solution_2, "iddfs should return a valid solution");
    let (node, value) = res.unwrap();
    assert_eq!(node.path[0], solution_1);
    assert_eq!(value, f32::INFINITY);
}


#[test]
fn test_dfs_standard_1() {
    let game = read_and_parse("tests/games/puzzles/standard-mate-1.json");
    let partial_game = no_partial_game(&game);

    let solution = Moveset::new(vec![Move::new(&game, &partial_game, Coords(0, 2, 3, 0), Coords(0, 2, 5, 2)).unwrap()], &game.info);
    assert!(solution.is_ok());
    let solution = solution.unwrap();

    let node = TreeNode {
        partial_game,
        path: vec![],
        branches: 0,
    };

    let res = dfs(&game, node, 3, Some(Duration::new(20, 0)), NoEvalFn::new(), TrueGoal, FalseGoal, false);
    assert!(res.is_some(), "dfs timed out or errored out on standard-mate-1!");
    assert_eq!(res, dfs_schedule(&game, 1, NoEvalFn::new(), *OPTIONS, false), "dfs_schedule to return the same value as dfs");
    let (node, value) = res.unwrap();
    assert_eq!(node.path[0], solution);
    assert_eq!(value, f32::INFINITY);
}

#[test]
fn test_dfs_rook_tactics_3() {
    let game = read_and_parse("tests/games/puzzles/rook-tactics-3.json");
    let partial_game = no_partial_game(&game);

    let solution = Moveset::new(vec![Move::new(&game, &partial_game, Coords(1, 4, 3, 0), Coords(1, 4, 3, 4)).unwrap()], &game.info);
    assert!(solution.is_ok());
    let solution = solution.unwrap();

    let node = TreeNode {
        partial_game,
        path: vec![],
        branches: 0,
    };

    let res = dfs(&game, node, 3, Some(Duration::new(20, 0)), NoEvalFn::new(), TrueGoal, FalseGoal, false);
    assert!(res.is_some(), "dfs timed out or errored out on rook-tactics-3!");
    assert_eq!(res, dfs_schedule(&game, 1, NoEvalFn::new(), *OPTIONS, false), "dfs_schedule to return the same value as dfs");
    let (node, value) = res.unwrap();
    assert_eq!(node.path[0], solution);
    assert_eq!(value, f32::INFINITY);
}

#[test]
fn test_dfs_advanced_branching_2() {
    let game = read_and_parse("tests/games/puzzles/advanced-branching-2.json");
    let partial_game = no_partial_game(&game);

    let node = TreeNode {
        partial_game,
        path: vec![],
        branches: 0,
    };

    let res = dfs(&game, node, 1, Some(Duration::new(20, 0)), NoEvalFn::new(), TrueGoal, FalseGoal, false);
    assert!(res.is_some(), "dfs timed out or errored out on advanced-branching-2!");
    let partial_game = no_partial_game(&game);
    let (node, value) = res.unwrap();
    let new_partial_game = node.path[0].generate_partial_game(&game, &partial_game).unwrap();
    assert_eq!(value, f32::INFINITY);
    assert_eq!(is_mate(&game, &new_partial_game, None), Mate::Checkmate);
}

#[test]
fn test_eval_standard_empty() {
    let game = read_and_parse("tests/games/standard-empty.json");

    let node = TreeNode::empty(&game);
    let piece_values = PieceValues::default();

    assert_eq!(piece_values.into_eval().eval(&game, &node), Some(0.0));
}
