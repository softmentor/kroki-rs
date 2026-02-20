---
title: Tech Debt Tracking
label: kroki-rs.developer-guide.tech-debt-tracking
---

# Tech Debt Tracking

Kroki-rs uses a living document (`tech-debts.md` in the project root) to track technical debt across releases.

## Purpose

Tech debt tracking helps the team:
- **Prioritize** remediation by severity and effort
- **Batch** related fixes into coherent commits
- **Measure** progress across releases
- Ensure critical items aren't forgotten between releases

## File Structure

The `tech-debts.md` file uses the following format:

```markdown
# Tech Debts — Project vX.Y.Z

## 🔴 Critical
- [x] **TD-01**: Short description (`file.rs`)
- [ ] **TD-02**: Short description (`file.rs`)

## 🟠 Major
- [x] **TD-03**: Description (`file.rs`)

## 🟡 Moderate
- [ ] **TD-04**: Description (`file.rs`)

## 🔵 Minor
- [ ] **TD-05**: Description (`file.rs`)
```

### Conventions

| Element | Description |
|---------|-------------|
| `TD-NN` | Sequential identifier, never reused |
| Severity | 🔴 Critical → 🟠 Major → 🟡 Moderate → 🔵 Minor |
| `[x]` | Fixed — include the version it was fixed in |
| `[ ]` | Open — not yet addressed |
| File ref | Always include the affected file(s) in parentheses |

### Severity Guidelines

| Level | Criteria |
|-------|----------|
| 🔴 **Critical** | Bugs, security issues, code duplication >50 lines, missing core validations |
| 🟠 **Major** | Performance issues, architectural problems, missing error handling |
| 🟡 **Moderate** | Code smells, inconsistent patterns, weak error messages |
| 🔵 **Minor** | Cosmetic issues, stale comments, missing assertions in tests |

## Workflow

1. **Identify**: During code review or development, log new items with the next `TD-NN` ID
2. **Triage**: Assign the appropriate severity level
3. **Batch**: Group related fixes (e.g., "all provider error handling" or "all async I/O fixes")
4. **Fix & verify**: Implement, run `make verify`, commit with descriptive message
5. **Mark complete**: Check off items in `tech-debts.md` with `[x]`
6. **Review per release**: At each release, review remaining items and re-prioritize

## Example — v0.0.2 Remediation

In v0.0.2, we fixed 29 of 30 identified items in two batches:

**Batch 1** (22 items): Error handling, server architecture, CLI deduplication, provider fixes, input validation

**Batch 2** (7 items): WebP quality, capabilities logging, output validation, decode errors, provider validation

Each batch was committed atomically with a detailed commit message referencing `TD-NN` IDs.
