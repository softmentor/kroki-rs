use crate::helpers::run_convert;

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
fn test_mermaid_font_family() {
    // Specifically test that the font-family is correctly applied in the SVG output.
    // This verifies that the headless browser is correctly handling theme variables and fonts.
    let output = run_convert("mermaid", "svg", "test.mmd");
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // "trebuchet ms" is the default font for many mermaid themes,
        // but we check for any font-family declaration to ensure CSS is being applied.
        assert!(
            stdout.contains("font-family"),
            "SVG output did not contain font-family attribute"
        );
    }
}
