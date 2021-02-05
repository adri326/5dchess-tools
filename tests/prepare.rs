use std::fs::*;
use std::path::Path;

#[test]
fn verify_tests_are_prepared() {
    let dir = read_dir(Path::new("./converted-db/standard/none/"));
    assert!(dir.is_ok(), "Can't open `./converted-db/standard/none/`");
    let mut dir = dir.unwrap().filter_map(|entry| entry.ok());
    assert!(
        dir.find(|entry| entry
            .path()
            .as_path()
            .extension()
            .map(|ext| ext == "json")
            .unwrap_or(false))
            .is_some(),
        "Expected ./converted-db/standard/none/ to contain one or more JSON files"
    );

    let dir = read_dir(Path::new("./converted-db/standard/white/"));
    assert!(dir.is_ok(), "Can't open `./converted-db/standard/white/`");
    let mut dir = dir.unwrap().filter_map(|entry| entry.ok());
    assert!(
        dir.find(|entry| entry
            .path()
            .as_path()
            .extension()
            .map(|ext| ext == "json")
            .unwrap_or(false))
            .is_some(),
        "Expected ./converted-db/standard/white/ to contain one or more JSON files"
    );
}
