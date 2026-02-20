use std::fs;
use std::path::PathBuf;

#[allow(dead_code)]
pub fn create_test_files(dir: &PathBuf) {
    fs::write(dir.join("test.d2"), "x -> y").unwrap();
    fs::write(dir.join("test.excalidraw"), "{\"type\":\"excalidraw\"}").unwrap();
    fs::write(dir.join("skip.txt"), "ignored").unwrap();
}

use std::process::Command;

pub fn get_binary_path() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let release_path = root.join("target/release/kroki-rs");
    if release_path.exists() {
        return release_path;
    }

    root.join("target/debug/kroki-rs")
}

pub fn run_convert(diagram_type: &str, format: &str, input_file: &str) -> std::process::Output {
    let bin = get_binary_path();
    let mut input_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    input_path.push("tests/fixtures");
    input_path.push(input_file);

    // Create a temporary cache directory to ensure fresh execution
    let temp_cache = tempfile::Builder::new()
        .prefix("kroki-test-cache-")
        .tempdir()
        .expect("Failed to create temp cache dir");

    Command::new(bin)
        .env("KROKI_CACHE_DIR", temp_cache.path())
        .arg("convert")
        .arg("-t")
        .arg(diagram_type)
        .arg("-f")
        .arg(format)
        .arg(input_path)
        .output()
        .expect("Failed to execute command")
}
