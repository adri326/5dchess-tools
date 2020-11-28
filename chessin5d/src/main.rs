extern crate chess5dlib;
extern crate serde;
extern crate roy;
#[macro_use]
extern crate lazy_static;
extern crate tokio;

use chess5dlib::{game::*, moves::*, moveset::*, resolve::*, tree::*};
use serde::{Deserialize};
use std::fs::File;
use std::io::prelude::*;
use roy::Client;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use tokio::{time, runtime, task::JoinHandle};
use tokio::join;

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
    pub start_date: u128,
    pub ended: bool,
    pub end_date: u128,
    pub archive_date: usize,
    pub player: bool,
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
const NEW_GAME_TIMEOUT: u128 = 60 * 5 * 1000;
const PING_INTERVAL: u64 = 5;

type SessionReturnType = ();

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
        let username = config.username.clone();
        let client = Arc::clone(&client);
        rt.spawn(async move {
            let mut interval = time::interval(time::Duration::from_secs(PING_INTERVAL));
            let mut started_sessions: HashMap<String, JoinHandle<SessionReturnType>> = HashMap::new();
            let mut ready_sessions: HashMap<String, Session> = HashMap::new();
            loop {
                interval.tick().await;
                let new_sessions = handle_sessions(client.clone()).await;
                for sess in new_sessions.into_iter() {
                    if !sess.started && now() - sess.start_date >= NEW_GAME_TIMEOUT {
                        if &sess.host == &username {
                            println!("[Removing session {}]", sess.id);
                            if !request::remove_session(&client, sess.id.clone()).await {
                                println!("Coudln't remove session {}!", sess.id);
                            }
                        }
                        continue;
                    } else if started_sessions.get(&sess.id).is_some() {
                        continue;
                    } else if ready_sessions.get(&sess.id).is_some() {
                        if sess.started {
                            println!("[Starting session {}]", sess.id);
                            ready_sessions.remove(&sess.id);
                            let client = Arc::clone(&client);
                            let white = sess.white == Some(username.clone());
                            started_sessions.insert(sess.id.clone(), tokio::spawn(async move {
                                handle_session(client, white, sess).await
                            }));
                        }
                    } else {
                        if !sess.ready && !sess.started {
                            println!("[Getting ready for session {}]", sess.id);
                            if request::session_ready(&client, sess.id.clone()).await {
                                ready_sessions.insert(sess.id.clone(), sess);
                            } else {
                                println!("Couldn't flag ourselves as ready!");
                            }
                        } else {
                            println!("[Catch up ready session: {}]", sess.id);
                            ready_sessions.insert(sess.id.clone(), sess);
                        }
                    }
                }
            }
        })
    };

    if std::env::args().find(|x| x == "--new-session").is_some() {
        let session = request::new_session(&client, Color::White).await;
        match session {
            Some(info) => {
                println!("Session created!");
                println!("{:#?}", info);
            },
            _ => println!("Couldn't create session!"),
        }
    }
    if std::env::args().find(|x| x == "--clear-sessions").is_some() {
        println!("Removing sessions!");
        let sessions = request::sessions(&client).await;
        for sess in sessions {
            if sess.started && !sess.ended {
                if request::forfeit_session(&client, sess.id.clone()).await.is_some() {
                    println!("Forfeited session {}", sess.id);
                } else {
                    println!("Couldn't forfeit session {}", sess.id);
                }
            } else if sess.host == config.hostname && !sess.ended {
                if request::remove_session(&client, sess.id.clone()).await {
                    println!("Removed session {}", sess.id);
                } else {
                    println!("Couldn't remove session {}", sess.id);
                }
            }
        }
    }

    ping_handle.await.unwrap();
}

async fn handle_sessions(client: Arc<Client>) -> Vec<Session> {
    println!("[Sessions handler loop]");
    let sessions = request::sessions(&client).await;
    let active_sessions = sessions.into_iter().filter(|sess| !sess.ended).collect::<Vec<Session>>();
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

    active_sessions.into_iter().filter(|x| dropped.iter().find(|d| x.id == **d).is_none()).collect()
}

async fn handle_session(client: Arc<Client>, white: bool, mut session: Session) {
    let mut interval = time::interval(time::Duration::from_secs(PING_INTERVAL));
    println!("[Session handler: {}]", session.id);
    loop {
        interval.tick().await;

        match request::session(&client, session.id.clone()).await {
            Some(s) => session = s,
            None => println!("Couldn't get session {}!", session.id),
        }

        if session.player == white {
            // do them moves
            // send the moves
        }
    }
}

pub fn now() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("This software does not support actual time travel!").as_millis()
}
