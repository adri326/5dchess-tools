use super::*;

pub fn benchmark() -> Duration {
    let game = read_and_parse("examples/monte_carlo/bench.json");
    let partial_game = no_partial_game(&game);
    let start = Instant::now();
    // Since we're using a randomized method, we need quite a few samples!
    for _ in 0..1000 {
        match random_legal_moveset(&game, &partial_game, Some(Duration::new(5, 0))) {
            Ok(_) => panic!("Expected no moveset to be found!"),
            Err(_) => {}
        }
    }
    println!("Benchmark done! Avg. speed: {} μs/checkmate", start.elapsed().as_nanos() as f32 / 1000000.0);
    start.elapsed() / 1000
}
