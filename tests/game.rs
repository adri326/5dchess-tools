use chess5dlib::parse::*;
use chess5dlib::prelude::*;
use std::fs::File;
use std::io::Read;

#[test]
fn test_even_timelines() {
    {
        let timelines_white = vec![
            TimelineInfo::new(0, None, 3, 0),
            TimelineInfo::new(1, Some((0, 0)), 3, 1),
            TimelineInfo::new(2, Some((0, 0)), 1, 1),
        ];
        let timelines_black = vec![];

        let info = Info::new(false, timelines_white, timelines_black);
        assert!(info.is_active(0));
        assert!(info.is_active(1));
        assert!(!info.is_active(2));
        assert!(info.is_active(-1));
        assert!(!info.is_active(-2));
        assert_eq!(info.timeline_advantage(true), 0);
        assert_eq!(info.timeline_advantage(false), 3);
    }

    {
        let timelines_white = vec![
            TimelineInfo::new(0, None, 3, 0),
            TimelineInfo::new(1, Some((0, 0)), 3, 1),
            TimelineInfo::new(2, Some((0, 0)), 1, 1),
        ];
        let timelines_black = vec![TimelineInfo::new(-1, None, 3, 0)];

        let info = Info::new(true, timelines_white, timelines_black);
        assert!(info.is_active(0));
        assert!(info.is_active(1));
        assert!(!info.is_active(2));
        assert!(info.is_active(-1));
        assert!(info.is_active(-2));
        assert!(!info.is_active(-3));
        assert_eq!(info.timeline_advantage(true), 0);
        assert_eq!(info.timeline_advantage(false), 3);
    }

    {
        let timelines_white = vec![TimelineInfo::new(0, None, 4, 0)];
        let timelines_black = vec![
            TimelineInfo::new(-1, Some((0, 1)), 4, 2),
            TimelineInfo::new(-2, Some((0, 1)), 2, 2),
        ];

        let info = Info::new(false, timelines_white, timelines_black);
        assert!(info.is_active(0));
        assert!(info.is_active(-1));
        assert!(!info.is_active(-2));
        assert!(info.is_active(1));
        assert!(!info.is_active(2));
        assert_eq!(info.timeline_advantage(true), 3);
        assert_eq!(info.timeline_advantage(false), 0);
    }

    {
        let timelines_white = vec![TimelineInfo::new(0, None, 3, 0)];
        let timelines_black = vec![
            TimelineInfo::new(-1, None, 3, 0),
            TimelineInfo::new(-2, Some((0, 1)), 4, 2),
            TimelineInfo::new(-3, Some((0, 1)), 2, 2),
        ];

        let info = Info::new(true, timelines_white, timelines_black);
        assert!(info.is_active(0));
        assert!(info.is_active(-1));
        assert!(info.is_active(-2));
        assert!(!info.is_active(-3));
        assert!(info.is_active(1));
        assert!(!info.is_active(2));
        assert_eq!(info.timeline_advantage(true), 3);
        assert_eq!(info.timeline_advantage(false), 0);
    }
}

pub fn read_and_parse(path: String) -> Option<Game> {
    let mut file = File::open(&path).ok()?;
    let mut contents = String::new();

    file.read_to_string(&mut contents).ok()?;

    parse(&contents)
}

#[test]
fn test_get_board() {
    let game = read_and_parse(String::from("tests/games/standard-d4.json")).unwrap();

    assert!(game.get_board((0, 0)).is_some());
    assert!(game.get_board((0, 0)).unwrap() == game.get_board_unchecked((0, 0)));

    assert!(game.get_board((0, 1)).is_some());
    assert!(game.get_board((0, 1)).unwrap() == game.get_board_unchecked((0, 1)));

    assert!(game.get_board((1, 0)).is_none());
    assert!(game.get_board((-1, 0)).is_none());
}

#[test]
#[should_panic]
fn test_get_board_unchecked_fail() {
    let game = read_and_parse(String::from("tests/games/standard-d4.json"));

    if let Some(game) = game {
        if let None = game.get_board((1, 0)) {
            game.get_board_unchecked((1, 0));
        }
    }
}

#[test]
fn test_get() {
    let game = read_and_parse(String::from("tests/games/standard-d4.json")).unwrap();

    for y in 0..8 {
        for x in 0..8 {
            assert!(!game.get(Coords(0, 0, x, y)).is_void());
            assert!(game.get(Coords(0, 0, x, y)) == game.get_unchecked(Coords(0, 0, x, y)));

            assert!(!game.get(Coords(0, 1, x, y)).is_void());
            assert!(game.get(Coords(0, 1, x, y)) == game.get_unchecked(Coords(0, 1, x, y)));
        }
    }
}
