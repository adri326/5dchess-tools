/*
    Structures and functions related to the game's state.
*/

use std::fmt;
use std::convert::TryFrom;
use std::collections::HashMap;

/// The main structure, contains the entire state of a game
#[derive(Debug)]
pub struct Game {
    pub timelines: HashMap<i32, Timeline>,
    pub width: u8,
    pub height: u8,
    pub info: GameInfo,
}

/// Information about whose turn it is, where the present is and timeline priority
#[derive(Debug, Clone, Copy)]
pub struct GameInfo {
    pub present: isize,
    pub active_player: bool,
    pub min_timeline: i32,
    pub max_timeline: i32,
    pub even_initial_timelines: bool,
}

/// Represents an in-game timeline
#[derive(Debug)]
pub struct Timeline {
    pub index: i32,
    pub states: Vec<Board>,
    pub width: u8,
    pub height: u8,
    pub begins_at: isize,
    pub emerges_from: Option<i32>,
}

/// Represents an in-game board (be it active or not)
#[derive(Debug, Clone)]
pub struct Board {
    pub pieces: Vec<Piece>,
    pub width: u8,
    pub height: u8,
    pub l: i32, // its timeline
    pub t: isize, // its time coordinate
    pub king_w: Option<(u8, u8)>, // TODO: update if the king moves
    pub king_b: Option<(u8, u8)>,
    pub castle_w: (bool, bool),
    pub castle_b: (bool, bool),
}

/// Represents the contents of a board's square
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
    PrincessW,

    KingB,
    QueenB,
    PawnB,
    KnightB,
    RookB,
    BishopB,
    UnicornB,
    DragonB,
    PrincessB,
}

impl Game {
    pub fn new(width: u8, height: u8) -> Self {
        Game {
            timelines: HashMap::new(),
            width,
            height,
            info: GameInfo {
                even_initial_timelines: false,
                present: 0,
                active_player: true,
                min_timeline: 0,
                max_timeline: 0,
            }
        }
    }
}

impl GameInfo {
    pub fn tick(mut self) -> Self {
        self.present += 1;
        self.active_player = !self.active_player;
        self
    }
}

impl Timeline {
    pub fn new(index: i32, width: u8, height: u8, begins_at: isize, emerges_from: Option<i32>) -> Self {
        Timeline {
            index,
            states: vec![],
            width,
            height,
            begins_at,
            emerges_from,
        }
    }
}

impl From<usize> for Piece {
    /// Converts 5dchess-notation piece indices into `Piece`s.
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
            9 => Piece::PrincessW,
            33 => Piece::PawnB,
            34 => Piece::KnightB,
            35 => Piece::BishopB,
            36 => Piece::RookB,
            37 => Piece::QueenB,
            38 => Piece::KingB,
            39 => Piece::UnicornB,
            40 => Piece::DragonB,
            41 => Piece::PrincessB,
            _ => panic!("Invalid piece: {}", raw),
        }
    }
}

impl From<Piece> for usize {
    /// Convert `Piece`s into 5dchess-notation indices
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
            Piece::PrincessW => 9,
            Piece::PawnB => 33,
            Piece::KnightB => 34,
            Piece::BishopB => 35,
            Piece::RookB => 36,
            Piece::QueenB => 37,
            Piece::KingB => 38,
            Piece::UnicornB => 39,
            Piece::DragonB => 40,
            Piece::PrincessB => 41,
        }
    }
}

