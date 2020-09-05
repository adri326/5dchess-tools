use chess5dlib::{game::*, moves::*};
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

    let game = Game::from(json::parse(&contents).expect("Couldn't parse JSON"));

    let virtual_boards: Vec<&Board> = vec![];

    let mut movesets = legal_movesets(&game, &game.info, &virtual_boards).take(40).collect::<Vec<_>>();

    if game.info.active_player {
        movesets.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());
    } else {
        movesets.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());
    }

    for moveset in movesets.iter().take(3) {
        println!("{:?}: {}", moveset.0, moveset.3);
        for b in &moveset.1 {
            println!("{}", b);
            println!("");
        }
    }

    Ok(())
}
