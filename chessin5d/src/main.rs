use chess5dlib::{game::*, moves::*, moveset::*, resolve::*, tree::*};
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::prelude::*;
use roy::Client;

pub mod request;

#[derive(Deserialize)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub bio: String,
    pub fullname: String,
    pub hostname: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    White,
    Black,
    Random
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub id: String,
    pub host: String,
    pub white: Option<String>,
    pub black: Option<String>,
    pub variant: String,
    pub format: String,
    pub ranked: bool,
    pub ready: bool,
    pub offer_draw: bool,
    pub started: bool,
    pub start_date: usize,
    pub ended: bool,
    pub end_date: usize,
    pub archive_date: usize,
    pub player: String,
    pub winner: Option<String>,
    pub win_cause: Option<String>,
    pub width: usize,
    pub height: usize,
}

#[tokio::main]
async fn main() {
    let mut config_file = File::open("./config.ron").expect("Coudln't open file ./config.ron!");
    let mut config_raw = String::new();
    config_file.read_to_string(&mut config_raw).expect("Couldn't read config!");
    let config = ron::from_str::<Config>(&config_raw).expect("Couldn't parse config!");

    let client = Client::new(config.hostname.clone());

    let token = if std::env::args().find(|x| x == "--register").is_some() {
        println!("Registering a new account for \"{}\"", config.username);
        request::register(&client, &config).await
    } else {
        println!("Logging in as \"{}\"", config.username);
        request::login(&client, &config).await
    };

    if token.is_none() {
        eprintln!("Error logging in or registering an account; exiting!");
        return;
    }

    // roy, you made me do that.
    let client = Client::new_auth(config.hostname.clone(), Box::leak(token.unwrap().into_boxed_str()));

    let res = request::new_session(&client, &config, Color::White).await;
    println!("{:#?}", res);

    // println!("{:?}", client.get("/sessions", false).await.unwrap().text().await);
}
