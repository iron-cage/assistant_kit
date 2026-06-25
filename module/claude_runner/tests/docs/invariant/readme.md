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
| `01_default_flags.md` | Test cases for the default flags invariant | ✅ |
| `02_dep_constraints.md` | Test cases for the dependency constraints invariant | ✅ |
| `03_command_naming.md` | Test cases for the command naming invariant | ✅ |
| `04_trace_universality.md` | Test cases for the trace universality invariant | ✅ |
| `05_isolated_subprocess_defaults.md` | Test cases for the isolated/refresh subprocess defaults invariant | ✅ |
| `06_exit_codes.md` | Test cases for the exit code contract invariant | ✅ |
| `07_print_mode_timeout.md` | Test cases for the print-mode timeout default invariant | ✅ |
| `08_render_summary_gate.md` | Test cases for the render_summary() gate field invariant | ✅ |
