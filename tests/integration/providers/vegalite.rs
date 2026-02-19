use crate::helpers::run_convert;

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
