use chess5dlib::parse::test::read_and_parse;
use chess5dlib::{
    prelude::*,
    tree::*,
    eval::*,
    eval::value::*,
};
use criterion::measurement::Measurement;
use criterion::{
    criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion,
    BatchSize
};
use std::time::{Duration, Instant};

fn bench_dfs<M: Measurement>(
    group: &mut BenchmarkGroup<M>,
    game: &Game,
    depth: usize,
    max_branches: usize,
    name: &str
) {
    group.bench_with_input(BenchmarkId::new(name, format!("dfs, d={}, bl={}", depth, max_branches)), &game, |b, game| {
        let options = TasksOptions::default().pool_size(256).max_duration(Some(Duration::new(10, 0)));
        let mut tasks = Tasks::new(game, options);
        b.iter_batched(|| {
            match tasks.next() {
                Some(x) => x,
                None => {
                    tasks = Tasks::new(game, options);
                    tasks.next().unwrap()
                }
            }
        }, |(node, handle)| {
            let (node, value) = if node.path.len() > depth {
                let score = NoEvalFn::new().eval(&game, &node).unwrap();

                (node.into(), score)
            } else {
                let depth = depth - node.path.len();
                dfs_bl(
                    &game,
                    node,
                    depth,
                    max_branches,
                    Some(Duration::new(10, 0)),
                    NoEvalFn::new(),
                    FalseGoal,
                    false
                ).expect("dfs timed out!")
            };

            handle.report(value);
        }, BatchSize::SmallInput);
    });
}


fn bench_eval_sub<M: Measurement>(
    group: &mut BenchmarkGroup<M>,
    game: &Game,
    name: &str,
    eval_fn: impl EvalFn,
) {
    group.bench_with_input(name, &game, |b, game| {
        let options = TasksOptions::default().pool_size(256).max_duration(Some(Duration::new(10, 0)));
        let mut tasks = Tasks::new(game, options);
        b.iter_batched(|| {
            match tasks.next() {
                Some(x) => x,
                None => {
                    tasks = Tasks::new(game, options);
                    tasks.next().unwrap()
                }
            }
        }, |(node, handle)| {
            eval_fn.eval(game, &node);
        }, BatchSize::SmallInput);
    });
}

pub fn bench_tree<M: Measurement>(c: &mut Criterion<M>) {
    {
        let mut moveset_group = c.benchmark_group("dfs");
        moveset_group
            .warm_up_time(Duration::new(10, 0))
            .measurement_time(Duration::new(60, 0))
            .sample_size(100);

        let game = read_and_parse("tests/games/standard-check.json");
        bench_dfs(&mut moveset_group, &game, 1, 0, "Standard Check 1");
        bench_dfs(&mut moveset_group, &game, 2, 0, "Standard Check 1");
        bench_dfs(&mut moveset_group, &game, 3, 0, "Standard Check 1");
        bench_dfs(&mut moveset_group, &game, 4, 0, "Standard Check 1");
        bench_dfs(&mut moveset_group, &game, 5, 0, "Standard Check 1");

        bench_dfs(&mut moveset_group, &game, 1, 1, "Standard Check 1");
        bench_dfs(&mut moveset_group, &game, 2, 1, "Standard Check 1");
        bench_dfs(&mut moveset_group, &game, 3, 1, "Standard Check 1");
        bench_dfs(&mut moveset_group, &game, 4, 1, "Standard Check 1");
        bench_dfs(&mut moveset_group, &game, 5, 1, "Standard Check 1");

        bench_dfs(&mut moveset_group, &game, 1, 2, "Standard Check 1");
        bench_dfs(&mut moveset_group, &game, 2, 2, "Standard Check 1");
        bench_dfs(&mut moveset_group, &game, 3, 2, "Standard Check 1");
        bench_dfs(&mut moveset_group, &game, 4, 2, "Standard Check 1");

        bench_dfs(&mut moveset_group, &game, 1, 3, "Standard Check 1");
        bench_dfs(&mut moveset_group, &game, 2, 3, "Standard Check 1");
        bench_dfs(&mut moveset_group, &game, 3, 3, "Standard Check 1");

        bench_dfs(&mut moveset_group, &game, 1, 4, "Standard Check 1");
        bench_dfs(&mut moveset_group, &game, 2, 4, "Standard Check 1");
        bench_dfs(&mut moveset_group, &game, 3, 4, "Standard Check 1");

        let game = read_and_parse("tests/games/tree/slow-1.json");
        bench_dfs(&mut moveset_group, &game, 1, 0, "Slow 1");
        bench_dfs(&mut moveset_group, &game, 2, 0, "Slow 1");
        bench_dfs(&mut moveset_group, &game, 1, 1, "Slow 1");
        bench_dfs(&mut moveset_group, &game, 2, 1, "Slow 1");
    }
}

pub fn bench_eval<M: Measurement>(c: &mut Criterion<M>) {
    {
        let mut moveset_group = c.benchmark_group("PieceValues");
        moveset_group
            .warm_up_time(Duration::new(3, 0))
            .measurement_time(Duration::new(20, 0))
            .sample_size(1000);
        let game = read_and_parse("tests/games/standard-check.json");
        bench_eval_sub(&mut moveset_group, &game, "Standard Check 1", PieceValues::default());
    }
}

criterion_group!(
    name = tree;
    config = Criterion::default();
    targets = bench_tree, bench_eval
);
criterion_main!(tree);
