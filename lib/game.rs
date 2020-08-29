use json::{object::Object, JsonValue};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Game {
    pub timelines: Vec<Timeline>,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone)]
pub struct Timeline {
    pub index: f32,
    pub states: Vec<Board>,
    pub width: usize,
    pub height: usize,
    pub begins_at: usize,
    pub emerges_from: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct Board {
    pub pieces: Vec<Piece>,
    pub width: usize,
    pub height: usize,
    pub l: f32,
    pub t: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Piece {
    Blank,
    KingW,
    QueenW,
    PawnW,
    KnightW,
    RookW,
    BishopW,
    UnicornW,
    DragonW,
    KingB,
    QueenB,
    PawnB,
    KnightB,
    RookB,
    BishopB,
    UnicornB,
    DragonB,
}

impl From<JsonValue> for Game {
    fn from(raw: JsonValue) -> Self {
        if let JsonValue::Object(obj) = raw {
            Self::from(obj)
        } else {
            panic!("Expected JSON root element to be an object.")
        }
    }
}

impl From<Object> for Game {
    fn from(raw: Object) -> Self {
        let timelines = match raw.get("timelines") {
            Some(JsonValue::Array(tl)) => tl
                .iter()
                .map(|raw_timeline| Timeline::from(raw_timeline))
                .collect::<Vec<Timeline>>(),
            _ => panic!("Expected JSON element 'timelines' to be an array!"),
        };
        let width = raw
            .get("width")
            .expect("Expected JSON key 'width' for Game.")
            .as_usize()
            .expect("Expected JSON element 'width' for Game to be a number");
        let height = raw
            .get("height")
            .expect("Expected JSON key 'height' for Game.")
            .as_usize()
            .expect("Expected JSON element 'height' for Game to be a number");
        Game {
            timelines,
            width,
            height,
        }
    }
}

impl From<&JsonValue> for Timeline {
    fn from(raw: &JsonValue) -> Self {
        if let JsonValue::Object(obj) = raw {
            Self::from(obj)
        } else {
            panic!("Expected JSON Timeline to be an object.")
        }
    }
}

impl From<&Object> for Timeline {
    fn from(raw: &Object) -> Self {
        let index = raw
            .get("index")
            .expect("Expected JSON to contain key 'index' for Timeline.")
            .as_f32()
            .expect("Expected JSON element 'index' for Timeline to be a number.");
        let width = raw
            .get("width")
            .expect("Expected JSON to contain key 'width' for Timeline.")
            .as_usize()
            .expect("Expected JSON element 'width' for Timeline to be a number.");
        let height = raw
            .get("height")
            .expect("Expected JSON to contain key 'height' for Timeline.")
            .as_usize()
            .expect("Expected JSON element 'height' for Timeline to be a number.");
        let begins_at = raw
            .get("begins_at")
            .expect("Expected JSON to contain key 'begins_at' for Timeline.")
            .as_usize()
            .expect("Expected JSON element 'begins_at' for Timeline to be a number.");
        let emerges_from = raw.get("emerges_from").map(|x| x.as_f32()).flatten();
        let states = match raw.get("states") {
            Some(JsonValue::Array(arr)) => {
                let mut res: Vec<Board> = Vec::new();
                let mut t: usize = 0;
                for board in arr {
                    if let JsonValue::Array(raw_pieces) = board {
                        let mut pieces: Vec<Piece> = Vec::with_capacity(width * height);
                        for raw_piece in raw_pieces {
                            pieces.push(Piece::from(
                                raw_piece.as_usize().expect("Expected piece to be a number"),
                            ));
                        }
                        res.push(Board {
                            pieces,
                            width,
                            height,
                            l: index,
                            t,
                        });
                    } else {
                        panic!("Expected JSON State to be an array.")
                    }
                    t += 1;
                }
                res
            }
            _ => panic!("Expected JSON element 'states' to be an array."),
        };
        Timeline {
            index,
            states,
            width,
            height,
            begins_at,
            emerges_from,
        }
    }
}

impl From<usize> for Piece {
    fn from(raw: usize) -> Self {
        match raw {
            0 => Piece::Blank,
            1 => Piece::PawnW,
            2 => Piece::KnightW,
            3 => Piece::BishopW,
            4 => Piece::RookW,
            5 => Piece::QueenW,
            6 => Piece::KingW,
            7 => Piece::UnicornW,
            8 => Piece::DragonW,
            11 => Piece::PawnB,
            12 => Piece::KnightB,
            13 => Piece::BishopB,
            14 => Piece::RookB,
            15 => Piece::QueenB,
            16 => Piece::KingB,
            17 => Piece::UnicornB,
            18 => Piece::DragonB,
            _ => panic!("Invalid piece: {}", raw),
        }
    }
}

impl From<Piece> for usize {
    fn from(raw: Piece) -> usize {
        match raw {
            Piece::Blank => 0,
            Piece::PawnW => 1,
            Piece::KnightW => 2,
            Piece::BishopW => 3,
            Piece::RookW => 4,
            Piece::QueenW => 5,
            Piece::KingW => 6,
            Piece::UnicornW => 7,
            Piece::DragonW => 8,
            Piece::PawnB => 11,
            Piece::KnightB => 12,
            Piece::BishopB => 13,
            Piece::RookB => 14,
            Piece::QueenB => 15,
            Piece::KingB => 16,
            Piece::UnicornB => 17,
            Piece::DragonB => 18,
        }
    }
}

impl Piece {
    pub fn is_blank(&self) -> bool {
        match &self {
            Piece::Blank => true,
            _ => false,
        }
    }

    pub fn is_white(&self) -> bool {
        match &self {
            Piece::PawnW
            | Piece::KnightW
            | Piece::BishopW
            | Piece::RookW
            | Piece::QueenW
            | Piece::KingW
            | Piece::UnicornW
            | Piece::DragonW => true,
            _ => false,
        }
    }

    pub fn is_black(&self) -> bool {
        match &self {
            Piece::PawnB
            | Piece::KnightB
            | Piece::BishopB
            | Piece::RookB
            | Piece::QueenB
            | Piece::KingB
            | Piece::UnicornB
            | Piece::DragonB => true,
            _ => false,
        }
    }

    pub fn is_king(&self) -> bool {
        match &self {
            Piece::KingW | Piece::KingB => true,
            _ => false,
        }
    }

    pub fn is_queen(&self) -> bool {
        match &self {
            Piece::QueenW | Piece::QueenB => true,
            _ => false,
        }
    }

    pub fn is_pawn(&self) -> bool {
        match &self {
            Piece::PawnW | Piece::PawnB => true,
            _ => false,
        }
    }

    pub fn is_knight(&self) -> bool {
        match &self {
            Piece::KnightW | Piece::KnightB => true,
            _ => false,
        }
    }

    pub fn is_rook(&self) -> bool {
        match &self {
            Piece::RookW | Piece::RookB => true,
            _ => false,
        }
    }

    pub fn is_bishop(&self) -> bool {
        match &self {
            Piece::BishopW | Piece::BishopB => true,
            _ => false,
        }
    }

    pub fn is_unicorn(&self) -> bool {
        match &self {
            Piece::UnicornW | Piece::UnicornB => true,
            _ => false,
        }
    }

    pub fn is_dragon(&self) -> bool {
        match &self {
            Piece::DragonW | Piece::DragonB => true,
            _ => false,
        }
    }

    pub fn is_opponent_piece(&self, active_player: bool) -> bool {
        if active_player {self.is_black()} else {self.is_white()}
    }

    pub fn is_takable_piece(&self, active_player: bool) -> bool {
        self.is_blank() || if active_player {self.is_black()} else {self.is_white()}
    }
}

impl Game {
    pub fn even_initial_timelines(&self) -> bool {
        self.timelines.iter().any(|tl| tl.index == 0.5 || tl.index == -0.5)
    }

    pub fn get_timeline<'a>(&'a self, l: f32) -> Option<&'a Timeline> {
        for timeline in self.timelines.iter() {
            if timeline.index == l {
                return Some(timeline);
            }
        }
        None
    }

