# Parameter Tests

### Scope

- **Purpose**: Document edge case coverage for individual clr parameters.
- **Responsibility**: Index of per-parameter edge case test files covering parameter-level behavior.
- **In Scope**: All active clr parameters: `[MESSAGE]`, `--print`, `--model`, `--verbose`, `--no-skip-permissions`, `--interactive`, `--new-session`, `--dir`, `--max-tokens`, `--session-dir`, `--dry-run`, `--quiet`, `--trace`, `--no-ultrathink`, `--system-prompt`, `--append-system-prompt`, `--effort`, `--no-effort-max`, `--creds`, `--timeout` (isolated/refresh), `--no-chrome`, `--no-persist`, `--json-schema`, `--mcp-config`, `--file`, `--strip-fences`, `--keep-claudecode`, `--topic`, `--output-file`, `--expect`, `--expect-strategy`, `--max-sessions`, `--retry-on-transient`, `--transient-delay`, `--timeout` (run/ask), `--retry-on-account`, `--account-delay`, `--retry-on-auth`, `--auth-delay`, `--retry-on-service`, `--service-delay`, `--retry-on-process`, `--process-delay`, `--retry-on-validation`, `--validation-delay`, `--retry-on-runner`, `--runner-delay`, `--retry-on-unknown`, `--unknown-delay`, `--retry-override`, `--retry-override-delay`, `--retry-default`, `--retry-default-delay`, `--mode`, `--columns`, `--wide`, `--pid`, `--inspect`, `--output-style`, `--summary-fields`, `--journal`, `--journal-dir`, `--output-format`, `--input-format`, `--max-turns`, `--allowed-tools`, `--disallowed-tools`, `--max-budget-usd`, `--add-dir`, `--fallback-model`, `--no-compact-window`, `--args-file`, `--from`, `--name`, `--category`, `--value`, `--gate-poll-secs`, `--gate-max-attempts`, `--gate-stale-secs`, `CLR_REMAINING_TIMEOUT_SECS`, `--no-stdin`, `--global`, `--topic-mode`. (`--verbosity` DEPRECATED → `012_verbosity.md` deprecated)
- **Out of Scope**: Command-level tests (→ `command/`), parameter group interactions (→ `param_group/`).

