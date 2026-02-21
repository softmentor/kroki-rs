use std::process::Command;

/// Docker-based integration tests.
/// These are IGNORED by default to ensure fast developer cycles on macOS/Linux
/// without requiring a container engine.
///
/// Run manually with: `cargo test --test integration test_docker_version -- --ignored`

#[test]
#[ignore]
fn test_docker_version() {
    let output = Command::new("docker").arg("--version").output();

    match output {
        Ok(out) => {
            assert!(
                out.status.success(),
                "Docker is installed but failed to execute"
            );
            let version = String::from_utf8_lossy(&out.stdout);
            println!("Docker version: {}", version);
        }
        Err(_) => {
            // If docker is not installed, we treat it as a skip rather than a failure
            // if the test is ignored anyway.
            println!("Skipping docker test: docker not found");
        }
    }
}

#[test]
#[ignore]
fn test_docker_image_build() {
    // This test ensures the Dockerfile is at least syntactically correct
    // by running a 'docker build' check (non-recursive).
    // Note: This requires a docker daemon to be running.
    let output = Command::new("docker")
        .arg("build")
        .arg(".")
        .arg("--target")
        .arg("base")
        .arg("-t")
        .arg("kroki-rs-test-base")
        .output();

    if let Ok(out) = output {
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            panic!("Docker build failed:\n{}", stderr);
        }
    } else {
        println!("Skipping docker build test: docker command failed");
    }
}
