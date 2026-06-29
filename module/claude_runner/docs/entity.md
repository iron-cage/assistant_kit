# Doc Entities

## Master Doc Entities Table

| Entity | Purpose | Master File | Instances |
|--------|---------|-------------|----------:|
| `api/` | Public API contract for the runner crate | [api/readme.md](api/readme.md) | 1 |
| `cli/command/` | CLI command specifications | [cli/command/readme.md](cli/command/readme.md) | 8 |
| `cli/param/` | CLI parameter specifications | [cli/param/readme.md](cli/param/readme.md) | 69 |
| `cli/param_group/` | CLI parameter group definitions | [cli/param_group/readme.md](cli/param_group/readme.md) | 5 |
| `cli/parity/` | Claude binary parity validation docs | [cli/parity/readme.md](cli/parity/readme.md) | 2 |
| `cli/type/` | CLI type definitions | [cli/type/readme.md](cli/type/readme.md) | 14 |
| `cli/user_story/` | User story catalog for runner use cases | [cli/user_story/readme.md](cli/user_story/readme.md) | 27 |
| `feature/` | Behavioral requirements for the runner | [feature/readme.md](feature/readme.md) | 2 |
| `invariant/` | Measurable constraints for runner behavior | [invariant/readme.md](invariant/readme.md) | 9 |
| `tests/docs/api/` | Per-API test case specifications | [../../tests/docs/api/readme.md](../../tests/docs/api/readme.md) | 0 |
| `tests/docs/feature/` | Per-feature test case specifications | [../../tests/docs/feature/readme.md](../../tests/docs/feature/readme.md) | 2 |
| `tests/docs/invariant/` | Per-invariant test case specifications | [../../tests/docs/invariant/readme.md](../../tests/docs/invariant/readme.md) | 9 |

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| api | 001 | Public API | [api/001_public_api.md](api/001_public_api.md) |
| feature | 001 | Runner Tool | [feature/001_runner_tool.md](feature/001_runner_tool.md) |
| feature | 002 | Journaling Integration | [feature/002_journaling_integration.md](feature/002_journaling_integration.md) |
| invariant | 001 | Default Flags | [invariant/001_default_flags.md](invariant/001_default_flags.md) |
| invariant | 002 | Dependency Constraints | [invariant/002_dep_constraints.md](invariant/002_dep_constraints.md) |
| invariant | 003 | Command Naming | [invariant/003_command_naming.md](invariant/003_command_naming.md) |
| invariant | 004 | Trace Universality | [invariant/004_trace_universality.md](invariant/004_trace_universality.md) |
| invariant | 005 | Isolated Subprocess Defaults | [invariant/005_isolated_subprocess_defaults.md](invariant/005_isolated_subprocess_defaults.md) |
| invariant | 006 | Exit Codes | [invariant/006_exit_codes.md](invariant/006_exit_codes.md) |
| invariant | 007 | Print Mode Timeout | [invariant/007_print_mode_timeout.md](invariant/007_print_mode_timeout.md) |
| invariant | 008 | Render Summary Gate | [invariant/008_render_summary_gate.md](invariant/008_render_summary_gate.md) |
| invariant | 009 | Session Mismatch Detection | [invariant/009_session_mismatch_detection.md](invariant/009_session_mismatch_detection.md) |
