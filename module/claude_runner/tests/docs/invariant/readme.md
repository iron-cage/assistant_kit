# Invariant Doc Tests

### Scope

- **Purpose**: Document test case planning for invariant doc instances in `docs/invariant/`.
- **Responsibility**: Index of per-invariant-doc test case spec files.
- **In Scope**: All `docs/invariant/` doc instances.
- **Out of Scope**: CLI parameter tests (→ `../cli/`), feature tests (→ `../feature/`).

Per-invariant-doc test case indices for `claude_runner`. See [invariant/readme.md](../../../docs/invariant/readme.md) for the invariant doc instances.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| `001_default_flags.md` | Test cases for the default flags invariant | ✅ |
| `002_dep_constraints.md` | Test cases for the dependency constraints invariant | ✅ |
| `003_command_naming.md` | Test cases for the command naming invariant | ✅ |
| `004_trace_universality.md` | Test cases for the trace universality invariant | ✅ |
| `005_isolated_subprocess_defaults.md` | Test cases for the isolated/refresh subprocess defaults invariant | ✅ |
| `006_exit_codes.md` | Test cases for the exit code contract invariant | ✅ |
| `007_print_mode_timeout.md` | Test cases for the print-mode timeout default invariant | ✅ |
| `008_render_summary_gate.md` | Test cases for the render_summary() gate field invariant | ✅ |
| `009_session_mismatch_detection.md` | Test cases for the session mismatch detection invariant | ✅ |
