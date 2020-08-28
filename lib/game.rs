use json::{JsonValue, object::Object};

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
}

#[derive(Debug, Clone)]
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
            Some(JsonValue::Array(tl)) => tl.iter().map(|raw_timeline| Timeline::from(raw_timeline)).collect::<Vec<Timeline>>(),
            _ => panic!("Expected JSON element 'timelines' to be an array!"),
        };
        let width = raw.get("width")
            .expect("Expected JSON key 'width' for Game.")
            .as_usize()
            .expect("Expected JSON element 'width' for Game to be a number");
        let height = raw.get("height")
            .expect("Expected JSON key 'height' for Game.")
            .as_usize()
            .expect("Expected JSON element 'height' for Game to be a number");
        Game {
            timelines,
            width,
            height
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
        let index = raw.get("index")
            .expect("Expected JSON to contain key 'index' for Timeline.")
            .as_f32()
            .expect("Expected JSON element 'index' for Timeline to be a number.");
        let width = raw.get("width")
            .expect("Expected JSON to contain key 'width' for Timeline.")
            .as_usize()
            .expect("Expected JSON element 'width' for Timeline to be a number.");
        let height = raw.get("height")
            .expect("Expected JSON to contain key 'height' for Timeline.")
            .as_usize()
            .expect("Expected JSON element 'height' for Timeline to be a number.");
        let begins_at = raw.get("begins_at")
            .expect("Expected JSON to contain key 'begins_at' for Timeline.")
            .as_usize()
            .expect("Expected JSON element 'begins_at' for Timeline to be a number.");
        let emerges_from = raw.get("emerges_from").map(|x| x.as_f32()).flatten();
        let states = match raw.get("states") {
            Some(JsonValue::Array(arr)) => {
                let mut res: Vec<Board> = Vec::new();
                for board in arr {
                    if let JsonValue::Array(raw_pieces) = board {
                        let mut pieces: Vec<Piece> = Vec::with_capacity(width * height);
                        for raw_piece in raw_pieces {
                            pieces.push(Piece::from(raw_piece.as_usize().expect("Expected piece to be a number")));
                        }
                        res.push(Board {
                            pieces,
                            width,
                            height
                        });
                    } else {
                        panic!("Expected JSON State to be an array.")
                    }
                }
                res
            },
            _ => panic!("Expected JSON element 'states' to be an array.")
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
            _ => false
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
            _ => false
        }
    }

    pub fn is_king(&self) -> bool {
        match &self {
            Piece::KingW
            | Piece::KingB => true,
            _ => false,
        }
    }

    pub fn is_queen(&self) -> bool {
        match &self {
            Piece::QueenW
            | Piece::QueenB => true,
            _ => false,
        }
    }

    pub fn is_knight(&self) -> bool {
        match &self {
            Piece::KnightW
            | Piece::KnightB => true,
            _ => false,
        }
    }

    pub fn is_rook(&self) -> bool {
        match &self {
            Piece::RookW
            | Piece::RookB => true,
            _ => false,
        }
    }

    pub fn is_bishop(&self) -> bool {
        match &self {
            Piece::BishopW
            | Piece::BishopB => true,
            _ => false,
        }
    }

    pub fn is_unicorn(&self) -> bool {
        match &self {
            Piece::UnicornW
            | Piece::UnicornB => true,
            _ => false,
        }
    }

    pub fn is_dragon(&self) -> bool {
        match &self {
            Piece::DragonW
            | Piece::DragonB => true,
            _ => false,
        }
    }
}