impl Piece {
    pub fn as_uppercase(&self) -> &'static str {
        match &self {
            Piece::Blank => " ",
            Piece::KingW | Piece::KingB => "K",
            Piece::QueenW | Piece::QueenB => "Q",
            Piece::PawnW | Piece::PawnB => "P",
            Piece::BishopW | Piece::BishopB => "B",
            Piece::KnightW | Piece::KnightB => "N",
            Piece::RookW | Piece::RookB => "R",
            Piece::UnicornW | Piece::UnicornB => "U",
            Piece::DragonW | Piece::DragonB => "D",
            Piece::PrincessW | Piece::PrincessB => "S",
        }
    }

    /// Returns whether or not that `Piece` is `Piece::Blank`
    #[inline]
    pub fn is_blank(&self) -> bool {
        match &self {
            Piece::Blank => true,
            _ => false,
        }
    }

    /// Returns whether or not that `Piece` is a `Piece::*W`. Returns `false` if it is blank
    #[inline]
    pub fn is_white(&self) -> bool {
        match &self {
            Piece::PawnW
            | Piece::KnightW
            | Piece::BishopW
            | Piece::RookW
            | Piece::QueenW
            | Piece::KingW
            | Piece::UnicornW
            | Piece::DragonW
            | Piece::PrincessW => true,
            _ => false,
        }
    }

    /// Returns whether or not that `Piece` is a `Piece::*B`. Returns `false` if it is blank
    #[inline]
    pub fn is_black(&self) -> bool {
        match &self {
            Piece::PawnB
            | Piece::KnightB
            | Piece::BishopB
            | Piece::RookB
            | Piece::QueenB
            | Piece::KingB
            | Piece::UnicornB
            | Piece::DragonB
            | Piece::PrincessB => true,
            _ => false,
        }
    }

    /// Returns whether or not that `Piece` is a `Piece::King*`
    #[inline]
    pub fn is_king(&self) -> bool {
        match &self {
            Piece::KingW | Piece::KingB => true,
            _ => false,
        }
    }

    /// Returns whether or not that Piece is a `Piece::Queen*`
    #[inline]
    pub fn is_queen(&self) -> bool {
        match &self {
            Piece::QueenW | Piece::QueenB => true,
            _ => false,
        }
    }

    /// Returns whether or not that Piece is a `Piece::Pawn*`
    #[inline]
    pub fn is_pawn(&self) -> bool {
        match &self {
            Piece::PawnW | Piece::PawnB => true,
            _ => false,
        }
    }

    /// Returns whether or not that Piece is a `Piece::Knight*`
    #[inline]
    pub fn is_knight(&self) -> bool {
        match &self {
            Piece::KnightW | Piece::KnightB => true,
            _ => false,
        }
    }

    /// Returns whether or not that Piece is a `Piece::Rook*`
    #[inline]
    pub fn is_rook(&self) -> bool {
        match &self {
            Piece::RookW | Piece::RookB => true,
            _ => false,
        }
    }

    /// Returns whether or not that Piece is a `Piece::Bishop*`
    #[inline]
    pub fn is_bishop(&self) -> bool {
        match &self {
            Piece::BishopW | Piece::BishopB => true,
            _ => false,
        }
    }

    /// Returns whether or not that Piece is a `Piece::Unicorn*`
    #[inline]
    pub fn is_unicorn(&self) -> bool {
        match &self {
            Piece::UnicornW | Piece::UnicornB => true,
            _ => false,
        }
    }

    /// Returns whether or not that Piece is a `Piece::Dragon*`
    #[inline]
    pub fn is_dragon(&self) -> bool {
        match &self {
            Piece::DragonW | Piece::DragonB => true,
            _ => false,
        }
    }

    /// Returns whether or not that Piece is a `Piece::Princess*`
    #[inline]
    pub fn is_princess(&self) -> bool {
        match &self {
            Piece::PrincessW | Piece::PrincessB => true,
            _ => false,
        }
    }

    /**
        Whether or not the piece can move by `n` tiles in any direction until it is blocked.
    **/
    #[inline]
    pub fn slides(&self) -> bool {
        match &self {
            Piece::BishopW
            | Piece::RookW
            | Piece::QueenW
            | Piece::UnicornW
            | Piece::DragonW
            | Piece::PrincessW
            | Piece::BishopB
            | Piece::RookB
            | Piece::QueenB
            | Piece::UnicornB
            | Piece::DragonB
            | Piece::PrincessB => true,
            _ => false,
        }
    }

    /// Whether or not the target piece belongs to the opponent
    #[inline]
    pub fn is_opponent_piece(&self, active_player: bool) -> bool {
        if active_player {
            self.is_black()
        } else {
            self.is_white()
        }
    }

    /// Whether or not the target piece belongs to themselves
    #[inline]
    pub fn is_own_piece(&self, active_player: bool) -> bool {
        if active_player {
            self.is_white()
        } else {
            self.is_black()
        }
    }

    /// Whether or not a piece could theoretically move to that square (by moving there or taking an opponent's piece)
    #[inline]
    pub fn is_takable_piece(&self, active_player: bool) -> bool {
        self.is_blank()
            || if active_player {
                self.is_black()
            } else {
                self.is_white()
            }
    }
}

