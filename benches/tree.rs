use chess5dlib::parse::test::read_and_parse;
use chess5dlib::{
    prelude::*,
    tree::*,
    eval::*,
};
use criterion::measurement::Measurement;
use criterion::{
    criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion,
    // BatchSize
};
use std::time::{Duration, Instant};
