use std::fs::*;
use std::path::Path;

#[test]
fn verify_tests_are_prepared() {
    let dir = read_dir(Path::new("tests/converted-db/"));
    assert!(dir.is_ok(), "Can't open `tests/converted-db`");
    let mut dir = dir.unwrap().filter_map(|entry| entry.ok());
    assert!(dir.find(|entry| entry.path().as_path().extension().map(|ext| ext == "json").unwrap_or(false)).is_some(), "Expected tests/converted-db to contain one or more JSON files");
}
