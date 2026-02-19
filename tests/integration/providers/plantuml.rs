use crate::helpers::run_convert;

#[test]
fn test_convert_plantuml() {
    let output = run_convert("plantuml", "svg", "test.puml");
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.contains("<svg") && !stdout.contains("<?xml") {
            println!("DEBUG: PlantUML stdout: {}", stdout);
            println!(
                "DEBUG: PlantUML stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        assert!(
            stdout.contains("<svg") || stdout.contains("<?xml"),
            "Output did not contain SVG/XML tag. Stdout len: {}",
            stdout.len()
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
