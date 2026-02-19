use crate::helpers::run_convert;

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
