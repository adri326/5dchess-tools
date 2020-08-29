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
    println!("{:#?}", probable_moves(&game, game.get_last_board(0.0).unwrap(), &vec![]));

    Ok(())
}
