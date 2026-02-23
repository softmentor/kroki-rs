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

    // 1. Stable Core (CLI-based)
    fs::write(input_path.join("file1.dot"), "digraph G { A -> B }").unwrap();
    fs::write(input_path.join("file2.d2"), "A -> B").unwrap();
    fs::write(
        input_path.join("file3.ditaa"),
        "+---+  +---+\n| A |->| B |\n+---+  +---+",
    )
    .unwrap();

    // 2. Browser-based (Conditional)
    #[cfg(feature = "native-browser")]
    {
        fs::write(input_path.join("file4.mmd"), "graph TD; A-->B;").unwrap();
        fs::write(input_path.join("file5.bpmn"), "<?xml version=\"1.0\" encoding=\"UTF-8\"?><bpmn:definitions xmlns:bpmn=\"http://www.omg.org/spec/BPMN/20100524/MODEL\" id=\"Definitions_1\"><bpmn:process id=\"Process_1\" isExecutable=\"false\"/></bpmn:definitions>").unwrap();
    }

    // 3. Plugins / Static
    fs::write(
        input_path.join("file6.excalidraw"),
        "{\"type\":\"excalidraw\",\"version\":2,\"elements\":[]}",
    )
    .unwrap();
    fs::write(input_path.join("file7.vega"), "{\"data\": {\"values\": [{\"a\": \"A\", \"b\": 28}]}, \"mark\": \"bar\", \"encoding\": {\"x\": {\"field\": \"a\", \"type\": \"nominal\"}, \"y\": {\"field\": \"b\", \"type\": \"quantitative\"}}}").unwrap();
    fs::write(
        input_path.join("file8.wavedrom"),
        "{signal: [{name: 'clk', wave: 'p.....'}]}",
    )
    .unwrap();

    let bin = get_binary_path();

    let mut cmd = Command::new(&bin);
    cmd.arg("batch")
        .arg(input_path)
        .arg("--out-dir")
        .arg(output_path)
        .arg("-f")
        .arg("svg");

    let output = cmd.output().expect("Failed to execute batch command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Some tools might be missing on the host (e.g. d2, dot), but the core should skip them gracefully
        // if they are not essential or if the conversion fails due to missing binary.
        println!(
            "Batch Warning/Failure (continuing if expected):\nSTDOUT:\n{}\nSTDERR:\n{}",
            stdout, stderr
        );
    }

    let entries: Vec<_> = fs::read_dir(output_path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .unwrap();

    // Verify we got at least some outputs (Graphviz is definitely present in CI/Base)
    assert!(!entries.is_empty(), "Batch output directory is empty");
    println!("Batch output entries: {:?}", entries);
}
