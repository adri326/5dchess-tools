use chess5dlib::game::*;

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
    }

    {
        let timelines_white = vec![
            TimelineInfo::new(0, None, 3, 0),
            TimelineInfo::new(1, Some((0, 0)), 3, 1),
            TimelineInfo::new(2, Some((0, 0)), 1, 1),
        ];
        let timelines_black = vec![
            TimelineInfo::new(-1, None, 3, 0),
        ];

        let info = Info::new(true, timelines_white, timelines_black);
        assert!(info.is_active(0));
        assert!(info.is_active(1));
        assert!(!info.is_active(2));
        assert!(info.is_active(-1));
        assert!(info.is_active(-2));
        assert!(!info.is_active(-3));
    }

    {
        let timelines_white = vec![
            TimelineInfo::new(0, None, 4, 0),
        ];
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
    }

    {
        let timelines_white = vec![
            TimelineInfo::new(0, None, 3, 0),
        ];
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
    }
}
