use crate::helpers::run_convert;

#[test]
fn test_webp_generation_from_svg() {
    let output = run_convert("graphviz", "webp", "test.dot");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not supported or tool not found")
            || stderr.contains("Failed to execute")
        {
            println!("Skipping WebP test: dot tool not found");
            return;
        }
        panic!("Graphviz to WebP conversion failed: {}", stderr);
    }

    assert!(output.status.success(), "Conversion command should succeed");

    // Verify WEBP magic bytes (RIFF...WEBP)
    let stdout = &output.stdout;
    assert!(stdout.len() >= 12, "Output too small to be WebP");
    assert_eq!(&stdout[0..4], b"RIFF", "Missing RIFF header");
    assert_eq!(&stdout[8..12], b"WEBP", "Missing WEBP header");
}

#[test]
fn test_webp_generation_from_png() {
    let output = run_convert("ditaa", "webp", "test.ditaa");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not supported or tool not found")
            || stderr.contains("Failed to execute")
        {
            println!("Skipping WebP test: ditaa tool not found");
            return;
        }
        panic!("Ditaa to WebP conversion failed: {}", stderr);
    }

    assert!(output.status.success(), "Conversion command should succeed");

    // Verify WEBP magic bytes (RIFF...WEBP)
    let stdout = &output.stdout;
    assert!(stdout.len() >= 12, "Output too small to be WebP");
    assert_eq!(&stdout[0..4], b"RIFF", "Missing RIFF header");
    assert_eq!(&stdout[8..12], b"WEBP", "Missing WEBP header");
}
