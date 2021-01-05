/*
    Structures and functions related to the game's state.
*/

use std::fmt;
use std::collections::HashMap;

pub type Layer = isize;
pub type Time = isize;
pub type Physical = u8;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    Princess,
    King,
    Brawn,
    Unicorn,
    Dragon,
    CommonKing,
    RoyalQueen,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Piece {
    pub kind: PieceKind,
    pub white: bool,
    pub moved: bool,
}

#[derive(Debug)]
pub struct Board {
    pub l: Layer,
    pub t: Time,
    pub width: Physical,
    pub height: Physical,
    pub pieces: Vec<Option<Piece>>,
}

pub struct TimelineInfo {
    pub index: Layer,
    pub starts_from: Option<(Layer, Time)>,
    pub last_board: Time,
    pub first_board: Time,
}

pub struct Info {
    pub present: Time,
    pub active_player: bool,
    pub min_timeline: Layer,
    pub max_timeline: Layer,
    pub even_timelines: bool,
    pub timelines_white: Vec<TimelineInfo>,
    pub timelines_black: Vec<TimelineInfo>,
}

pub struct Game {
    pub boards: HashMap<(Layer, Time), Board>,
    pub width: Physical,
    pub height: Physical,
    pub info: Info,
}

impl Piece {
    pub fn new(kind: PieceKind, white: bool, moved: bool) -> Self {
        Self {
            kind,
            white,
            moved,
        }
    }

    #[inline]
    pub fn is_royal(&self) -> bool {
        match self.kind {
            PieceKind::King
            | PieceKind::RoyalQueen => true,
            _ => false
        }
    }

    #[inline]
    pub fn can_promote(&self) -> bool {
        match self.kind {
            PieceKind::Pawn
            | PieceKind::Brawn => true,
            _ => false
        }
    }
}

impl TimelineInfo {
    pub fn new(index: Layer, starts_from: Option<(Layer, Time)>, last_board: Time, first_board: Time) -> Self {
        TimelineInfo {
            index,
            starts_from,
            last_board,
            first_board,
        }
    }
}

// timelines_white correspond to white's timelines and timelines_black correspond to black's timelines
// on an odd variant, white's timelines include the 0-timeline
// on an even variant, black's timeline include the -0-timeline and white's the +0-timeline
impl Info {
    pub fn new(
        even_timelines: bool,
        timelines_white: Vec<TimelineInfo>,
        timelines_black: Vec<TimelineInfo>
    ) -> Self {
        if timelines_white.len() == 0 {
            panic!("Expected at least one timeline!");
        }

        let min_timeline = -(timelines_black.len() as Layer);
        let max_timeline = timelines_white.len() as Layer - 1;
        let timeline_width = max_timeline.min(-min_timeline - (if even_timelines {1} else {0})) as usize + 1;
        let mut present = timelines_white[0].last_board;

        for tl in timelines_white.iter().take(timeline_width) {
            if tl.last_board < present {
                present = tl.last_board;
            }
        }

        for tl in timelines_black.iter().take(timeline_width - (if even_timelines {0} else {1})) {
            if tl.last_board < present {
                present = tl.last_board;
            }
        }

        let active_player = present % 2 == 0;

        Info {
            present,
            active_player,
            min_timeline,
            max_timeline,
            even_timelines,
            timelines_white,
            timelines_black
        }
    }

    pub fn get_timeline(&self, l: Layer) -> Option<&TimelineInfo> {
        if l < 0 {
            self.timelines_black.get(-l as usize - 1)
        } else {
            self.timelines_white.get(l as usize)
        }
    }

    pub fn is_active(&self, l: Layer) -> bool {
        let timeline_width = self.max_timeline.min(-self.min_timeline - (if self.even_timelines {1} else {0})) + 1;
        if l < 0 {
            if self.even_timelines {
                -l <= timeline_width + 1
            } else {
                -l <= timeline_width
            }
        } else {
            l <= timeline_width
        }
    }
}

impl Board {
    pub fn new(width: Physical, height: Physical, l: Layer, t: Time, pieces: Vec<Option<Piece>>) -> Self {
        Board {
            width,
            height,
            l,
            t,
            pieces,
        }
    }

    pub fn get(&self, x: Physical, y: Physical) -> Option<Piece> {
        self.pieces.get((x + self.width * y) as usize).map(|x| *x).flatten()
    }

    pub fn get_unchecked(&self, x: Physical, y: Physical) -> Option<Piece> {
        self.pieces[(x + self.width * y) as usize]
    }
}

impl Game {
    pub fn new(width: Physical, height: Physical, even_timelines: bool, timelines_white: Vec<TimelineInfo>, timelines_black: Vec<TimelineInfo>) -> Self {
        Game {
            boards: HashMap::new(),
            width,
            height,
            info: Info::new(even_timelines, timelines_white, timelines_black),
        }
    }

    pub fn get_board(&self, l: Layer, t: Time) -> Option<&Board> {
        self.boards.get(&(l, t))
    }

    pub fn get_board_unchecked(&self, l: Layer, t: Time) -> &Board {
        &self.boards[&(l, t)]
    }

    pub fn get(&self, l: Layer, t: Time, x: Physical, y: Physical) -> Option<Piece> {
        self.boards.get(&(l, t)).map(|b| b.get(x, y)).flatten()
    }

    pub fn get_unchecked(&self, l: Layer, t: Time, x: Physical, y: Physical) -> Option<Piece> {
        self.boards[&(l, t)].get_unchecked(x, y)
    }
}
