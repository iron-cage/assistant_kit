# Doc Entities

## Master Doc Entities Table

| Entity | Purpose | Master File | Instances |
|--------|---------|-------------|----------:|
| `algorithm/` | Path computation algorithm specifications | [algorithm/readme.md](algorithm/readme.md) | 3 |
| `api/` | Public API contract for the runner crate | [api/readme.md](api/readme.md) | 1 |
| `cli/command/` | CLI command specifications | [cli/command/readme.md](cli/command/readme.md) | 10 |
| `cli/param/` | CLI parameter specifications | [cli/param/readme.md](cli/param/readme.md) | 72 |
| `cli/param_group/` | CLI parameter group definitions | [cli/param_group/readme.md](cli/param_group/readme.md) | 6 |
| `cli/parity/` | Cross-command behavioral parity comparison docs | [cli/parity/readme.md](cli/parity/readme.md) | 2 |
| `cli/type/` | CLI type definitions | [cli/type/readme.md](cli/type/readme.md) | 14 |
| `cli/user_story/` | User story catalog for runner use cases | [cli/user_story/readme.md](cli/user_story/readme.md) | 29 |
| `feature/` | Behavioral requirements for the runner | [feature/readme.md](feature/readme.md) | 6 |
| `invariant/` | Measurable constraints for runner behavior | [invariant/readme.md](invariant/readme.md) | 14 |
| `variable/` | Output variable definitions for the six CLAUDE_* paths | [variable/readme.md](variable/readme.md) | 6 |
| `tests/docs/api/` | Per-API test case specifications | [../tests/docs/api/readme.md](../tests/docs/api/readme.md) | 1 |
| `tests/docs/feature/` | Per-feature test case specifications | [../tests/docs/feature/readme.md](../tests/docs/feature/readme.md) | 5 |
| `tests/docs/cli/command/` | Per-command integration test case specifications | [../tests/docs/cli/command/readme.md](../tests/docs/cli/command/readme.md) | 10 |
| `tests/docs/cli/env_param/` | Per-env-parameter edge case test specifications | [../tests/docs/cli/env_param/readme.md](../tests/docs/cli/env_param/readme.md) | 2 |
| `tests/docs/cli/param/` | Per-parameter edge case test specifications | [../tests/docs/cli/param/readme.md](../tests/docs/cli/param/readme.md) | 72 |
| `tests/docs/cli/param_group/` | Per-parameter-group interaction test specifications | [../tests/docs/cli/param_group/readme.md](../tests/docs/cli/param_group/readme.md) | 6 |
| `tests/docs/cli/parity/` | Per-parity-matrix cross-command test specifications | [../tests/docs/cli/parity/readme.md](../tests/docs/cli/parity/readme.md) | 2 |
| `tests/docs/cli/type/` | Per-type validation edge case test specifications | [../tests/docs/cli/type/readme.md](../tests/docs/cli/type/readme.md) | 14 |
| `tests/docs/cli/user_story/` | Per-user-story end-to-end test specifications | [../tests/docs/cli/user_story/readme.md](../tests/docs/cli/user_story/readme.md) | 29 |
| `tests/docs/invariant/` | Per-invariant test case specifications | [../tests/docs/invariant/readme.md](../tests/docs/invariant/readme.md) | 13 |

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| algorithm | 001 | Path Encoding | [algorithm/001_path_encoding.md](algorithm/001_path_encoding.md) |
| algorithm | 002 | Git Root Detection | [algorithm/002_git_root_detection.md](algorithm/002_git_root_detection.md) |
| algorithm | 003 | Session File Selection | [algorithm/003_session_file_selection.md](algorithm/003_session_file_selection.md) |
| api | 001 | Public API | [api/001_public_api.md](api/001_public_api.md) |
| feature | 001 | Runner Tool | [feature/001_runner_tool.md](feature/001_runner_tool.md) |
| feature | 002 | Journaling Integration | [feature/002_journaling_integration.md](feature/002_journaling_integration.md) |
| feature | 003 | Retry Hierarchy | [feature/003_retry_hierarchy.md](feature/003_retry_hierarchy.md) |
| feature | 004 | JSON Config Loading | [feature/004_json_config.md](feature/004_json_config.md) |
| feature | 005 | Session Path Resolution | [feature/005_session_path_resolution.md](feature/005_session_path_resolution.md) |
| feature | 006 | CLI Design | [feature/006_cli_design.md](feature/006_cli_design.md) |
| invariant | 001 | Default Flags | [invariant/001_default_flags.md](invariant/001_default_flags.md) |
| invariant | 002 | Dependency Constraints | [invariant/002_dep_constraints.md](invariant/002_dep_constraints.md) |
| invariant | 003 | Command Naming | [invariant/003_command_naming.md](invariant/003_command_naming.md) |
| invariant | 004 | Trace Universality | [invariant/004_trace_universality.md](invariant/004_trace_universality.md) |
| invariant | 005 | Isolated Subprocess Defaults | [invariant/005_isolated_subprocess_defaults.md](invariant/005_isolated_subprocess_defaults.md) |
| invariant | 006 | Exit Codes | [invariant/006_exit_codes.md](invariant/006_exit_codes.md) |
| invariant | 007 | Print Mode Timeout | [invariant/007_print_mode_timeout.md](invariant/007_print_mode_timeout.md) |
| invariant | 008 | Render Summary Gate | [invariant/008_render_summary_gate.md](invariant/008_render_summary_gate.md) |
| invariant | 009 | Session Mismatch Detection | [invariant/009_session_mismatch_detection.md](invariant/009_session_mismatch_detection.md) |
| invariant | 010 | Container-Only Test Execution | [invariant/010_container_only_test_execution.md](invariant/010_container_only_test_execution.md) |
| invariant | 011 | Session Source Isolation | [invariant/011_session_source_isolation.md](invariant/011_session_source_isolation.md) |
| invariant | 012 | Gate Slot Atomicity | [invariant/012_gate_slot_atomicity.md](invariant/012_gate_slot_atomicity.md) |
| invariant | 013 | Slot-Wait Message Differentiation | [invariant/013_slot_wait_message_differentiation.md](invariant/013_slot_wait_message_differentiation.md) |
| invariant | 014 | JSON String Extraction Escape Handling | [invariant/014_json_string_extraction_escape_handling.md](invariant/014_json_string_extraction_escape_handling.md) |
| variable | 001 | CLAUDE_HOME | [variable/001_claude_home.md](variable/001_claude_home.md) |
| variable | 002 | CLAUDE_PROJECTS_DIR | [variable/002_claude_projects_dir.md](variable/002_claude_projects_dir.md) |
| variable | 003 | CLAUDE_SESSION_DIR | [variable/003_claude_session_dir.md](variable/003_claude_session_dir.md) |
| variable | 004 | CLAUDE_MEMORY_DIR | [variable/004_claude_memory_dir.md](variable/004_claude_memory_dir.md) |
| variable | 005 | CLAUDE_MEMORY_FILE | [variable/005_claude_memory_file.md](variable/005_claude_memory_file.md) |
| variable | 006 | CLAUDE_SESSION_FILE | [variable/006_claude_session_file.md](variable/006_claude_session_file.md) |
