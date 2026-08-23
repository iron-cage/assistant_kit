# Algorithm Doc Entity

### Scope

- **Purpose**: Document computational procedures with non-obvious design rationale.
- **Responsibility**: Index of algorithm doc instances covering procedure design, tradeoffs, and correctness guarantees.
- **In Scope**: Agent session discovery across storage layouts; inference of which sessions are currently running.
- **Out of Scope**: CLI command specs (→ `../cli/command/`), system invariants (→ `../invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Agent Session Tracking](001_agent_session_tracking.md) | Discover and enumerate agent sessions across flat and hierarchical storage layouts | ✅ |
| 002 | [Session Liveness](002_session_liveness.md) | Infer which projects have a Claude Code process attached, and which conversation it drives | ✅ |
