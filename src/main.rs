#[allow(unused_imports)]
use chess5dlib::{parse::*, prelude::*, check::*, gen::*, tree::{*, dfs::*}, eval::*};
use std::env;
use std::fs::File;
use std::io::prelude::*;

// TODO: move replay, game analysis, args

fn main() -> std::io::Result<()> {
    let path = env::args().last().unwrap();

    let mut file = File::open(path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;
    let game = parse(&contents).unwrap();
    let partial_game = no_partial_game(&game);

    // prints a few informations on the game
    println!(
        "Active player: {}",
        if game.info.active_player {
            "white"
        } else {
            "black"
        }
    );
    println!("Timelines: {}", game.info.len_timelines());
    println!(
        "Boards to play on: {}",
        partial_game.own_boards(&game).count()
    );

    println!("Is in check? {:?}", is_in_check(&game, &partial_game));
    println!("Is mate? {:?}", chess5dlib::mate::is_mate(&game, &partial_game, None));

    const D: usize = 3;
    println!("DFS, d={}: {:?}", D, dfs_bl(&game, TreeNode::empty(&game), D, 3, None, PieceValues::default(), false));

    // println!("Number of movesets: {}", GenMovesetIter::new(
    //     partial_game.own_boards(&game).collect(),
    //     &game,
    //     &partial_game
    // ).flatten().count());

    // println!("Number of legal movesets: {}", GenMovesetIter::new(
    //     partial_game.own_boards(&game).collect(),
    //     &game,
    //     &partial_game
    // )
    // .flatten()
    // .filter(|ms| {
    //     match ms {
    //         Ok(ms) => {
    //             if let Some(new_partial_game) = ms.generate_partial_game(&game, &partial_game) {
    //                 !is_in_check(&game, &new_partial_game).unwrap()
    //             } else {
    //                 false
    //             }
    //         }
    //         Err(_) => false
    //     }
    // }).count());

    // let mut iter = generate_movesets_prefilter(
    //     partial_game.own_boards(&game).collect(),
    //     &game,
    //     &partial_game,
    // )
    // .flatten()
    // .filter(|ms| match ms {
    //     Ok(ms) => {
    //         if let Some(new_partial_game) = ms.generate_partial_game(&game, &partial_game) {
    //             !is_illegal(&game, &new_partial_game).unwrap().0
    //         } else {
    //             false
    //         }
    //     }
    //     Err(_) => false,
    // });

    // let mut dn = 0;
    // println!(
    //     "Is checkmate? {}",
    //     if iter.next().is_some() {
    //         dn = 1;
    //         "no"
    //     } else {
    //         "yes"
    //     }
    // );

    // println!("Number of legal movesets (filtered): {}", iter.count() + dn);

    Ok(())
}
