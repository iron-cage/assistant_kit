# Parameter Tests

### Scope

- **Purpose**: Document edge case coverage for individual clr parameters.
- **Responsibility**: Index of per-parameter edge case test files covering parameter-level behavior.
- **In Scope**: All 66 clr parameters: `[MESSAGE]`, `--print`, `--model`, `--verbose`, `--no-skip-permissions`, `--interactive`, `--new-session`, `--dir`, `--max-tokens`, `--session-dir`, `--dry-run`, `--verbosity`, `--trace`, `--no-ultrathink`, `--system-prompt`, `--append-system-prompt`, `--effort`, `--no-effort-max`, `--creds`, `--timeout` (isolated/refresh), `--no-chrome`, `--no-persist`, `--json-schema`, `--mcp-config`, `--file`, `--strip-fences`, `--keep-claudecode`, `--subdir`, `--output-file`, `--expect`, `--expect-strategy`, `--max-sessions`, `--retry-on-transient`, `--transient-delay`, `--timeout` (run/ask), `--retry-on-account`, `--account-delay`, `--retry-on-auth`, `--auth-delay`, `--retry-on-service`, `--service-delay`, `--retry-on-process`, `--process-delay`, `--retry-on-validation`, `--validation-delay`, `--retry-on-runner`, `--runner-delay`, `--retry-on-unknown`, `--unknown-delay`, `--retry-override`, `--retry-override-delay`, `--retry-default`, `--retry-default-delay`, `--mode`, `--columns`, `--wide`, `--pid`, `--inspect`, `--output-style`, `--output-format`, `--max-turns`, `--allowed-tools`, `--disallowed-tools`, `--max-budget-usd`, `--add-dir`, `--fallback-model`.
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
| `33_max_sessions.md` | Edge cases for `--max-sessions` parameter | ✅ |
| `34_retry_on_transient.md` | Edge cases for `--retry-on-transient` parameter | ✅ |
| `35_transient_delay.md` | Edge cases for `--transient-delay` parameter | ✅ |
| `36_timeout.md` | Edge cases for `--timeout` flag (run/ask) | ✅ |
| `040_retry_on_account.md` | Edge cases for `--retry-on-account` parameter | ✅ |
| `041_account_delay.md` | Edge cases for `--account-delay` parameter | ✅ |
| `042_retry_on_auth.md` | Edge cases for `--retry-on-auth` parameter | ✅ |
| `043_auth_delay.md` | Edge cases for `--auth-delay` parameter | ✅ |
| `044_retry_on_service.md` | Edge cases for `--retry-on-service` parameter | ✅ |
| `045_service_delay.md` | Edge cases for `--service-delay` parameter | ✅ |
| `046_retry_on_process.md` | Edge cases for `--retry-on-process` parameter | ✅ |
| `047_process_delay.md` | Edge cases for `--process-delay` parameter | ✅ |
| `048_retry_on_validation.md` | Edge cases for `--retry-on-validation` parameter | ✅ |
| `049_validation_delay.md` | Edge cases for `--validation-delay` parameter | ✅ |
| `050_retry_on_runner.md` | Edge cases for `--retry-on-runner` parameter | ✅ |
| `051_runner_delay.md` | Edge cases for `--runner-delay` parameter | ✅ |
| `052_retry_on_unknown.md` | Edge cases for `--retry-on-unknown` parameter | ✅ |
| `053_unknown_delay.md` | Edge cases for `--unknown-delay` parameter | ✅ |
| `054_retry_override.md` | Edge cases for `--retry-override` parameter (Tier 1) | ✅ |
| `055_retry_override_delay.md` | Edge cases for `--retry-override-delay` parameter (Tier 1) | ✅ |
| `056_retry_default.md` | Edge cases for `--retry-default` parameter (Tier 3) | ✅ |
| `057_retry_default_delay.md` | Edge cases for `--retry-default-delay` parameter (Tier 3) | ✅ |
| `058_mode.md` | Edge cases for `--mode` parameter (ps filter) | ✅ |
| `059_columns.md` | Edge cases for `--columns` parameter (ps column selector) | ✅ |
| `060_wide.md` | Edge cases for `--wide` flag (ps wide output) | ✅ |
| `068_pid.md` | Edge cases for `--pid` parameter (ps PID filter) | ✅ |
| `069_inspect.md` | Edge cases for `--inspect` flag (ps key:value output) | ✅ |
| `061_output_format.md` | Edge cases for `--output-format` parameter | ✅ |
| `062_max_turns.md` | Edge cases for `--max-turns` parameter | ✅ |
| `063_allowed_tools.md` | Edge cases for `--allowed-tools` parameter | ✅ |
| `064_disallowed_tools.md` | Edge cases for `--disallowed-tools` parameter | ✅ |
| `065_max_budget_usd.md` | Edge cases for `--max-budget-usd` parameter | ✅ |
| `066_add_dir.md` | Edge cases for `--add-dir` parameter | ✅ |
| `067_fallback_model.md` | Edge cases for `--fallback-model` parameter | ✅ |
| `070_output_style.md` | Edge cases for `--output-style` parameter (EC-01–EC-13) | ✅ |
