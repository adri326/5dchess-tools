use chess5dlib::parse::test::read_and_parse;
use chess5dlib::{
    prelude::*,
    tree::*,
    eval::*,
};
use criterion::measurement::Measurement;
use criterion::{
    criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion,
    BatchSize
};
use std::time::{Duration, Instant};
use std::sync::Arc;

fn bench_dfs<M: Measurement>(
    group: &mut BenchmarkGroup<M>,
    game: Arc<Game>,
    depth: usize,
    max_branches: usize,
    name: &str
) {
    group.bench_with_input(BenchmarkId::new(name, format!("dfs, d={}, bl={}", depth, max_branches)), &game, |b, game| {
        let mut tasks = Tasks::new(Arc::clone(&game), 256, Some(Duration::new(10, 0)));
        b.iter_batched(|| {
            match tasks.next() {
                Some(x) => x,
                None => {
                    tasks = Tasks::new(Arc::clone(&game), 256, Some(Duration::new(10, 0)));
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
                ).expect("dfs timed out!")
            };

            handle.report(value);
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

        let game = Arc::new(read_and_parse("tests/games/standard-check.json"));
        bench_dfs(&mut moveset_group, Arc::clone(&game), 1, 0, "Standard Check 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 2, 0, "Standard Check 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 3, 0, "Standard Check 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 4, 0, "Standard Check 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 5, 0, "Standard Check 1");

        bench_dfs(&mut moveset_group, Arc::clone(&game), 1, 1, "Standard Check 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 2, 1, "Standard Check 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 3, 1, "Standard Check 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 4, 1, "Standard Check 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 5, 1, "Standard Check 1");

        bench_dfs(&mut moveset_group, Arc::clone(&game), 1, 2, "Standard Check 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 2, 2, "Standard Check 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 3, 2, "Standard Check 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 4, 2, "Standard Check 1");

        bench_dfs(&mut moveset_group, Arc::clone(&game), 1, 3, "Standard Check 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 2, 3, "Standard Check 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 3, 3, "Standard Check 1");

        bench_dfs(&mut moveset_group, Arc::clone(&game), 1, 4, "Standard Check 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 2, 4, "Standard Check 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 3, 4, "Standard Check 1");

        let game = Arc::new(read_and_parse("tests/games/tree/slow-1.json"));
        bench_dfs(&mut moveset_group, Arc::clone(&game), 1, 0, "Slow 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 2, 0, "Slow 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 1, 1, "Slow 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 2, 1, "Slow 1");
    }
}


criterion_group!(
    name = tree;
    config = Criterion::default();
    targets = bench_tree
);
criterion_main!(tree);
