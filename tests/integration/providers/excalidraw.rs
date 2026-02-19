use crate::helpers::run_convert;

#[test]
fn test_convert_excalidraw() {
    let output = run_convert("excalidraw", "svg", "test.excalidraw");
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("<svg"), "Output did not contain SVG tag");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") || stderr.contains("No such file or directory") {
            println!("Skipping Excalidraw test: tool not found");
        } else {
            panic!("Excalidraw conversion failed: {}", stderr);
        }
    }
}
