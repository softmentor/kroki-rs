use crate::diagrams::{DiagramError, DiagramProvider, DiagramResult};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

crate::diagrams::define_provider!(D2Provider);

#[async_trait]
impl DiagramProvider for D2Provider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "Diagram source is empty".into(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, source: &str, format: &str) -> DiagramResult<Vec<u8>> {
        // d2 - - reads from stdin and writes to stdout
        // But only if format is svg?
        // d2 supports --stdout-format json|svg|png|...

        // Let's check d2 help again for --stdout-format default.
        // It says "d2 compiles ... to file.svg ... defaults to file.svg".
        // "Use - to have d2 read from stdin or write to stdout."

        // If I use `d2 - -`, it writes SVG to stdout (default).
        // If I want other formats, I need to specify?
        // `d2 input.d2 output.png`
        // `d2 --stdout-format png - -` ?

        let mut cmd = Command::new(&self.bin_path);

        // Input is stdin: "-"
        // Output is stdout: "-"

        // Need to handle formats.
        // d2 supports: svg, png, pdf, pptx, gif, txt

        // If format is passed, we might need a flag.
        // d2 usage: d2 [flags] input output

        // If format is svg (default): `d2 - -` works.
        // If format is png: `d2 input.d2 output.png`.
        // Does `d2 - -` support changing format?
        // Help says: `--stdout-format string output format when writing to stdout ... Usage: d2 input.d2 --stdout-format png - > output.png`

        // So correct usage for stdout is: `d2 --stdout-format <format> - -` (input -, output - implicit or explicit?)
        // The help example `d2 input.d2 --stdout-format png -` has input file `input.d2` and output `-`.

        // So if input is `-`, use `d2 --stdout-format <format> - -`?
        // Let's assume input is `-`.

        match format {
            "svg" | "png" | "pdf" => {
                // OK
            }
            _ => {
                return Err(DiagramError::UnsupportedFormat {
                    format: format.into(),
                    provider: "D2".into(),
                })
            }
        }

        cmd.arg("--layout=dagre"); // Default layout, maybe make configurable?
                                   // Actually don't enforce layout unless needed.

        // Use --stdout-format
        cmd.arg("--stdout-format").arg(format);

        // Input file: "-" (stdin)
        cmd.arg("-");

        // Output file: "-" (stdout) - Wait, if --stdout-format is used, maybe output file argument is not needed or must be `-`?
        // Help example: `d2 input.d2 --stdout-format png -`
        // Here `input.d2` is arg1, `-` is arg2 (output).
        // So if input is `-`: `d2 --stdout-format png - -`
        cmd.arg("-");

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = crate::diagrams::run_process_with_timeout(
            "d2",
            cmd,
            Some(source.as_bytes()),
            self.timeout_ms,
            source.len(),
        )
        .await?;

        if output.status.success() {
            if output.stdout.is_empty() {
                return Err(DiagramError::ProcessFailed(
                    "D2 conversion succeeded but output is empty".into(),
                ));
            }
            Ok(output.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(DiagramError::ProcessFailed(format!(
                "D2 conversion failed: {}",
                stderr
            )))
        }
    }
}
