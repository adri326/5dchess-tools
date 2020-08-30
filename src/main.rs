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

    for boards in probable_moves(&game, game.get_last_board(0.0).unwrap(), &vec![])
        .into_iter()
        .map(|m| m.generate_vboards(&game, &game.info, &vec![]).unwrap().1)
    {
        println!("{}\n", boards[0]);
    }

    Ok(())
}
