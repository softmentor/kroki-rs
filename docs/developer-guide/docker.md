---
title: Docker Developer Guide
label: kroki-rs.developer-guide.docker
---
# Docker Developer Guide

This guide covers building, testing, debugging, and operating **Kroki-rs** within OCI-compliant containers (Docker/Podman).

## 0. Quick Setup: Podman

If you don't have Podman installed, use the following commands:

### macOS (Homebrew)
```bash
brew install podman
podman machine init --memory=2048 --cpus=2 --disk-size=50
podman machine start
```

### Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install -y podman
```

## 1. Building the Image

Kroki-rs uses a multi-stage `Dockerfile` based on Debian Bookworm Slim to ensure a lightweight but fully-functional environment for all diagram providers.

### Local Build (Podman/Docker)

To build the image locally, run the following from the project root:

```bash
# Automates the build and tagging
make docker-build

# Reclaim space by pruning unused containers/images
make docker-clean
```

### Resource Optimization (Low-Memory Builds)

Compiling Rust and its dependencies can be memory-intensive. For environments with limited RAM (e.g., 2GB Podman machines), the `Dockerfile` is configured to limit parallel jobs:

- **Base Image Strategy**: The Dockerfile is split into a `base` stage (dependencies) and a `builder` stage. By pre-building the base, CI verification time is reduced to < 1 min.
- **Cargo Jobs**: The build stage uses `cargo build --release` (on high-capacity CI) or `-j 2` locally to prevent OOM.
- **Context Optimization**: A `.dockerignore` file is used to exclude `target/`, `node_modules/`, and `.git/`.

### Fast Local Packaging (Binary Injection)
If you already have a Linux binary (either built locally on Linux or downloaded from the CI), you can skip the Rust compilation entirely:

```bash
# Downloads the Linux binary for your architecture from GitHub Releases
./scripts/fetch-binary.sh

# Packages the image in < 5 seconds
make docker-pack
```

## 2. Testing & Verification

Once built, you should verify the containerized services and Playwright worker.

### Running & Testing

```bash
# Rotates container, health checks, and renders a test diagram
make docker-test

# Or run to interact/view logs directly
make docker-run
```

### End-to-End Verification

1.  **Health Check**: Verify the server and its browser pool are operational.
    ```bash
    curl http://localhost:8081/health
    ```
2.  **Diagram Rendering**: Test a Mermaid diagram to ensure Playwright and Chromium are correctly configured.
    ```bash
    # Content: graph TD; A-->B;
    curl http://localhost:8000/mermaid/svg/Z3JhcGggVEQ7IEEtLT5COw
    ```

## 3. Debugging

### Verbose Logging

If the server fails to start or the browser worker crashes, enable debug tracing:

```bash
podman run -it --rm -e RUST_LOG=debug kroki-rs:local-test
```

### Inspecting logs

Use `podman logs` to view output from both the Rust server and the Node.js background worker:

```bash
podman logs kroki-test
```

### Common Pitfalls

- **Chromium Sandbox**: In some container environments, the Chromium sandbox must be disabled. The internal worker automatically uses `--no-sandbox` for compatibility.
- **Architecture Mismatch**: Ensure you are building for the correct architecture. The image supports both `amd64` and `arm64`.

## 6. Local CI Testing (GitHub Actions)

You can run your GitHub Actions workflows locally using [act](https://github.com/nektos/act). This is useful for verifying the Docker build and test logic without pushing to GitHub.

### 1. Install `act`

On macOS, use Homebrew:
```bash
brew install act
```

### 2. Configure for Podman

`act` expects a Docker socket. You must point `DOCKER_HOST` to your Podman machine's socket:

```bash
# Set Podman socket for act
export DOCKER_HOST=unix://$(podman machine inspect --format '{{.ConnectionInfo.PodmanSocket.Path}}')
```

### 3. Run the Docker Workflow

To run the Docker-specific workflow:

```bash
# Automates socket configuration and act execution
# Mimics: Image Build -> Load -> Smoke Test (Health + Rendering)
make ci-local
```

### Complete Lifecycle

To build, test, and verify CI locally in one shot:

```bash
make docker-all
```

> [!NOTE]
> The first time you run `act`, it will ask you to choose a "Large", "Medium", or "Small" image. For `kroki-rs` builds, "Medium" is usually sufficient, but ensure your Podman machine has at least 2GB of RAM as configured in Step 0.

## 4. Operations & Monitoring

### Health Monitoring

The image exposes a dedicated Admin server on port `8081`. 
- **Endpoint**: `/health`
- **Metrics**: Returns the status of the browser pool (active/spare/pending connections).

### CI/CD Pipeline
The project uses a high-performance 3-tier pipeline:
- **Verification ([ci-build.yml](file:///Users/jinythattil/jt/code/softmentor/kroki-rs/.github/workflows/ci-build.yml))**: Extremely fast (sub-minute) smoke tests on PRs using the pre-built base image.
- **Base Image ([base-image.yml](file:///Users/jinythattil/jt/code/softmentor/kroki-rs/.github/workflows/base-image.yml))**: Automated dependency management.
- **Distribution ([release.yml](file:///Users/jinythattil/jt/code/softmentor/kroki-rs/.github/workflows/release.yml))**: Atomic release of multi-platform binaries and verified multi-arch Docker images on version tags.

## 5. Maintenance

When adding new providers that require native Node.js modules (like `canvas`), you must ensure that `build-essential` and `python3` are available in the build stage of the `Dockerfile` to satisfy `node-gyp`.