impl Game {
    /// Returns whether or not there are +0/-0 timelines
    pub fn even_initial_timelines(&self) -> bool {
        self.info.even_initial_timelines
    }

    /// Returns the `l`-th timeline
    pub fn get_timeline<'a>(&'a self, l: i32) -> Option<&'a Timeline> {
        self.timelines.get(&l)
    }

    /// Returns a mutable reference to the `l`-th timeline
    pub fn get_timeline_mut<'a>(&'a mut self, l: i32) -> Option<&'a mut Timeline> {
        self.timelines.get_mut(&l)
    }

    /// Returns the `(l, t)` board, None if not found
    pub fn get_board<'a>(&'a self, l: i32, t: isize) -> Option<&'a Board> {
        self.get_timeline(l).map(|tl| tl.get_board(t)).flatten()
    }

    /// Returns the `(l, t)` board, panics if not found
    pub fn get_board_unsafe<'a>(&'a self, l: i32, t: isize) -> &'a Board {
        self.get_timeline(l).expect("Couldn't find timeline!").get_board_unsafe(t)
    }

    /// Returns a mutable reference to the `(l, t)` board, None if not found
    pub fn get_board_mut<'a>(&'a mut self, l: i32, t: isize) -> Option<&'a mut Board> {
        self.get_timeline_mut(l)
            .map(|tl| tl.get_board_mut(t))
            .flatten()
    }

    /// Returns a mutable reference to the `(l, t)` board, panics if not found
    pub fn get_board_mut_unsafe<'a>(&'a mut self, l: i32, t: isize) -> &'a mut Board {
        self.get_timeline_mut(l)
            .expect("Couldn't find timeline!")
            .get_board_mut_unsafe(t)
    }

    /// Returns the last board in the `l`-th timeline, None if not found
    pub fn get_last_board<'a>(&'a self, l: i32) -> Option<&'a Board> {
        self.get_timeline(l).map(|tl| tl.get_last_board()).flatten()
    }

    /// Returns the `(l, t, x, y)` piece, None if not found
    pub fn get<'a>(&'a self, l: i32, t: isize, x: u8, y: u8) -> Option<Piece> {
        self.get_timeline(l).map(|tl| tl.get(t, x, y)).flatten()
    }

    /// Returns the `(l, t, x, y)` piece, panics if not found
    pub fn get_unsafe<'a>(&'a self, l: i32, t: isize, x: u8, y: u8) -> Piece {
        self.timelines[&l].get_unsafe(t, x, y)
    }

    /** Appends a set of boards to the current game structure; currently only supports appending one board to every timeline.
        This is usually used together with `Move::generate_vboards`:

        ```
        let mut game: Game;
        let mv: Move;

        // Assign to `mv` and `game`

        let boards = mv.generate_vboards(&game, &game.info, &vec![], &vec![]);
        game.commit_moves(boards);
        ```
    **/
    pub fn commit_moves(&mut self, mut boards: Vec<Board>, info: GameInfo) {
        self.info = info;
        boards.sort_by_key(|b| b.t);
        boards.reverse();
        for b in boards.into_iter() {
            if let Some(tl) = self.get_timeline_mut(b.l) {
                if tl.get_board(b.t).is_none() {
                    tl.states.push(b)
                } else {
                    for (i, b2) in tl.states.iter().enumerate() {
                        println!("> {}+{}={} : {}", tl.begins_at, i, tl.begins_at + i as isize, b2.t);
                    }
                    println!("{:?}", tl.get_board(b.t));
                    println!("{:?}", b);
                    panic!("Board already there: {}/{}", b.l, b.t);
                }
            } else {
                self.timelines.insert(b.l, Timeline {
                    index: b.l,
                    begins_at: b.t,
                    width: self.width,
                    height: self.height,
                    emerges_from: None,
                    states: vec![b],
                });
            }
        }
        self.info.active_player = !self.info.active_player;
    }
}

