#[allow(unused_imports)]
use chess5dlib::{game::*, moves::*, moveset::*, resolve::*, tree::*};
use std::env;
use std::fs::File;
use std::io::prelude::*;
extern crate json;

// TODO: move replay, game analysis, args

fn main() -> std::io::Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        .init();

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
    //     // println!("{:#?}", probs);
    //     let probs = probs.into_iter().map(|m| {
    //         let res = m.generate_vboards(&game, &game.info, &virtual_boards, &vec![]).unwrap();
    //         (m, res.0, res.1)
    //     }).collect::<Vec<_>>();
    //     let lore = Lore::new(&game, &virtual_boards, b, get_opponent_boards(&game, &virtual_boards, &game.info).iter().map(|x| *x), &game.info);
    //     let scored = score_moves(&game, &virtual_boards, b, &lore, probs, &game.info);
    //     println!("{:#?}", scored.iter().map(|(m, _, _, s)| (m, s)).collect::<Vec<_>>());
    //     println!("{} :: {}", b.t, game.info.present);
    // }

    println!(
        "Turn {}, {} to play:",
        ((game.info.present) / 2) + 1,
        if game.info.active_player {
            "white"
        } else {
            "black"
        }
    );
    println!("Candidates:");
    // let best_move = alphabeta(&game, 4, 2000, 2000, 1000, 16);
    let best_move = iterative_deepening(
        &game,
        4000,
        1000,
        1000,
        50000,
        32,
        32.0,
        0.9,
        16,
        std::time::Duration::new(15, 0),
    );
    if let Some((best, value)) = best_move {
        println!("Best move:");
        println!("{:?}: {}", best.0, value);
        for b in &best.1 {
            println!("{}", b);
            println!("({}T{})\n", write_timeline(b.l), b.t / 2 + 1);
        }
        game.commit_moves(best.1);
        game.info = best.2;
    } else {
        if is_draw(&game, &virtual_boards, &game.info) {
            println!("Draw!");
        } else {
            println!("Checkmate!");
        }
        // break;
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