Per-parameter edge case indices for `clr`. See [param/readme.md](../../../../docs/cli/param/readme.md) for specification.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| [001_message.md](001_message.md) | Edge cases for `[MESSAGE]` positional parameter | ✅ |
| [002_print.md](002_print.md) | Edge cases for `--print` / `-p` flag | ✅ |
| [003_model.md](003_model.md) | Edge cases for `--model` flag | ✅ |
| [004_verbose.md](004_verbose.md) | Edge cases for `--verbose` flag | ✅ |
| [005_no_skip_permissions.md](005_no_skip_permissions.md) | Edge cases for `--no-skip-permissions` flag | ✅ |
| [006_interactive.md](006_interactive.md) | Edge cases for `--interactive` flag | ✅ |
| [007_new_session.md](007_new_session.md) | Edge cases for `--new-session` flag | ✅ |
| [008_dir.md](008_dir.md) | Edge cases for `--dir` flag | ✅ |
| [009_max_tokens.md](009_max_tokens.md) | Edge cases for `--max-tokens` flag | ✅ |
| [010_session_dir.md](010_session_dir.md) | Edge cases for `--session-dir` flag (DEPRECATED, inert — BUG-493) | ✅ |
| [011_dry_run.md](011_dry_run.md) | Edge cases for `--dry-run` flag | ✅ |
| [012_verbosity.md](012_verbosity.md) | Edge cases for `--verbosity` flag (DEPRECATED — `--verbosity` removed) | ⚠️ |
| [074_quiet.md](074_quiet.md) | Edge cases for `--quiet` flag (suppress non-fatal runner diagnostics) | ✅ |
| [013_trace.md](013_trace.md) | Edge cases for `--trace` flag | ✅ |
| [014_no_ultrathink.md](014_no_ultrathink.md) | Edge cases for `--no-ultrathink` flag | ✅ |
| [015_system_prompt.md](015_system_prompt.md) | Edge cases for `--system-prompt` flag | ✅ |
| [016_append_system_prompt.md](016_append_system_prompt.md) | Edge cases for `--append-system-prompt` flag | ✅ |
| [017_effort.md](017_effort.md) | Edge cases for `--effort` flag | ✅ |
| [018_no_effort_max.md](018_no_effort_max.md) | Edge cases for `--no-effort-max` flag | ✅ |
| [019_creds.md](019_creds.md) | Edge cases for `--creds` flag | ✅ |
| [020_timeout.md](020_timeout.md) | Edge cases for `--timeout` flag | ✅ |
| [021_no_chrome.md](021_no_chrome.md) | Edge cases for `--no-chrome` flag | ✅ |
| [022_no_persist.md](022_no_persist.md) | Edge cases for `--no-persist` flag | ✅ |
| [023_json_schema.md](023_json_schema.md) | Edge cases for `--json-schema` parameter | ✅ |
| [024_mcp_config.md](024_mcp_config.md) | Edge cases for `--mcp-config` parameter | ✅ |
| [025_file.md](025_file.md) | Edge cases for `--file` parameter | ✅ |
| [026_strip_fences.md](026_strip_fences.md) | Edge cases for `--strip-fences` flag | ✅ |
| [027_keep_claudecode.md](027_keep_claudecode.md) | Edge cases for `--keep-claudecode` flag | ✅ |
| [028_topic.md](028_topic.md) | Edge cases for `--topic` parameter | ✅ |
| [029_output_file.md](029_output_file.md) | Edge cases for `--output-file` parameter | ✅ |
| [030_expect.md](030_expect.md) | Edge cases for `--expect` parameter | ✅ |
| [031_expect_strategy.md](031_expect_strategy.md) | Edge cases for `--expect-strategy` parameter | ✅ |
| [033_max_sessions.md](033_max_sessions.md) | Edge cases for `--max-sessions` parameter | ✅ |
| [034_retry_on_transient.md](034_retry_on_transient.md) | Edge cases for `--retry-on-transient` parameter | ✅ |
| [035_transient_delay.md](035_transient_delay.md) | Edge cases for `--transient-delay` parameter | ✅ |
| [036_timeout.md](036_timeout.md) | Edge cases for `--timeout` flag (run/ask) | ✅ |
| [040_retry_on_account.md](040_retry_on_account.md) | Edge cases for `--retry-on-account` parameter | ✅ |
| [041_account_delay.md](041_account_delay.md) | Edge cases for `--account-delay` parameter | ✅ |
| [042_retry_on_auth.md](042_retry_on_auth.md) | Edge cases for `--retry-on-auth` parameter | ✅ |
| [043_auth_delay.md](043_auth_delay.md) | Edge cases for `--auth-delay` parameter | ✅ |
| [044_retry_on_service.md](044_retry_on_service.md) | Edge cases for `--retry-on-service` parameter | ✅ |
| [045_service_delay.md](045_service_delay.md) | Edge cases for `--service-delay` parameter | ✅ |
| [046_retry_on_process.md](046_retry_on_process.md) | Edge cases for `--retry-on-process` parameter | ✅ |
| [047_process_delay.md](047_process_delay.md) | Edge cases for `--process-delay` parameter | ✅ |
| [048_retry_on_validation.md](048_retry_on_validation.md) | Edge cases for `--retry-on-validation` parameter | ✅ |
| [049_validation_delay.md](049_validation_delay.md) | Edge cases for `--validation-delay` parameter | ✅ |
| [050_retry_on_runner.md](050_retry_on_runner.md) | Edge cases for `--retry-on-runner` parameter | ✅ |
| [051_runner_delay.md](051_runner_delay.md) | Edge cases for `--runner-delay` parameter | ✅ |
| [052_retry_on_unknown.md](052_retry_on_unknown.md) | Edge cases for `--retry-on-unknown` parameter | ✅ |
| [053_unknown_delay.md](053_unknown_delay.md) | Edge cases for `--unknown-delay` parameter | ✅ |
| [054_retry_override.md](054_retry_override.md) | Edge cases for `--retry-override` parameter (Tier 1) | ✅ |
| [055_retry_override_delay.md](055_retry_override_delay.md) | Edge cases for `--retry-override-delay` parameter (Tier 1) | ✅ |
| [056_retry_default.md](056_retry_default.md) | Edge cases for `--retry-default` parameter (Tier 3) | ✅ |
| [057_retry_default_delay.md](057_retry_default_delay.md) | Edge cases for `--retry-default-delay` parameter (Tier 3) | ✅ |
| [058_mode.md](058_mode.md) | Edge cases for `--mode` parameter (ps filter) | ✅ |
| [059_columns.md](059_columns.md) | Edge cases for `--columns` parameter (ps column selector) | ✅ |
| [060_wide.md](060_wide.md) | Edge cases for `--wide` flag (ps wide output) | ✅ |
| [068_pid.md](068_pid.md) | Edge cases for `--pid` parameter (ps PID filter) | ✅ |
| [069_inspect.md](069_inspect.md) | Edge cases for `--inspect` flag (ps key:value output) | ✅ |
| [061_output_format.md](061_output_format.md) | Edge cases for `--output-format` parameter | ✅ |
| [062_max_turns.md](062_max_turns.md) | Edge cases for `--max-turns` parameter | ✅ |
| [063_allowed_tools.md](063_allowed_tools.md) | Edge cases for `--allowed-tools` parameter | ✅ |
| [064_disallowed_tools.md](064_disallowed_tools.md) | Edge cases for `--disallowed-tools` parameter | ✅ |
| [065_max_budget_usd.md](065_max_budget_usd.md) | Edge cases for `--max-budget-usd` parameter | ✅ |
| [066_add_dir.md](066_add_dir.md) | Edge cases for `--add-dir` parameter | ✅ |
| [067_fallback_model.md](067_fallback_model.md) | Edge cases for `--fallback-model` parameter | ✅ |
| [070_output_style.md](070_output_style.md) | Edge cases for `--output-style` parameter (EC-01–EC-14, IT-7) | ✅ |
| [071_summary_fields.md](071_summary_fields.md) | Edge cases for `--summary-fields` parameter (EC-01–EC-12) | ✅ |
| [072_journal.md](072_journal.md) | Edge cases for `--journal` parameter | ✅ |
| [073_journal_dir.md](073_journal_dir.md) | Edge cases for `--journal-dir` parameter | ✅ |
| [075_args_file.md](075_args_file.md) | Edge cases for `--args-file` parameter | ✅ |
| [076_from.md](076_from.md) | Edge cases for `--from` parameter | ✅ |
| [077_no_compact_window.md](077_no_compact_window.md) | Edge cases for `--no-compact-window` flag (suppress CLAUDE_CODE_AUTO_COMPACT_WINDOW) | ⏳ |
| [078_name.md](078_name.md) | Edge cases for `--name` parameter (tools name filter) | ⏳ |
| [079_category.md](079_category.md) | Edge cases for `--category` parameter (tools category filter) | ⏳ |
| [080_value.md](080_value.md) | Edge cases for `--value` parameter (tools bare value output) | ⏳ |
| [081_input_format.md](081_input_format.md) | Edge cases for `--input-format` parameter (IT-1, IT-1b, IT-2, IT-3) | ✅ |
| [082_gate_poll_secs.md](082_gate_poll_secs.md) | Edge cases for `--gate-poll-secs` parameter (concurrency gate poll interval) | ✅ |
| [083_gate_max_attempts.md](083_gate_max_attempts.md) | Edge cases for `--gate-max-attempts` parameter (concurrency gate attempt limit) | ✅ |
| [084_gate_stale_secs.md](084_gate_stale_secs.md) | Edge cases for `--gate-stale-secs` parameter (concurrency gate staleness reclaim) | ✅ |
| [085_gate_remaining_timeout_secs.md](085_gate_remaining_timeout_secs.md) | Edge cases for `CLR_REMAINING_TIMEOUT_SECS` env var (gate remaining timeout budget) | ✅ |
| [086_no_stdin.md](086_no_stdin.md) | Edge cases for `--no-stdin` flag (stdin opt-out; BUG-492 reproducers) | ✅ |
| [087_global.md](087_global.md) | Edge cases for `--global` flag (topic base redirection, precedence, env pair) | ✅ |
| [088_topic_mode.md](088_topic_mode.md) | Fork-topic suite: mode selection, fork/resume argv, guards, registry, `topics --file`/listing, auto-naming (F01–F18) | ✅ |
| [089_keep_clone.md](089_keep_clone.md) | Edge cases for `--keep-clone` (preserve vs default re-clone of an existing destination session copy) | ✅ |
