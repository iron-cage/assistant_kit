# Invariant Doc Entity

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
| `010_container_only_test_execution.md` | Test cases for the container-only test execution invariant | ✅ |
| `011_session_source_isolation.md` | Test cases for the session source isolation invariant | ⏳ |
| `012_gate_slot_atomicity.md` | Test cases for the gate slot atomicity invariant (BUG-387, BUG-392, BUG-402, BUG-404) | ✅ |
| `013_slot_wait_message_differentiation.md` | Test cases for the slot-wait message differentiation invariant (BUG-393) | ⏳ |
| `014_json_string_extraction_escape_handling.md` | Test cases for the JSON string extraction escape handling invariant (BUG-394, BUG-395) | ⏳ |
| `015_tools_array_doc_sync.md` | Test cases for the tools array/doc sync bijection invariant | ⏳ |
