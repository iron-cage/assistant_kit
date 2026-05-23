# Parameter Tests

### Scope

- **Purpose**: Document edge case coverage for individual clr parameters.
- **Responsibility**: Index of per-parameter edge case test files covering parameter-level behavior.
- **In Scope**: All 27 clr parameter edge case files.
- **Out of Scope**: Command-level tests (→ `command/`), parameter group interactions (→ `param_group/`).

Per-parameter edge case indices for `clr`. See [param/readme.md](../../../../docs/cli/param/readme.md) for specification.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 001_message.md | Edge cases for `[MESSAGE]` positional parameter |
| 002_print.md | Edge cases for `--print` / `-p` flag |
| 003_model.md | Edge cases for `--model` flag |
| 004_verbose.md | Edge cases for `--verbose` flag |
| 005_no_skip_permissions.md | Edge cases for `--no-skip-permissions` flag |
| 006_interactive.md | Edge cases for `--interactive` flag |
| 007_new_session.md | Edge cases for `--new-session` flag |
| 008_dir.md | Edge cases for `--dir` flag |
| 009_max_tokens.md | Edge cases for `--max-tokens` flag |
| 010_session_dir.md | Edge cases for `--session-dir` flag |
| 011_dry_run.md | Edge cases for `--dry-run` flag |
| 012_verbosity.md | Edge cases for `--verbosity` flag |
| 013_trace.md | Edge cases for `--trace` flag |
| 014_no_ultrathink.md | Edge cases for `--no-ultrathink` flag |
| 015_system_prompt.md | Edge cases for `--system-prompt` flag |
| 016_append_system_prompt.md | Edge cases for `--append-system-prompt` flag |
| 017_effort.md | Edge cases for `--effort` flag |
| 018_no_effort_max.md | Edge cases for `--no-effort-max` flag |
| 019_creds.md | Edge cases for `--creds` flag |
| 020_timeout.md | Edge cases for `--timeout` flag |
| 021_no_chrome.md | Edge cases for `--no-chrome` flag |
| 022_no_persist.md | Edge cases for `--no-persist` flag |
| 023_json_schema.md | Edge cases for `--json-schema` parameter |
| 024_mcp_config.md | Edge cases for `--mcp-config` parameter |
| 025_file.md | Edge cases for `--file` parameter |
| 026_strip_fences.md | Edge cases for `--strip-fences` flag |
| 027_keep_claudecode.md | Edge cases for `--keep-claudecode` flag |

### Index

| Parameter | File | Tests |
|-----------|------|-------|
| `[MESSAGE]` | [001_message.md](001_message.md) | 6 EC |
| `--print` | [002_print.md](002_print.md) | 6 EC |
| `--model` | [003_model.md](003_model.md) | 6 EC |
| `--verbose` | [004_verbose.md](004_verbose.md) | 6 EC |
| `--no-skip-permissions` | [005_no_skip_permissions.md](005_no_skip_permissions.md) | 6 EC |
| `--interactive` | [006_interactive.md](006_interactive.md) | 6 EC |
| `--new-session` | [007_new_session.md](007_new_session.md) | 6 EC |
| `--dir` | [008_dir.md](008_dir.md) | 6 EC |
| `--max-tokens` | [009_max_tokens.md](009_max_tokens.md) | 6 EC |
| `--session-dir` | [010_session_dir.md](010_session_dir.md) | 6 EC |
| `--dry-run` | [011_dry_run.md](011_dry_run.md) | 6 EC |
| `--verbosity` | [012_verbosity.md](012_verbosity.md) | 6 EC |
| `--trace` | [013_trace.md](013_trace.md) | 8 EC |
| `--no-ultrathink` | [014_no_ultrathink.md](014_no_ultrathink.md) | 6 EC |
| `--system-prompt` | [015_system_prompt.md](015_system_prompt.md) | 6 EC |
| `--append-system-prompt` | [016_append_system_prompt.md](016_append_system_prompt.md) | 6 EC |
| `--effort` | [017_effort.md](017_effort.md) | 8 EC |
| `--no-effort-max` | [018_no_effort_max.md](018_no_effort_max.md) | 6 EC |
| `--creds` | [019_creds.md](019_creds.md) | 6 EC |
| `--timeout` | [020_timeout.md](020_timeout.md) | 6 EC |
| `--no-chrome` | [021_no_chrome.md](021_no_chrome.md) | 6 EC |
| `--no-persist` | [022_no_persist.md](022_no_persist.md) | 6 EC |
| `--json-schema` | [023_json_schema.md](023_json_schema.md) | 6 EC |
| `--mcp-config` | [024_mcp_config.md](024_mcp_config.md) | 6 EC |
| `--file` | [025_file.md](025_file.md) | 6 EC |
| `--strip-fences` | [026_strip_fences.md](026_strip_fences.md) | 6 EC |
| `--keep-claudecode` | [027_keep_claudecode.md](027_keep_claudecode.md) | 6 EC |
