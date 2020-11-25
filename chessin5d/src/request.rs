use roy::Client;
use super::{Config, Color, Session};
use serde::{Deserialize, Serialize};
use chess5dlib::game::*;

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

pub async fn new_session(client: &Client, color: Color) -> Option<Session> {
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
                    let raw_obj: sessions::SessionRaw = serde_json::from_str(&raw_json).ok()?;
                    return Some(raw_obj.into());
                }
                None => {},
            }
        }
    }

    None
}

pub async fn sessions(client: &Client) -> Vec<Session> {
    let res = client.get("/sessions", false).await;

    if let Some(res) = res {
        if res.status().is_success() {
            match res.text().await.ok() {
                Some(raw_json) => {
                    let raw_vec: Option<Vec<sessions::SessionRaw>> = serde_json::from_str::<Vec<sessions::SessionRaw>>(&raw_json).ok();
                    return raw_vec.unwrap_or(vec![]).into_iter().map(|raw| raw.into()).collect();
                }
                None => {},
            }
        }
    }

    vec![]
}

pub async fn forfeit_session(client: &Client, id: String) -> Option<Session> {
    #[derive(Serialize, Debug)]
    struct ForfeitSessionBody {
        pub id: String
    };

    let res = client.post(&format!("/sessions/{}/forfeit", id), ForfeitSessionBody {id}).await;

    if let Some(res) = res {
        if res.status().is_success() {
            match res.text().await.ok() {
                Some(raw_json) => {
                    let raw_obj: sessions::SessionRaw = serde_json::from_str(&raw_json).ok()?;
                    return Some(raw_obj.into());
                }
                None => {},
            }
        }
    }

    None
}

pub async fn remove_session(client: &Client, id: String) -> bool {
    #[derive(Serialize, Debug)]
    struct ForfeitSessionBody {
        pub id: String
    };

    let res = client.post(&format!("/sessions/{}/remove", id), ForfeitSessionBody {id}).await;

    if let Some(res) = res {
        let x = res.status().is_success();
        if !x {
            println!("{:#?}", res.text().await);
        }
        x
    } else {
        false
    }
}

mod sessions {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[allow(non_snake_case)]
    pub struct SessionRaw {
        id: String,
        host: String,
        white: Option<String>,
        black: Option<String>,
        variant: String,
        format: String,
        ranked: bool,
        ready: bool,
        offerDraw: bool,
        started: bool,
        startDate: usize,
        ended: bool,
        endDate: usize,
        archiveDate: usize,
        player: String,
        winner: Option<String>,
        winCause: Option<String>,
        board: BoardRaw,
    }