    pub fn get_board<'a>(&'a self, l: f32, t: usize) -> Option<&'a Board> {
        self.get_timeline(l).map(|tl| tl.get_board(t)).flatten()
    }

    pub fn get_last_board<'a>(&'a self, l: f32) -> Option<&'a Board> {
        self.get_timeline(l).map(|tl| tl.get_last_board()).flatten()
    }

    pub fn get<'a>(&'a self, l: f32, t: usize, x: usize, y: usize) -> Option<Piece> {
        self.get_timeline(l).map(|tl| tl.get(t, x, y)).flatten()
    }
}

impl Timeline {
    pub fn get_last_board<'a>(&'a self) -> Option<&'a Board> {
        self.states.get(self.states.len() - 1)
    }

    pub fn get_board<'a>(&'a self, t: usize) -> Option<&'a Board> {
        if t < self.begins_at {
            None
        } else {
            self.states.get(t - self.begins_at)
        }
    }

    pub fn get<'a>(&'a self, t: usize, x: usize, y: usize) -> Option<Piece> {
        self.get_board(t).map(|board| board.get(x, y)).flatten()
    }
}

impl Board {
    pub fn get<'a>(&'a self, x: usize, y: usize) -> Option<Piece> {
        if x >= self.width || y >= self.height {
            None
        } else {
            self.pieces.get(x + y * self.width).copied()
        }
    }

    pub fn active_player(&self) -> bool {
        self.t % 2 == 0
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Piece::Blank => ".",
                Piece::PawnW => "P",
                Piece::KnightW => "N",
                Piece::BishopW => "B",
                Piece::RookW => "R",
                Piece::QueenW => "Q",
                Piece::KingW => "K",
                Piece::UnicornW => "U",
                Piece::DragonW => "D",
                Piece::PawnB => "p",
                Piece::KnightB => "n",
                Piece::BishopB => "b",
                Piece::RookB => "r",
                Piece::QueenB => "q",
                Piece::KingB => "k",
                Piece::UnicornB => "u",
                Piece::DragonB => "d",
            }
        )
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                write!(f, "{}", self.pieces[x + y * self.height])?;
            }
            if y > 0 {
                write!(f, "\n")?;
            }
        }
        Ok(())
    }
}

pub fn timeline_below(game: &Game, l: f32) -> f32 {
    if l == 1.0 && game.even_initial_timelines() {
        0.5
    } else if l == -0.5 {
        -1.0
    } else {
        l - 1.0
    }
}

pub fn timeline_above(game: &Game, l: f32) -> f32 {
    if l == -1.0 && game.even_initial_timelines() {
        -0.5
    } else if l == 0.5 {
        1.0
    } else {
        l + 1.0
    }
}

pub fn write_timeline(l: f32) -> String {
    if l == -0.5 {
        String::from("-0")
    } else if l == 0.5 {
        String::from("+0")
    } else {
        (l as isize).to_string()
    }
}

pub fn write_file(x: usize) -> char {
    ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w'][x]
}
