use roy::Client;
use super::{Config, Color, Session};
use serde::{Deserialize, Serialize};

pub async fn register(client: &Client, config: &Config) -> Option<String> {
    #[derive(Serialize, Debug)]
    struct RegisterConfig {
        pub username: String,
        pub password: String,
        pub bio: String,
        pub fullname: String,
    };

    let res = client.post("/register", RegisterConfig {
        username: config.username.clone(),
        password: config.password.clone(),
        bio: config.bio.clone(),
        fullname: config.fullname.clone(),
    }).await;


    if let Some(res) = res {
        if res.status().is_success() {
            println!("Registering successful!");
            res.text().await.ok()
        } else {
            eprintln!("Couldn't register:");
            eprintln!("{:?}", res.text().await);
            None
        }
    } else {
        eprintln!("Unknown error while registering!");
        None
    }
}

pub async fn login(client: &Client, config: &Config) -> Option<String> {
    #[derive(Serialize, Debug)]
    struct LoginConfig {
        pub username: String,
        pub password: String,
    };

    let res = client.post("/login", LoginConfig {
        username: config.username.clone(),
        password: config.password.clone(),
    }).await;


    if let Some(res) = res {
        if res.status().is_success() {
            println!("Login successful!");
            res.text().await.ok()
        } else {
            eprintln!("Couldn't log in:");
            eprintln!("{:?}", res.text().await);
            None
        }
    } else {
        eprintln!("Unknown error while logging in!");
        None
    }
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct SessionRaw {
    pub id: String,
    pub host: String,
    pub white: Option<String>,
    pub black: Option<String>,
    pub variant: String,
    pub format: String,
    pub ranked: bool,
    pub ready: bool,
    pub offerDraw: bool,
    pub started: bool,
    pub startDate: usize,
    pub ended: bool,
    pub endDate: usize,
    pub archiveDate: usize,
    pub player: String,
    pub winner: Option<String>,
    pub winCause: Option<String>,
    pub board: BoardRaw,
}

impl Into<Session> for SessionRaw {
    fn into(self) -> Session {
        Session {
            id: self.id,
            host: self.host,
            white: self.white,
            black: self.black,
            variant: self.variant,
            format: self.format,
            ranked: self.ranked,
            ready: self.ready,
            offer_draw: self.offerDraw,
            started: self.started,
            start_date: self.startDate,
            ended: self.ended,
            end_date: self.endDate,
            archive_date: self.archiveDate,
            player: self.player,
            winner: self.winner,
            win_cause: self.winCause,
            width: self.board.width,
            height: self.board.height,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct BoardRaw {
    pub action: usize,
    pub player: String,
    pub width: usize,
    pub height: usize,
    pub timelines: Vec<TimelineRaw>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct TimelineRaw {
    pub timeline: f32,
    pub active: bool,
    pub present: bool,
    pub turns: Vec<TurnRaw>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct TurnRaw {
    pub turn: usize,
    pub player: String,
    pub pieces: Vec<PieceRaw>,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct PieceRaw {
    pub position: PositionRaw,
    pub piece: String,
    pub player: String,
    pub hasMoved: bool,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct PositionRaw {
    pub timeline: f32,
    pub rank: usize,
    pub file: usize,
}

pub async fn new_session(client: &Client, config: &Config, color: Color) -> Option<Session> {
    #[derive(Serialize, Debug)]
    struct NewSessionBody {
        pub player: String
    };

    let res = client.post("/sessions/new", NewSessionBody {
        player: match color {
            Color::White => String::from("white"),
            Color::Black => String::from("black"),
            Color::Random => String::from("random"),
        }
    }).await;

    if let Some(res) = res {
        if res.status().is_success() {
            match res.text().await.ok() {
                Some(raw_json) => {
                    let raw_obj: SessionRaw = serde_json::from_str(&raw_json).ok()?;
                    return Some(raw_obj.into());
                }
                None => {},
            }
        }
    }

    None
}
