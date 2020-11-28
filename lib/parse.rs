use super::game;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GameRaw {
    timelines: Vec<TimelineRaw>,
    width: u8,
    height: u8,
    active_player: bool,
}

/// Represents an in-game timeline
#[derive(Debug, Deserialize)]
struct TimelineRaw {
    index: f32,
    states: Vec<Vec<usize>>,
    width: u8,
    height: u8,
    begins_at: isize,
    emerges_from: Option<f32>,
}

pub fn parse(raw: &str) -> Option<game::Game> {
    let game_raw: GameRaw = serde_json::from_str(raw).ok()?;

    let even_initial_timelines = game_raw
        .timelines
        .iter()
        .any(|tl| tl.index == -0.5 || tl.index == 0.5);

    let min_timeline = game_raw.timelines
        .iter()
        .map(|tl| tl.index)
        .min_by_key(|x| (*x) as isize)?;
    let max_timeline = game_raw.timelines
        .iter()
        .map(|tl| tl.index)
        .max_by_key(|x| (*x) as isize)?;

    let timeline_width = ((-min_timeline).min(max_timeline) + 1.0).round();
    let active_timelines = game_raw.timelines
        .iter()
        .filter(|tl| tl.index.abs() <= timeline_width);
    let present = active_timelines
        .map(|tl| tl.begins_at + (tl.states.len() as isize) - 1)
        .min()?;

    let mut res = game::Game::new(game_raw.width, game_raw.height);

    res.info.present = present;
    res.info.min_timeline = de_l(min_timeline, even_initial_timelines);
    res.info.max_timeline = de_l(max_timeline, even_initial_timelines);
    res.info.active_player = game_raw.active_player;
    res.info.even_initial_timelines = even_initial_timelines;

    for tl in game_raw.timelines.into_iter() {
        res.timelines.insert(
            de_l(tl.index, even_initial_timelines),
            de_timeline(tl, even_initial_timelines),
        );
    }

    Some(res)
}

fn de_board(raw: Vec<usize>, t: isize, l: i32, width: u8, height: u8) -> game::Board {
    let mut res = game::Board::new(t, l, width, height);
    res.pieces = raw
        .into_iter()
        .map(|x| game::Piece::from(x))
        .collect();
    res
}

fn de_l(raw: f32, even: bool) -> i32 {
    if even && raw < 0.0 {
        (raw.ceil() - 1.0) as i32
    } else {
        raw.floor() as i32
    }
}

fn de_timeline(raw: TimelineRaw, even: bool) -> game::Timeline {
    let mut res = game::Timeline::new(
        de_l(raw.index, even),
        raw.width,
        raw.height,
        raw.begins_at,
        raw.emerges_from.map(|x| de_l(x, even)),
    );

    let index = de_l(raw.index, even);
    let begins_at = raw.begins_at;
    let width = raw.width;
    let height = raw.height;

    res.states = raw
        .states
        .into_iter()
        .enumerate()
        .map(|(i, b)| de_board(b, begins_at + i as isize, index, width, height))
        .collect();

    res
}
