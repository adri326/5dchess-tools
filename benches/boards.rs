use chess5dlib::parse::test::read_and_parse;
use chess5dlib::{*, boards::*, utils::*};
use criterion::measurement::Measurement;
use criterion::{
    criterion_group, criterion_main, BenchmarkGroup, BatchSize, BenchmarkId, Criterion,
};
use rand::prelude::*;
use std::time::{Duration, Instant};
use std::convert::TryFrom;

fn bench_board_sub<M: Measurement, B>(group: &mut BenchmarkGroup<M>, game: &Game, name: &str)
where
    B: Clone + AsRef<Board> + AsMut<Board>,
    for<'b> &'b B: GenMoves<'b, B>,
    for<'b> B: From<(Board, &'b Game, &'b PartialGame<'b, B>)> + PopulateBoard<'b, B>,
{
    let partial_game: PartialGame<B> = PartialGame::try_from((&no_partial_game(&game), game)).expect("Convert PartialGame");
    let mut partial_game_iter = list_legal_movesets(game, &partial_game, Some(Duration::new(1, 0))).cycle();
    let mut rng = rand::thread_rng();

    group.bench_with_input(
        BenchmarkId::new("with_flag_check", name),
        game,
        |b, game| {
            let (_initial_ms, mut partial_game) = partial_game_iter.next().unwrap();
            partial_game.populate(game);
            let own_boards: Vec<BoardOr<B>> = partial_game.own_boards(game).collect();

            let mut iter = own_boards[rng.gen_range(0..own_boards.len())]
                .generate_moves_flag(&game, &partial_game, GenMovesFlag::Check)
                .unwrap();
            b.iter(|| {
                match iter.next() {
                    Some(_) => {},
                    None => {
                        iter = own_boards[rng.gen_range(0..own_boards.len())]
                            .generate_moves_flag(&game, &partial_game, GenMovesFlag::Check)
                            .unwrap();
                    }
                }
            })
        },
    );

    let mut delta: Duration = Duration::new(0, 0);
    let mut sigma = 0usize;

    group.bench_with_input(
        BenchmarkId::new("is_illegal", name),
        game,
        |b, game| {
            let (_initial_ms, mut partial_game) = partial_game_iter.next().unwrap();
            partial_game.populate(game);
            let own_boards: Vec<BoardOr<B>> = partial_game.own_boards(game).collect();
            let lambda = |ms: Result<Moveset, MovesetValidityErr>| ms.ok();
            let mut iter = GenMovesetIter::new(own_boards.clone(), game, &partial_game).flatten().filter_map(lambda);

            b.iter_batched(|| {
                let start = Instant::now();
                let mut will_fail = false;
                loop {
                    match iter.next() {
                        Some(ms) => {
                            match ms.generate_partial_game(game, &partial_game) {
                                Some(pos) => break (ms, pos),
                                None => {}
                            }
                        }
                        None => {
                            if will_fail {
                                panic!("Failed to find a moveset twice: is the position a stalemate?");
                            }
                            will_fail = true;
                            iter = GenMovesetIter::new(own_boards.clone(), game, &partial_game).flatten().filter_map(lambda);
                        }
                    }
                }
            }, |(ms, partial_game)| {
                is_illegal(game, &partial_game);
            }, BatchSize::SmallInput);
        },
    );

    if sigma > 0 {
        let mpms = (sigma as f64) / (delta.as_millis() as f64);
        println!("Timelines: {}", game.info.len_timelines());
        println!("Time (s): {}", delta.as_millis() as f64 / 1000.0);
        println!("Movesets: {}", sigma);
        println!("Movesets / ms: {}", mpms);
    }
}

pub fn bench_normal<M: Measurement>(c: &mut Criterion<M>) {
    let mut group = c.benchmark_group("Board");
    group.significance_level(0.1);
    group.sample_size(500);
    group
        .warm_up_time(Duration::new(5, 0))
        .measurement_time(Duration::new(15, 0));

    let game = read_and_parse("tests/games/standard-d4d5.json");
    bench_board_sub::<M, Board>(&mut group, &game, "Simple");
    let game = read_and_parse("tests/games/standard-complex.json");
    bench_board_sub::<M, Board>(&mut group, &game, "Complex");
    let game = read_and_parse("tests/games/standard-complex-2.json");
    bench_board_sub::<M, Board>(&mut group, &game, "Complex 2");

    for x in vec![1, 2, 3, 4, 5, 6, 8, 10, 12, 13].into_iter() {
        let path = format!("tests/games/inc_timelines/{}.json", x);
        let name = format!("{} Timelines", x);
        let game = read_and_parse(&path);
        bench_board_sub::<M, Board>(&mut group, &game, &name);
    }
}

criterion_group!(
    name = boards;
    config = Criterion::default();
    targets = bench_normal, bench_phase
);
criterion_main!(boards);
