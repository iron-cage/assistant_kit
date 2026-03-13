# Feature Doc Entity

### Scope

- **Purpose**: Document user-facing capabilities of the claude_runner_core library for consumers building Claude Code automation.
- **Responsibility**: Index of feature doc instances covering execution control, dry-run, and command inspection.
- **In Scope**: Execution mode design, dry-run semantics, describe/describe_compact output format.
- **Out of Scope**: Builder pattern design (→ `pattern/`), API contracts (→ `api/`), type definitions (→ `data_structure/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Execution Control](001_execution_control.md) | Interactive vs. non-interactive execution modes | ✅ |
| 002 | [Dry Run](002_dry_run.md) | Command preview without process execution | ✅ |
| 003 | [Describe](003_describe.md) | Human-readable command inspection output | ✅ |
