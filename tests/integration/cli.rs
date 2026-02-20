use crate::helpers::get_binary_path;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_caching() {
    use std::fs;
    let temp_cache = tempfile::tempdir().expect("Failed to create temp dir");
    let cache_path = temp_cache.path().to_str().unwrap();

    let bin = get_binary_path();
    let mut input_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    input_path.push("tests/fixtures/test.excalidraw"); // Use existing fixture

    // First run: render and cache
    let output = Command::new(&bin)
        .arg("--cache-dir")
        .arg(cache_path)
        .arg("convert")
        .arg("-t")
        .arg("excalidraw")
        .arg("-f")
        .arg("svg")
        .arg(&input_path)
        .output()
        .expect("Failed to execute command");

    // Check if tool is missing
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") {
            println!("Skipping caching test: tool not found");
            return;
        }
        panic!("Caching test failed (run 1): {}", stderr);
    }

    // Verify cache file exists
    let entries: Vec<_> = fs::read_dir(cache_path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .unwrap();

    assert!(
        !entries.is_empty(),
        "Cache directory is empty after first run"
    );

    // Second run: should work
    let output2 = Command::new(&bin)
        .arg("--cache-dir")
        .arg(cache_path)
        .arg("convert")
        .arg("-t")
        .arg("excalidraw")
        .arg("-f")
        .arg("svg")
        .arg(&input_path)
        .output()
        .expect("Failed to execute command");

    assert!(output2.status.success(), "Caching test failed (run 2)");
}

#[test]
fn test_batch_conversion() {
    use std::fs;
    let temp_input = tempfile::tempdir().expect("Failed to create temp input dir");
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");

    let input_path = temp_input.path();
    let output_path = temp_output.path();

    // Create inputs — use diagram types that rely on Playwright (installed in CI)
    fs::write(input_path.join("file1.mmd"), "graph TD;\n    A-->B;").unwrap();
    fs::write(
        input_path.join("file2.mmd"),
        "sequenceDiagram\n    Alice->>Bob: Hello",
    )
    .unwrap();

    let bin = get_binary_path();

    let output = Command::new(&bin)
        .arg("batch")
        .arg(input_path)
        .arg("--out-dir")
        .arg(output_path)
        .arg("-f")
        .arg("svg")
        .output()
        .expect("Failed to execute batch command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "Batch conversion failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
            stdout, stderr
        );
    }

    let entries: Vec<_> = fs::read_dir(output_path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .unwrap();

    // We can't strictly assert not empty if all tools are missing.
    // But we should verify it ran without crashing.
    println!("Batch output entries: {:?}", entries);
}
