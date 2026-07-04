# Invariant Doc Entity

### Scope

- **Purpose**: Document non-functional constraints that claude_runner must always satisfy.
- **Responsibility**: Index of invariant doc instances covering default flag injection, dependency constraints, command naming convention, trace universality, isolated/refresh subprocess defaults, exit code contract, print-mode timeout default, render_summary gate field, session mismatch detection, and container-only test execution.
- **In Scope**: Default-on flags (`--dangerously-skip-permissions`, `-c`, `--chrome`), zero consumer workspace dependency rule, binary dependency gating, command naming convention (bare words vs `--` flags), `--trace` universality across all subprocess-executing commands, isolated/refresh subprocess defaults (model, effort, flags, CLAUDE.md, timeout semantics), exit code contract (exit 0/1/2/3/128+N mapping and exit-2 collision disambiguation), print-mode 1-hour watchdog default (`DEFAULT_PRINT_TIMEOUT_SECS = 3600`), `render_summary()` gate field (`"type":"result"` invariant; optional fields use `.unwrap_or_default()`), session mismatch detection (diagnostic warning when `-c` resumes wrong session), container-only test execution (all tests run inside runbox; host-native execution is a hard error at shell and nextest layers), session source isolation (`--session-from` reads from source dir, writes to target dir).
- **Out of Scope**: Feature behavior (→ `feature/`), API contracts (→ `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Default Flags](001_default_flags.md) | Automatic flag injection and opt-out mechanism | ✅ |
| 002 | [Dependency Constraints](002_dep_constraints.md) | Zero consumer workspace deps, binary deps gated by enabled, no routines.rs | ✅ |
| 003 | [Command Naming](003_command_naming.md) | Commands are bare words; parameters use `--`/`-` prefix | ✅ |
| 004 | [Trace Universality](004_trace_universality.md) | Every subprocess-executing command must support `--trace` | ✅ |
| 005 | [Isolated Subprocess Defaults](005_isolated_subprocess_defaults.md) | Model, effort, flags, CLAUDE.md, and timeout semantics for `isolated`/`refresh` | ✅ |
| 006 | [Exit Code Contract](006_exit_codes.md) | Complete exit code table, CLR-layer ad-hoc codes, and exit-2 disambiguation | ✅ |
| 007 | [Print-Mode Timeout Default](007_print_mode_timeout.md) | 1-hour safety watchdog default for `run_print_mode()`; interactive mode stays unbounded | ✅ |
| 008 | [render_summary() Gate Field](008_render_summary_gate.md) | `render_summary()` must gate on `"type":"result"` (invariant); optional fields use `.unwrap_or_default()` | ✅ |
| 009 | [Session Mismatch Detection](009_session_mismatch_detection.md) | Diagnostic warning when `-c` resumes a different session than expected (BUG-320 hardening) | ✅ |
| 010 | [Container-Only Test Execution](010_container_only_test_execution.md) | All tests run inside runbox; host-native execution is a hard error at shell and nextest layers | ✅ |
| 011 | [Session Source Isolation](011_session_source_isolation.md) | `--session-from` reads session from source dir, writes to target dir; source files never modified | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |
