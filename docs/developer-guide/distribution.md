---
title: How to package, release, deploy and operate
label: kroki-rs.developer-guide.release-deploy
---
# Release & Deploy

This guide outlines the recommended options for distributing **Kroki-rs** to users.

## 1. Homebrew (Recommended for macOS & Linux)

Homebrew is the most popular way to distribute CLI tools on macOS. To maintain your own distribution, you should create a **Homebrew Tap**.

-   **Pros**: Extremely user-friendly (`brew install`), supports auto-updates.
-   **Method**: Create a repository named `homebrew-tap` and add a `kroki-rs.rb` Formula.
-   **Security**: Homebrew uses SHA-256 checksums to verify binary integrity.

### Example Formula Snippet
```ruby
class KrokiRs < Formula
  desc "Rust-based drop-in replacement for Kroki"
  homepage "https://github.com/your-username/kroki-rs"
  url "https://github.com/your-username/kroki-rs/releases/download/v0.1.0/kroki-rs-macos.tar.gz"
  sha256 "ACTUAL_SHA256_HERE"

  def install
    bin.install "kroki-rs"
  end
end
```

## 2. GitHub Releases (Binary Distribution)

The simplest way to provide cross-platform support.

-   **Pros**: No extra infrastructure, supports all OSs (macOS, Linux, Windows).
-   **How**: Use GitHub Actions to automatically build and upload binaries for every tagged release.
-   **Security**: Always provide a `checksums.txt` file (SHA-256) so users can verify the downloads.

## 3. Cargo (crates.io)

For users who already have Rust installed.

-   **Pros**: Standard for Rust developers.
-   **How**: `cargo publish` to crates.io.
-   **Usage**: `cargo install kroki-rs`

## 4. Cargo-dist & Cargo-binstall

[cargo-dist](https://github.com/axodotdev/cargo-dist) is a specialized tool for Rust projects that automates the creation of installers and CI/CD pipelines.

-   **Pros**: Generates installers (shell scripts, PowerShell scripts) and Homebrew formulas automatically.
-   **Security**: Supports binary signing and generate detailed release manifests.

## 5. Security Best Practices

### Binary Signing
-   **macOS**: For a "safe" experience on macOS, binaries should be signed and notarized by an Apple Developer account. Without this, users will see a "Developer cannot be verified" warning.
-   **GPG Signing**: You can sign your release tags and binaries with GPG to prove authenticity.

### Integrity
-   Always publish SHA-256 hashes of your binaries.
-   Automate the build process using GitHub Actions to ensure the binary exactly matches the source code (Reproducible Builds).

## Recommended Strategy

1.  **Phase 1**: Set up **GitHub Actions** to build binaries for `x86_64` and `aarch64` (Apple Silicon).
2.  **Phase 2**: Use **cargo-dist** to generate release artifacts and a GitHub Release page.
3.  **Phase 3**: Create a **Homebrew Tap** pointing to these artifacts for the best end-user experience.

### 1. Automated Verification
The easiest way to build and verify a distribution locally is via the Makefile:

```bash
make verify
```

### 2. Create Distribution Package
To only build and package the binary:

```bash
make dist
```
This will create a `dist/` directory containing the release tarball and its SHA-256 checksum.

```bash
# Build for release
cargo build --release

# Create a temporary distribution directory
mkdir -p dist
cp target/release/kroki-rs dist/
cd dist

# Archive the binary
tar -czvf kroki-rs-macos.tar.gz kroki-rs

# Generate the SHA-256 checksum
shasum -a 256 kroki-rs-macos.tar.gz
```

### 2. Test Local Installation
You can test "installing" the binary by moving it to a directory in your `PATH` (like `/usr/local/bin` or a local `bin` folder):

```bash
# Install to local bin
cp kroki-rs /usr/local/bin/

# Verify it works from any directory
kroki-rs --version
```

### 3. Test Homebrew Formula Locally
If you want to test your Homebrew Formula without pushing to a Tap:

1.  Create the `kroki-rs.rb` file locally.
2.  Update the `url` to point to the **full path** of your local tarball.
3.  Run `brew install --build-from-source ./kroki-rs.rb`.

### 4. Continuous Integration
Use GitHub Actions locally with tools like [act](https://github.com/nektos/act) to run your release workflows on your machine before pushing.
