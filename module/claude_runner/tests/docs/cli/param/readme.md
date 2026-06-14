# Parameter Tests

### Scope

- **Purpose**: Document edge case coverage for individual clr parameters.
- **Responsibility**: Index of per-parameter edge case test files covering parameter-level behavior.
- **In Scope**: All 39 clr parameters: `[MESSAGE]`, `--print`, `--model`, `--verbose`, `--no-skip-permissions`, `--interactive`, `--new-session`, `--dir`, `--max-tokens`, `--session-dir`, `--dry-run`, `--verbosity`, `--trace`, `--no-ultrathink`, `--system-prompt`, `--append-system-prompt`, `--effort`, `--no-effort-max`, `--creds`, `--timeout` (isolated/refresh), `--no-chrome`, `--no-persist`, `--json-schema`, `--mcp-config`, `--file`, `--strip-fences`, `--keep-claudecode`, `--subdir`, `--output-file`, `--expect`, `--expect-strategy`, `--expect-retries`, `--max-sessions`, `--retry-on-rate-limit`, `--retry-delay`, `--timeout` (run/ask), `--retry-on-api-error`, `--api-error-delay`, `--retry-on-unknown-error`.
- **Out of Scope**: Command-level tests (→ `command/`), parameter group interactions (→ `param_group/`).

Per-parameter edge case indices for `clr`. See [param/readme.md](../../../../docs/cli/param/readme.md) for specification.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| `01_message.md` | Edge cases for `[MESSAGE]` positional parameter | ✅ |
| `02_print.md` | Edge cases for `--print` / `-p` flag | ✅ |
| `03_model.md` | Edge cases for `--model` flag | ✅ |
| `04_verbose.md` | Edge cases for `--verbose` flag | ✅ |
| `05_no_skip_permissions.md` | Edge cases for `--no-skip-permissions` flag | ✅ |
| `06_interactive.md` | Edge cases for `--interactive` flag | ✅ |
| `07_new_session.md` | Edge cases for `--new-session` flag | ✅ |
| `08_dir.md` | Edge cases for `--dir` flag | ✅ |
| `09_max_tokens.md` | Edge cases for `--max-tokens` flag | ✅ |
| `10_session_dir.md` | Edge cases for `--session-dir` flag | ✅ |
| `11_dry_run.md` | Edge cases for `--dry-run` flag | ✅ |
| `12_verbosity.md` | Edge cases for `--verbosity` flag | ✅ |
| `13_trace.md` | Edge cases for `--trace` flag | ✅ |
| `14_no_ultrathink.md` | Edge cases for `--no-ultrathink` flag | ✅ |
| `15_system_prompt.md` | Edge cases for `--system-prompt` flag | ✅ |
| `16_append_system_prompt.md` | Edge cases for `--append-system-prompt` flag | ✅ |
| `17_effort.md` | Edge cases for `--effort` flag | ✅ |
| `18_no_effort_max.md` | Edge cases for `--no-effort-max` flag | ✅ |
| `19_creds.md` | Edge cases for `--creds` flag | ✅ |
| `20_timeout.md` | Edge cases for `--timeout` flag | ✅ |
| `21_no_chrome.md` | Edge cases for `--no-chrome` flag | ✅ |
| `22_no_persist.md` | Edge cases for `--no-persist` flag | ✅ |
| `23_json_schema.md` | Edge cases for `--json-schema` parameter | ✅ |
| `24_mcp_config.md` | Edge cases for `--mcp-config` parameter | ✅ |
| `25_file.md` | Edge cases for `--file` parameter | ✅ |
| `26_strip_fences.md` | Edge cases for `--strip-fences` flag | ✅ |
| `27_keep_claudecode.md` | Edge cases for `--keep-claudecode` flag | ✅ |
| `28_subdir.md` | Edge cases for `--subdir` parameter | ✅ |
| `29_output_file.md` | Edge cases for `--output-file` parameter | ✅ |
| `30_expect.md` | Edge cases for `--expect` parameter | ✅ |
| `31_expect_strategy.md` | Edge cases for `--expect-strategy` parameter | ✅ |
| `32_expect_retries.md` | Edge cases for `--expect-retries` parameter | ✅ |
| `33_max_sessions.md` | Edge cases for `--max-sessions` parameter | ✅ |
| `34_retry_on_rate_limit.md` | Edge cases for `--retry-on-rate-limit` parameter | ✅ |
| `35_retry_delay.md` | Edge cases for `--retry-delay` parameter | ✅ |
| `36_timeout.md` | Edge cases for `--timeout` flag (run/ask) | ✅ |
| `037_retry_on_api_error.md` | Edge cases for `--retry-on-api-error` parameter | ✅ |
| `038_api_error_delay.md` | Edge cases for `--api-error-delay` parameter | ✅ |
| `039_retry_on_unknown_error.md` | Edge cases for `--retry-on-unknown-error` parameter | ✅ |
