# Feature Doc Entity

### Scope

- **Purpose**: Document user-facing capabilities of the `claude_session_core` library for consumers that need to know which Claude Code sessions are running and what they are doing.
- **Responsibility**: Index of feature doc instances covering the live-session registry scan and turn-boundary detection.
- **In Scope**: Reading and parsing `sessions/*.json`, liveness classification, `TurnWatcher` edge detection, the background-reporting caveat.
- **Out of Scope**: Reading conversation transcripts (→ `claude_storage_core`), hosting a session on a terminal (→ `claude_pty_core`), acting on a detected boundary (→ `claude_daemon_core`), invariant constraints (→ `invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Registry Scan](001_registry_scan.md) | Read the live-session registry and classify each entry's liveness | ✅ |
| 002 | [Turn Detection](002_turn_detection.md) | Derive turn boundaries from observed status transitions | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating feature doc instances | ✅ |
