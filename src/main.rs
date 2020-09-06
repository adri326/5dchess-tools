use chess5dlib::{game::*, moves::*, tree::*, resolve::*};
use std::env;
use std::fs::File;
use std::io::prelude::*;
extern crate json;

fn main() -> std::io::Result<()> {
    // This is a simple example which will take the 40 most promising movesets, sort them by their score and display the 3 best movesets
    let path = env::args().last().unwrap();

    let mut file = File::open(path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    let mut game = Game::from(json::parse(&contents).expect("Couldn't parse JSON"));

    let virtual_boards: Vec<&Board> = vec![];

    println!("Boards:");
    let own_boards = get_own_boards(&game, &virtual_boards, &game.info);
    for b in own_boards {
        println!("{}", b);
        println!("");
    }

    // println!("Moves per board:");
    // for b in get_own_boards(&game, &virtual_boards, &game.info) {
    //     let probs = probable_moves(&game, b, &virtual_boards);
    //     println!("{:#?}", probs);
    //     let probs = probs.into_iter().map(|m| {
    //         let res = m.generate_vboards(&game, &game.info, &virtual_boards).unwrap();
    //         (m, res.0, res.1)
    //     }).collect::<Vec<_>>();
    //     let lore = Lore::new(&game, &virtual_boards, b, get_opponent_boards(&game, &virtual_boards, &game.info).iter().map(|x| *x), &game.info);
    //     let scored = score_moves(&game, &virtual_boards, b, &lore, probs, &game.info);
    //     println!("{:#?}", scored.iter().map(|(m, _, _, s)| (m, s)).collect::<Vec<_>>());
    // }

    println!("Best move:");
    let best_move = alphabeta(&game, 4, 250, 16, 16);
    if let Some((best, value)) = best_move {
        println!("{:?}: {}", best.0, value);
        for b in &best.1 {
            println!("{}", b);
            println!("");
        }
        game.commit_moves(best.1);
        game.info = best.2;
        game.info.active_player = !game.info.active_player;
    }

    // println!("Possible answers:");

    // let mut movesets = legal_movesets(&game, &game.info, &virtual_boards, 0, 0)
    //     .take(40)
    //     .collect::<Vec<_>>();

    // if game.info.active_player {
    //     movesets.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());
    // } else {
    //     movesets.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());
    // }

    // for moveset in movesets.iter().take(3) {
    //     println!("{:?}: {}", moveset.0, moveset.3);
    //     for b in &moveset.1 {
    //         println!("{}", b);
    //         println!("");
    //     }
    // }

    Ok(())
}
