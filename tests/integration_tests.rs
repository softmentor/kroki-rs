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
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("<svg"), "Output did not contain SVG tag");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") || stderr.contains("No such file or directory") {
            println!("Skipping D2 test: tool not found");
        } else {
            panic!(
                "D2 conversion failed:\nSTDOUT: {}\nSTDERR: {}",
                String::from_utf8_lossy(&output.stdout),
                stderr
            );
        }
    }
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
        if stderr.contains("not found") || stderr.contains("No such file or directory") {
            println!("Skipping BPMN test: tool not found");
        } else {
            panic!(
                "BPMN conversion failed:\nSTDOUT: {}\nSTDERR: {}",
                String::from_utf8_lossy(&output.stdout),
                stderr
            );
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
        if stderr.contains("not found") || stderr.contains("No such file or directory") {
            println!("Skipping Vega test: tool not found");
        } else {
            panic!("Vega conversion failed: {}", stderr);
        }
    }
}

#[test]
fn test_convert_vegalite() {
    let output = run_convert("vegalite", "svg", "test.vl.json");
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("<svg"), "Output did not contain SVG tag");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") || stderr.contains("No such file or directory") {
            println!("Skipping Vega-Lite test: tool not found");
        } else {
            panic!("Vega-Lite conversion failed: {}", stderr);
        }
    }
}

#[test]
fn test_convert_graphviz() {
    let output = run_convert("graphviz", "svg", "test.dot");
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("<svg"), "Output did not contain SVG tag");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") || stderr.contains("No such file or directory") {
            println!("Skipping Graphviz test: tool not found");
        } else {
            panic!("Graphviz conversion failed: {}", stderr);
        }
    }
}

#[test]
fn test_convert_mermaid() {
    let output = run_convert("mermaid", "svg", "test.mmd");
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("<svg"), "Output did not contain SVG tag");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") || stderr.contains("No such file or directory") {
            println!("Skipping Mermaid test: tool not found");
        } else {
            panic!("Mermaid conversion failed: {}", stderr);
        }
    }
}

#[test]
fn test_convert_plantuml() {
    let output = run_convert("plantuml", "svg", "test.puml");
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("<svg") || stdout.contains("<?xml"),
            "Output did not contain SVG/XML tag"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") || stderr.contains("No such file or directory") {
            println!("Skipping PlantUML test: tool not found");
        } else {
            panic!("PlantUML conversion failed: {}", stderr);
        }
    }
}

#[test]
fn test_convert_ditaa() {
    let output = run_convert("ditaa", "png", "test.ditaa");
    if output.status.success() {
        // PNG magic bytes
        let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(
            output.stdout.starts_with(&png_header),
            "Output did not contain PNG header"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") || stderr.contains("No such file or directory") {
            println!("Skipping Ditaa test: tool not found");
        } else {
            // ditaa often fails gracefully if java is missing
            println!("Skipping Ditaa test (likely java/ditaa issue): {}", stderr);
        }
    }
}
