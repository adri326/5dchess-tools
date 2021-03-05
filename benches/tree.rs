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

fn bench_dfs<M: Measurement>(group: &mut BenchmarkGroup<M>, game: Arc<Game>, depth: usize, name: &str) {
    group.bench_with_input(BenchmarkId::new(name, format!("dfs, d={}", depth)), &game, |b, game| {
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
                dfs(
                    &game,
                    node,
                    depth,
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
            .sample_size(10);

        let game = Arc::new(read_and_parse("tests/games/standard-check.json"));
        bench_dfs(&mut moveset_group, Arc::clone(&game), 1, "Standard Check 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 2, "Standard Check 1");
        // bench_dfs(&mut moveset_group, Arc::clone(&game), 3, "Standard Check 1");

        let game = Arc::new(read_and_parse("tests/games/tree/slow-1.json"));
        bench_dfs(&mut moveset_group, Arc::clone(&game), 1, "Slow 1");
        bench_dfs(&mut moveset_group, Arc::clone(&game), 2, "Slow 1");
    }
}


criterion_group!(
    name = tree;
    config = Criterion::default();
    targets = bench_tree
);
criterion_main!(tree);
