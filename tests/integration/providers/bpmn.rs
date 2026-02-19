use crate::helpers::run_convert;

#[test]
fn test_convert_bpmn() {
    let output = run_convert("bpmn", "svg", "test.bpmn");
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // if !stdout.contains("<svg") {
        //      println!("DEBUG: BPMN stdout: {:?}", stdout);
        //      println!("DEBUG: BPMN stderr: {:?}", String::from_utf8_lossy(&output.stderr));
        // }
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
