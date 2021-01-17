use chess5dlib::parse::test::read_and_parse;
use chess5dlib::prelude::*;
use criterion::measurement::Measurement;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion};
use rand::prelude::*;
use std::time::{Duration, Instant};

fn bench_movement_board<M: Measurement>(
    group: &mut BenchmarkGroup<M>,
    game: &Game,
    name: &str,
) {
    let partial_game = no_partial_game(&game);
    let mut rng = rand::thread_rng();

    let own_boards: Vec<&Board> = partial_game.own_boards(game).collect();

    let mut time: Duration = Duration::new(0, 0);
    let mut n_moves = 0usize;

    group.bench_with_input(
        BenchmarkId::new("Board::generate_moves", name),
        game,
        |b, game| {
            let mut iter = own_boards[rng.gen_range(0..own_boards.len())].generate_moves(&game, &partial_game).unwrap();
            b.iter(|| {
                let start = Instant::now();
                match iter.next() {
                    Some(_) => n_moves += 1,
                    None => {
                        iter = own_boards[rng.gen_range(0..own_boards.len())].generate_moves(&game, &partial_game).unwrap();
                    }
                }
                time += start.elapsed();
            })
        },
    );

    let mpms = (n_moves as f64) / (time.as_millis() as f64);
    println!("Timelines: {}", game.info.len_timelines());
    println!("Boards to play on: {}", own_boards.len());
    println!("Time (s): {}", time.as_millis() as f64 / 1000.0);
    println!("Moves: {}", n_moves);
    println!("Moves / ms: {}", mpms);
}

fn bench_movement_piece<M: Measurement>(
    group: &mut BenchmarkGroup<M>,
    game: &Game,
    name: &str,
) {
    let partial_game = no_partial_game(&game);
    let mut rng = rand::thread_rng();

    let mut own_pieces: Vec<PiecePosition> = Vec::new();

    for board in partial_game.own_boards(game) {
        for y in 0..game.height {
            for x in 0..game.width {
                let piece = board.get((x, y));
                if piece.is_piece_of_color(game.info.active_player) {
                    own_pieces.push(PiecePosition(
                        piece.piece().unwrap(),
                        Coords(board.l(), board.t(), x, y),
                    ));
                }
            }
        }
    }

    group.bench_with_input(
        BenchmarkId::new("Piece::generate_moves", name),
        game,
        |b, game| {
            let mut iter = own_pieces[rng.gen_range(0..own_pieces.len())].generate_moves(&game, &partial_game).unwrap();
            b.iter(|| {
                match iter.next() {
                    Some(_) => {},
                    None => {
                        iter = own_pieces[rng.gen_range(0..own_pieces.len())].generate_moves(&game, &partial_game).unwrap();
                    }
                }
            })
        },
    );
}

pub fn bench_movement<M: Measurement>(c: &mut Criterion<M>) {
    {
        let mut board_group = c.benchmark_group("Board");
        board_group.significance_level(0.1);
        board_group.sample_size(250);

        let game = read_and_parse("tests/games/standard-d4d5.json");
        bench_movement_board(&mut board_group, &game, "Simple");
        let game = read_and_parse("tests/games/standard-complex.json");
        bench_movement_board(&mut board_group, &game, "Complex");
        let game = read_and_parse("tests/games/standard-complex-2.json");
        bench_movement_board(&mut board_group, &game, "Complex 2");
    }

    {
        let mut piece_group = c.benchmark_group("Piece");
        piece_group.significance_level(0.1);
        piece_group.sample_size(1000);
        piece_group
            .warm_up_time(Duration::new(20, 0))
            .measurement_time(Duration::new(20, 0));

        let game = read_and_parse("tests/games/standard-d4d5.json");
        bench_movement_piece(&mut piece_group, &game, "Simple");
        let game = read_and_parse("tests/games/standard-complex.json");
        bench_movement_piece(&mut piece_group, &game, "Complex");
        let game = read_and_parse("tests/games/standard-complex-2.json");
        bench_movement_piece(&mut piece_group, &game, "Complex 2");
    }
}

fn bench_moveset_sub<M: Measurement>(
    group: &mut BenchmarkGroup<M>,
    game: &Game,
    name: &str,
) {
    let partial_game = no_partial_game(&game);

    group.bench_with_input(BenchmarkId::new("GenMovesetIter", name), game, |b, game| {
        let lambda = |ms: Result<Moveset, MovesetValidityErr>| ms.ok();
        let mut iter = GenMovesetIter::new(
            vec![game.get_board((0, 1)).unwrap().into()],
            &game,
            &partial_game,
        ).flatten().filter_map(lambda);
        b.iter(|| {
            match iter.next() {
                Some(_) => {},
                None => {
                    iter = GenMovesetIter::new(
                        vec![game.get_board((0, 1)).unwrap().into()],
                        &game,
                        &partial_game,
                    ).flatten().filter_map(lambda);
                }
            }
        })
    });
}

pub fn bench_moveset<M: Measurement>(c: &mut Criterion<M>) {
    let mut moveset_group = c.benchmark_group("Moveset");
    let game = read_and_parse("tests/games/standard-d4d5.json");
    bench_moveset_sub(&mut moveset_group, &game, "Simple");
    let game = read_and_parse("tests/games/standard-complex.json");
    bench_moveset_sub(&mut moveset_group, &game, "Complex");
    let game = read_and_parse("tests/games/standard-complex-2.json");
    bench_moveset_sub(&mut moveset_group, &game, "Complex 2");
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = bench_movement, bench_moveset
);
criterion_main!(benches);