impl Timeline {
    /// Returns the last board in this timeline, None if the timeline is still empty (which it shouldn't if created by normal means)
    pub fn get_last_board<'a>(&'a self) -> Option<&'a Board> {
        self.states.get(self.states.len() - 1)
    }

    /// Returns the board at `t` in this timeline (this will be different from the `t`-th board if the timeline is synthetic), None if not found
    pub fn get_board<'a>(&'a self, t: isize) -> Option<&'a Board> {
        if t < self.begins_at {
            None
        } else {
            self.states.get(usize::try_from(t - self.begins_at).ok()?)
        }
    }

    /// Returns the board at `t` in this timeline; panics if the board does not exist
    pub fn get_board_unsafe<'a>(&'a self, t: isize) -> &'a Board {
        &self.states[(t - self.begins_at) as usize]
    }

    /// Returns a mutable reference to the board at `t` in this timeline, None if not found
    pub fn get_board_mut<'a>(&'a mut self, t: isize) -> Option<&'a mut Board> {
        if t < self.begins_at {
            None
        } else {
            self.states.get_mut(usize::try_from(t - self.begins_at).ok()?)
        }
    }

    /// Returns a mutable reference to the board at `t` in this timeline, panics if the board does not exist
    pub fn get_board_mut_unsafe<'a>(&'a mut self, t: isize) -> &'a mut Board {
        &mut self.states[(t - self.begins_at) as usize]
    }

    /// Returns the piece at `(t, x, y)` in this timeline, None if not found
    pub fn get<'a>(&'a self, t: isize, x: u8, y: u8) -> Option<Piece> {
        self.get_board(t).map(|board| board.get(x, y)).flatten()
    }

    /// Returns the piece at `(t, x, y)` in this timeline, panics if the square does not exist. UB if that board's size is not equal to the timeline's own size
    pub fn get_unsafe<'a>(&'a self, t: isize, x: u8, y: u8) -> Piece {
        self.states[(t - self.begins_at) as usize].pieces[(x + self.width * y) as usize]
    }
}

impl Board {
    /// Creates a new Board instance
    pub fn new(t: isize, l: i32, width: u8, height: u8) -> Self {
        Board {
            t,
            l,
            width,
            height,
            pieces: vec![Piece::Blank; (width as usize) * (height as usize)],
            king_w: None,
            king_b: None,
            castle_w: (false, false),
            castle_b: (false, false),
        }
    }

    /// Returns the piece at `(x, y)`, None if not found
    pub fn get(&self, x: u8, y: u8) -> Option<Piece> {
        if x >= self.width || y >= self.height {
            None
        } else {
            self.pieces.get((x + y * self.width) as usize).copied()
        }
    }

    /// Returns the piece at `(x, y)`, panics if not found
    pub fn get_unsafe(&self, x: u8, y: u8) -> Piece {
        self.pieces[(x + y * self.width) as usize]
    }

    /// Sets the piece at `(x, y)`, returns `Ok` on success and `Err` if the square does not exist
    pub fn set(&mut self, x: u8, y: u8, piece: Piece) -> Result<(), ()> {
        if x >= self.width || y >= self.height {
            Err(())
        } else {
            self.pieces[(x + y * self.width) as usize] = piece;
            Ok(())
        }
    }

    /// Sets the piece at `(x, y)`, panics if the square does not exist
    pub fn set_unsafe(&mut self, x: u8, y: u8, piece: Piece) {
        self.pieces[(x + y * self.width) as usize] = piece;
    }

    /// Returns whose player's turn it is on this board
    pub fn active_player(&self) -> bool {
        self.t % 2 == 0
    }

    /// Returns whether or not this board must be played on (does not check if it is the last board in its timeline)
    pub fn is_active(&self, info: &GameInfo) -> bool {
        if info.even_initial_timelines {
            self.t <= info.present
                && if self.l < 0 {
                    self.l >= -info.max_timeline - 2
                } else {
                    self.l <= -info.min_timeline + 1
                }
        } else {
            self.t <= info.present
                && if self.l < 0 {
                    self.l >= -info.max_timeline - 1
                } else {
                    self.l <= -info.min_timeline + 1
                }
        }
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
                Piece::PrincessW => "S",
                Piece::PawnB => "p",
                Piece::KnightB => "n",
                Piece::BishopB => "b",
                Piece::RookB => "r",
                Piece::QueenB => "q",
                Piece::KingB => "k",
                Piece::UnicornB => "u",
                Piece::DragonB => "d",
                Piece::PrincessB => "s",
            }
        )
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                write!(f, "{}", self.pieces[(x + y * self.height) as usize])?;
            }
            if y > 0 {
                write!(f, "\n")?;
            }
        }
        Ok(())
    }
}