    impl Into<Session> for SessionRaw {
        fn into(self) -> Session {
            let mut game = Game::new(self.board.width(), self.board.height());
            game.info.active_player = parse_player_color(&self.board.player);
            for tl in &self.board.timelines {
                game.timelines.push(
                    BoardTimelinePair(&self.board, &tl).into()
                );
            }
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
                width: self.board.width(),
                height: self.board.height(),
                game,
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[allow(non_snake_case)]
    struct BoardRaw {
        pub action: usize,
        pub player: String,
        width: Option<usize>,
        height: Option<usize>,
        pub timelines: Vec<TimelineRaw>,
    }

    impl BoardRaw {
        pub fn width(&self) -> usize {
            self.width.unwrap_or(8)
        }

        pub fn height(&self) -> usize {
            self.height.unwrap_or(8)
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[allow(non_snake_case)]
    struct TimelineRaw {
        pub timeline: f32,
        pub active: bool,
        pub present: bool,
        pub turns: Vec<TurnRaw>,
    }

    struct BoardTimelinePair<'a>(&'a BoardRaw, &'a TimelineRaw);

    impl<'a> Into<Timeline> for BoardTimelinePair<'a> {
        fn into(self) -> Timeline {
            // Come on, rust
            let (board, tl) = (self.0, self.1);
            let mut res = Timeline::new(tl.timeline, board.width(), board.height(), 0, None);

            if tl.turns.len() > 0 {
                res.begins_at = (tl.turns[0].turn as isize) * 2 + (if parse_player_color(&tl.turns[0].player) {0} else {1});
            }

            for (index, turn) in tl.turns.iter().enumerate() {
                res.states.push(TurnTriple(res.begins_at + index as isize, tl.timeline, turn).into())
            }
            res
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[allow(non_snake_case)]
    struct TurnRaw {
        pub turn: usize,
        pub player: String,
        pub pieces: Vec<PieceRaw>,
        width: Option<usize>,
        height: Option<usize>,
    }

    impl TurnRaw {
        pub fn width(&self) -> usize {
            self.width.unwrap_or(8)
        }

        pub fn height(&self) -> usize {
            self.height.unwrap_or(8)
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[allow(non_snake_case)]
    struct PieceRaw {
        pub position: PositionRaw,
        pub piece: String,
        pub player: String,
        pub hasMoved: bool,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[allow(non_snake_case)]
    struct PositionRaw {
        pub timeline: f32,
        pub rank: usize,
        pub file: usize,
    }

    struct TurnTriple<'a>(isize, f32, &'a TurnRaw);

    impl<'a> Into<Board> for TurnTriple<'a> {
        fn into(self) -> Board {
            let (t, l, turn) = (self.0, self.1, self.2);
            let mut res = Board::new(t, l, turn.width(), turn.height());

            for piece in &turn.pieces {
                // Revert once !52 is merged
                // res.set(piece.position.rank - 1, piece.position.file - 1, parse_piece_name(&piece.piece, parse_player_color(&piece.player)));
                res.set(piece.position.file - 1, piece.position.rank - 1, parse_piece_name(&piece.piece, parse_player_color(&piece.player))).unwrap();
            }

            res
        }
    }

    #[inline]
    fn parse_player_color(color: &str) -> bool {
        color == "white"
    }

    fn parse_piece_name(name: &str, white: bool) -> Piece {
        match (name, white) {
            ("", true) => Piece::PawnW,
            ("", false) => Piece::PawnB,
            ("N", true) => Piece::KnightW,
            ("N", false) => Piece::KnightB,
            ("B", true) => Piece::BishopW,
            ("B", false) => Piece::BishopB,
            ("R", true) => Piece::RookW,
            ("R", false) => Piece::RookB,
            ("Q", true) => Piece::QueenW,
            ("Q", false) => Piece::QueenB,
            ("P", true) => Piece::PrincessW,
            ("P", false) => Piece::PrincessB,
            ("K", true) => Piece::KingW,
            ("K", false) => Piece::KingB,
            ("U", true) => Piece::UnicornW,
            ("U", false) => Piece::UnicornB,
            ("D", true) => Piece::DragonW,
            ("D", false) => Piece::DragonB,
            _ => Piece::Blank,
        }
    }

    #[inline]
    fn export_player_color(white: bool) -> &'static str {
        if white {
            "white"
        } else {
            "black"
        }
    }

    fn export_piece_name(piece: Piece) -> Option<(&'static str, bool)> {
        match piece {
            Piece::PawnW => Some(("", true)),
            Piece::PawnB => Some(("", false)),
            Piece::KnightW => Some(("N", true)),
            Piece::KnightB => Some(("N", false)),
            Piece::BishopW => Some(("B", true)),
            Piece::BishopB => Some(("B", false)),
            Piece::RookW => Some(("R", true)),
            Piece::RookB => Some(("R", false)),
            Piece::QueenW => Some(("Q", true)),
            Piece::QueenB => Some(("Q", false)),
            Piece::PrincessW => Some(("P", true)),
            Piece::PrincessB => Some(("P", false)),
            Piece::KingW => Some(("K", true)),
            Piece::KingB => Some(("K", false)),
            Piece::UnicornW => Some(("U", true)),
            Piece::UnicornB => Some(("U", false)),
            Piece::DragonW => Some(("D", true)),
            Piece::DragonB => Some(("D", false)),
            _ => None,
        }
    }

}
