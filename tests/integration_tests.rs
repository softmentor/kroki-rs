use std::path::PathBuf;
use std::process::Command;

fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target/debug/kroki-rs");
    path
}

fn run_convert(diagram_type: &str, format: &str, input_file: &str) -> std::process::Output {
    let bin = get_binary_path();
    let mut input_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    input_path.push("tests/fixtures");
    input_path.push(input_file);

    Command::new(bin)
        .arg("convert")
        .arg("-t")
        .arg(diagram_type)
        .arg("-f")
        .arg(format)
        .arg(input_path)
        .output()
        .expect("Failed to execute command")
}

#[test]
fn test_convert_d2() {
    let output = run_convert("d2", "svg", "test.d2");
    assert!(
        output.status.success(),
        "D2 conversion failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("<svg"), "Output did not contain SVG tag");
}

#[test]
fn test_convert_wavedrom() {
    // Wavedrom might not be installed in CI/Everywhere, so we might need to skip if not available
    let output = run_convert("wavedrom", "svg", "test.json5");
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("<svg"), "Output did not contain SVG tag");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") || stderr.contains("No such file or directory") {
            println!("Skipping Wavedrom test: tool not found");
        } else {
            panic!("Wavedrom conversion failed: {}", stderr);
        }
    }
}

#[test]
fn test_convert_bpmn() {
    let output = run_convert("bpmn", "svg", "test.bpmn");
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("<svg"), "Output did not contain SVG tag");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") {
            println!("Skipping BPMN test: tool not found");
        } else {
            panic!("BPMN conversion failed: {}", stderr);
        }
    }
}

#[test]
fn test_convert_vega() {
    let output = run_convert("vega", "svg", "test.vega");
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("<svg"), "Output did not contain SVG tag");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") {
            println!("Skipping Vega test: tool not found");
        } else {
            panic!("Vega conversion failed: {}", stderr);
        }
    }
}