/// Applies a function on the `(l, t)` board and on all of its predecessors, stops if the function returns false
pub fn bubble_up<'a, F>(game: &'a Game, mut l: i32, mut t: isize, mut f: F)
where
    F: FnMut(&'a Board) -> bool,
{
    while let Some(board) = game.get_board(l, t) {
        if !f(board) {
            break;
        }
        t -= 1;
        let tl = game.get_timeline(l).unwrap();
        if t < tl.begins_at {
            if let Some(l2) = tl.emerges_from {
                l = l2;
            }
        }
    }
}

/** Applies a function on the `(l, t)` board and on all of its descendants, passes on a value down each branch of boards and stops if the function returns false.

    ## Example

    ```
    bubble_down(game, 0, 0, |board, _| {
        if board.get_unsafe(0, 0).is_blank() {
            (false, ())
        } else {
            println!("Piece on board {}/{} at 0:0 is {}", board.l, board.t, board.get_unsafe(0, 0))
        }
    }, ());
    ```
**/
pub fn bubble_down<'a, F, T>(game: &'a mut Game, l: i32, mut t: isize, mut f: F, initial: T)
where
    F: FnMut(&'_ mut Board, T) -> (bool, T),
    F: Copy,
    T: Copy,
{
    let checkpoints: Vec<(i32, isize)> = game
        .timelines
        .values()
        .filter(|tl| tl.emerges_from == Some(l))
        .map(|tl| (tl.index, tl.begins_at))
        .filter(|x| x.1 > t)
        .collect();
    let mut previous_value: T = initial;
    loop {
        match game.get_board_mut(l, t) {
            Some(b) => {
                let res = f(b, previous_value);
                if !res.0 {
                    break;
                }
                previous_value = res.1;
            }
            _ => break,
        }

        t += 1;
        checkpoints.iter().for_each(|(index, begins_at)| {
            if *begins_at == t + 1 {
                bubble_down(game, *index, t + 1, f, previous_value)
            }
        });
    }
}

/// Returns the string version out the timeline index as displayed in-game; does not prepend a `+` if `l >= 1`
pub fn write_timeline(l: i32, even_initial_timelines: bool) -> String {
    if even_initial_timelines {
        if l < 0 {
            format!("-{}", -l - 1)
        } else if l == 0 {
            String::from("+0")
        } else {
            l.to_string()
        }
    } else {
        l.to_string()
    }
}

/// Returns the string version of the `x` coordinate as displayed in-game
pub fn write_file(x: u8) -> char {
    [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w',
    ][x as usize]
}

/**
    Populates the castling rights of every board in the game; does so by induction (uses the `bubble_down` function)
**/
pub fn populate_castling_rights(game: &mut Game) {
    // I apologize to code readers for the visual density of this function

    let width = game.width;
    let height = game.height;
    let timeline_indices: Vec<i32> = game
        .timelines
        .values()
        .filter(|tl| tl.begins_at == 0)
        .map(|tl| tl.index.clone())
        .collect();

    for l in timeline_indices {
        // Extract white and black's position
        let (king_w, king_b) = {
            let board = game.get_board(l, 0).unwrap();
            let kings_w: Vec<(usize, Piece)> = board
                .pieces
                .iter()
                .copied()
                .enumerate()
                .filter(|(_i, p)| *p == Piece::KingW)
                .collect();
            let kings_b: Vec<(usize, Piece)> = board
                .pieces
                .iter()
                .copied()
                .enumerate()
                .filter(|(_i, p)| *p == Piece::KingB)
                .collect();
            if kings_w.len() != 1 && kings_b.len() != 1 {
                continue;
            }
            (
                kings_w.get(0).map(|k| ((k.0 % width as usize) as u8, (k.0 / width as usize) as u8)),
                kings_b.get(0).map(|k| ((k.0 % width as usize) as u8, (k.0 / width as usize) as u8)),
            )
        };

        // Get rook positions
        let (rook_w1, rook_w2, rook_b1, rook_b2) = {
            let board = game.get_board(l, 0).unwrap();

            let rooks_w: Vec<(usize, Piece)> = board
                .pieces
                .iter()
                .copied()
                .enumerate()
                .filter(|(i, p)| {
                    *p == Piece::RookW && *i / width as usize == king_w.map(|k| k.1).unwrap_or(height) as usize
                })
                .collect();
            let rooks_b: Vec<(usize, Piece)> = board
                .pieces
                .iter()
                .copied()
                .enumerate()
                .filter(|(i, p)| {
                    *p == Piece::RookB && *i / width as usize == king_b.map(|k| k.1).unwrap_or(height) as usize
                })
                .collect();

            let rook_w_left = rooks_w
                .iter()
                .filter(|(i, _p)| i % (width as usize) < king_w.map(|k| k.0).unwrap_or(0) as usize)
                .max_by_key(|(i, _p)| i % width as usize);
            let rook_w_right = rooks_w
                .iter()
                .filter(|(i, _p)| i % (width as usize) > king_w.map(|k| k.0).unwrap_or(width) as usize)
                .min_by_key(|(i, _p)| i % width as usize);
            let rook_b_left = rooks_b
                .iter()
                .filter(|(i, _p)| i % (width as usize) < king_b.map(|k| k.0).unwrap_or(0) as usize)
                .max_by_key(|(i, _p)| i % width as usize);
            let rook_b_right = rooks_b
                .iter()
                .filter(|(i, _p)| i % (width as usize) > king_b.map(|k| k.0).unwrap_or(width) as usize)
                .min_by_key(|(i, _p)| i % width as usize);

            (
                rook_w_left.map(|(i, _p)| ((i % width as usize) as u8, (i / width as usize) as u8)),
                rook_w_right.map(|(i, _p)| ((i % width as usize) as u8, (i / width as usize) as u8)),
                rook_b_left.map(|(i, _p)| ((i % width as usize) as u8, (i / width as usize) as u8)),
                rook_b_right.map(|(i, _p)| ((i % width as usize) as u8, (i / width as usize) as u8)),
            )
        };

        // Bubble down!
        bubble_down(
            game,
            l,
            0,
            |board, mut last_state| {
                if last_state.0 {
                    last_state.0 = rook_w1
                        .map(|(x, y)| board.get(x, y))
                        .flatten()
                        .map(|p| p == Piece::RookW)
                        .unwrap();
                }
                if last_state.1 {
                    last_state.1 = rook_w2
                        .map(|(x, y)| board.get(x, y))
                        .flatten()
                        .map(|p| p == Piece::RookW)
                        .unwrap();
                }
                if last_state.2 {
                    last_state.2 = rook_b1
                        .map(|(x, y)| board.get(x, y))
                        .flatten()
                        .map(|p| p == Piece::RookB)
                        .unwrap();
                }
                if last_state.3 {
                    last_state.3 = rook_b2
                        .map(|(x, y)| board.get(x, y))
                        .flatten()
                        .map(|p| p == Piece::RookB)
                        .unwrap();
                }
                if last_state.0 || last_state.1 {
                    if king_w
                        .map(|(x, y)| board.get(x, y))
                        .flatten()
                        .map(|p| p != Piece::KingW)
                        .unwrap()
                    {
                        last_state.0 = false;
                        last_state.1 = false;
                    }
                }
                if last_state.2 || last_state.3 {
                    if king_b
                        .map(|(x, y)| board.get(x, y))
                        .flatten()
                        .map(|p| p != Piece::KingB)
                        .unwrap()
                    {
                        last_state.2 = false;
                        last_state.3 = false;
                    }
                }

                board.king_w = king_w;
                board.king_b = king_b;
                board.castle_w = (last_state.0, last_state.1);
                board.castle_b = (last_state.2, last_state.3);

                (true, last_state)
            },
            (
                rook_w1.is_some() && king_w.is_some(),
                rook_w2.is_some() && king_w.is_some(),
                rook_b1.is_some() && king_b.is_some(),
                rook_b2.is_some() && king_b.is_some(),
            ),
        );
    }
}
