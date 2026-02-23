# Release 0.0.5 — Verified Status

## Verification record (rel/0.0.5)

| Field | Value |
| :--- | :--- |
| **Date** | 2026-02-23 |
| **Environment** | macOS (darwin), Podman; optional: Linux + Docker |
| **Command** | `make teardown` → `make docker-base` → `./dflow ci-verify --purge` |
| **Result** | Full cold-build verification: base and CI images build; `make ghrun` runs inside container (fmt, lint, build, test-ci, smoke-test, verify). All diagram providers exercised via integration tests (bpmn, d2, ditaa, excalidraw, graphviz, mermaid, vega, vegalite, wavedrom). |
| **CI parity** | Local ci-verify and GHA use the same fingerprinted CI image (`ghcr.io/<org>/<repo>-ci:<hash>` / `:latest`). See [Build Pipeline & CI/CD](../developer-guide/pipeline.md#environment-parity-local-ci-verify-vs-gha). |

## How to re-verify

```bash
make teardown
make docker-base
./dflow ci-verify --purge
```

Success: script exits 0 and prints "Local CI verification (ci-verify) completed successfully."
