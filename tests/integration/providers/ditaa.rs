use crate::helpers::run_convert;

#[test]
fn test_convert_ditaa() {
    let output = run_convert("ditaa", "png", "test.ditaa");
    if output.status.success() {
        assert!(!output.stdout.is_empty(), "Output was empty");
        // Check for PNG magic number: 89 50 4E 47
        assert_eq!(&output.stdout[0..4], b"\x89PNG");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") || stderr.contains("No such file or directory") {
            println!("Skipping Ditaa test: tool not found");
        } else {
            panic!("Ditaa conversion failed: {}", stderr);
        }
    }
}
