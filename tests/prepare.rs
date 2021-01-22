use std::fs::*;
use std::path::Path;

#[test]
fn verify_tests_are_prepared() {
    let dir = read_dir(Path::new("./converted-db/nonmate/"));
    assert!(dir.is_ok(), "Can't open `./converted-db/nonmate/`");
    let mut dir = dir.unwrap().filter_map(|entry| entry.ok());
    assert!(dir.find(|entry| entry.path().as_path().extension().map(|ext| ext == "json").unwrap_or(false)).is_some(), "Expected ./converted-db/nonmate/ to contain one or more JSON files");

    let dir = read_dir(Path::new("./converted-db/checkmate/"));
    assert!(dir.is_ok(), "Can't open `./converted-db/checkmate/`");
    let mut dir = dir.unwrap().filter_map(|entry| entry.ok());
    assert!(dir.find(|entry| entry.path().as_path().extension().map(|ext| ext == "json").unwrap_or(false)).is_some(), "Expected ./converted-db/checkmate/ to contain one or more JSON files");
}
