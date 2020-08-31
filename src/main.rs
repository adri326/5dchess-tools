use chess5dlib::{game::*, moves::*};
use std::env;
use std::fs::File;
use std::io::prelude::*;
extern crate json;

fn main() -> std::io::Result<()> {
    let path = env::args().last().unwrap();

    let mut file = File::open(path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    let game = Game::from(json::parse(&contents).expect("Couldn't parse JSON"));

    // for boards in probable_moves(&game, game.get_last_board(0.0).unwrap(), &vec![])
    //     .into_iter()
    //     .map(|m| m.generate_vboards(&game, &game.info, &vec![]).unwrap().1)
    // {
    //     println!("{}\n", boards[0]);
    // }

    // println!("{:#?}", probable_moves(&game, game.get_last_board(-1.0).unwrap(), &vec![]));

    let legal = legal_movesets(&game, &vec![], &game.info);
    println!("{:#?}", legal);
    for (_m, i, b) in legal {
        println!("{:#?}", i);
        // let responses = legal_movesets(&game, &b, &i);
        // println!("{:#?}", responses);
        // println!("{:#?}", responses.iter().take(1).map(|(_m, i, b)| legal_movesets(&game, b, i).len()).collect::<Vec<_>>());
        break;
    }

    Ok(())
}
