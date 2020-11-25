extern crate chess5dlib;
extern crate serde;
extern crate roy;

use chess5dlib::{game::*, moves::*, moveset::*, resolve::*, tree::*};
use serde::{Deserialize};
use std::fs::File;
use std::io::prelude::*;
use roy::Client;
use std::sync::mpsc;
use std::sync::Arc;
use tokio::{time, runtime};

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

#[derive(Debug)]
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
    pub game: Game,
}

#[derive(Clone, Debug)]
pub enum Message {
    Log(String),
}

const MAX_GAMES: usize = 2;
const NEW_GAME_TIMEOUT: usize = 60 * 5;
const PING_INTERVAL: u64 = 5;

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

    let client = Arc::new(Client::new_auth(config.hostname.clone(), Box::leak(token.unwrap().into_boxed_str())));

    let rt = runtime::Runtime::new().unwrap();

    let ping_handle = {
        let client = Arc::clone(&client);
        rt.spawn(async move {
            let mut interval = time::interval(time::Duration::from_secs(PING_INTERVAL));
            loop {
                interval.tick().await;
                let new_sessions = handle_sessions(client.clone()).await;
            }
        })
    };

    // let session = request::new_session(&client, Color::White).await;

    ping_handle.await.unwrap();
}

async fn handle_sessions(client: Arc<Client>) -> Vec<Session> {
    println!("[Session handler loop]");
    let sessions = request::sessions(&client).await;
    let mut active_sessions = sessions.into_iter().filter(|sess| !sess.ended).collect::<Vec<_>>();
    println!("{} active sessions", active_sessions.len());

    let mut dropped = Vec::new();

    if active_sessions.len() > MAX_GAMES {
        println!("Too many sessions, pruning the most recent ones...");
        for sess in active_sessions.iter() {
            if !sess.started {
                if !request::remove_session(&client, sess.id.clone()).await {
                    println!("Couldn't prune session!");
                } else {
                    dropped.push(sess.id.clone());
                }
            }

            if dropped.len() >= active_sessions.len() - MAX_GAMES {
                break;
            }
        }
    }
    active_sessions = active_sessions.into_iter().filter(|x| dropped.iter().find(|d| x.id == **d).is_none()).collect();

    active_sessions.into_iter().filter(|x| !x.started).collect()
}
