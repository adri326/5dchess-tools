use chess5dlib::prelude::*;
use std::thread::sleep;
use std::time::{Instant, Duration};

// TODO: TimedFilter/SigmaFilter-specific tests

#[test]
fn test_filter_timed() {
    let vec = vec![0, 1, 0, 1];
    let mut iter = vec.iter().filter_timed(|&x| {
        sleep(Duration::new(1, 0));
        *x == 1
    }, Duration::new(0, 500000000));

    assert!(iter.start.is_none());
    assert!(iter.elapsed().is_none());
    assert!(iter.remaining().is_none());
    assert!(iter.next().is_none());
    assert!(iter.elapsed().is_some());
    assert!(iter.elapsed().unwrap() > Duration::new(1, 0));
    assert!(iter.remaining() == Some(Duration::new(0, 0)));
    let start = Instant::now();
    assert!(iter.next().is_none());
    assert!(start.elapsed() < Duration::new(0, 500000000));

    // The following test *may* fail if `sleep` sleeps longer than .5s
    let mut iter = vec.iter().filter_timed(|&x| {
        sleep(Duration::new(0, 400000000));
        *x == 1
    }, Duration::new(1, 0));
    assert!(iter.next().is_some());
    assert!(iter.remaining().unwrap() <= Duration::new(0, 200000000));
    assert!(iter.remaining().unwrap() > Duration::new(0, 0));
    assert!(iter.next().is_none());
}

#[test]
fn test_filter_sigma() {
    let vec = vec![0, 1, 0, 1];
    let mut iter = vec.iter().filter_sigma(|&x| {
        sleep(Duration::new(1, 0));
        *x == 1
    }, Duration::new(0, 500000000));

    assert!(iter.sigma == Duration::new(0, 0));
    assert!(iter.elapsed() == Duration::new(0, 0));
    assert!(iter.remaining() == Duration::new(0, 500000000));
    assert!(iter.next().is_none());
    assert!(iter.elapsed() > Duration::new(1, 0));
    assert!(iter.remaining() == Duration::new(0, 0));
    let start = Instant::now();
    assert!(iter.next().is_none());
    assert!(start.elapsed() < Duration::new(0, 500000000));

    // The following test *may* fail if `sleep` sleeps longer than .5s
    let mut iter = vec.iter().filter_sigma(|&x| {
        sleep(Duration::new(0, 400000000));
        *x == 1
    }, Duration::new(1, 0));
    assert!(iter.next().is_some());
    assert!(iter.remaining() <= Duration::new(0, 200000000));
    assert!(iter.remaining() > Duration::new(0, 0));
    assert!(iter.next().is_none());
}
