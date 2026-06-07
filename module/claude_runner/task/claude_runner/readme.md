<!-- task_system_metadata
type: local
version: 1.0
crate: claude_runner
root: null
last_sync: null
-->

# Task Registry — claude_runner

### Scope

Task work items scoped to the `claude_runner` crate.

### Responsibility Table

| Path | Responsibility |
|------|----------------|
| `unverified/` | Unverified task files awaiting Verification Gate |
| `completed/` | Completed task files (all validation passed) |
| `cancelled/` | Cancelled task files (reopenable) |
| `bug/` | Bug reports for confirmed or filed defects (BUG-NNN global namespace) |
| `actors/` | Actors Registry — canonical identity for all task actors |
| `action_plan/` | Per-actor action plan files |

## Tasks Index

| Order | ID | Advisability | Value | Easiness | Safety | Priority | State | Executor | Task | Purpose |
|-------|----|----|---|---|---|---|---|---|---|---|
| 1 | 001 | 0 | 8 | 6 | 9 | 0 | ✅ (Completed) | ai | [Test Surface Remediation](completed/001_test_surface_remediation.md) | Fix all 18 test surface spec violations and create missing user story coverage |
| 2 | 002 | 0 | 7 | 8 | 9 | 0 | ✅ (Completed) | ai | [US-16 CLI Discoverability Tests](completed/002_us16_cli_discoverability_tests.md) | Add 4 Rust test functions for user story 016 (CLI Discoverability) |
| 3 | 003 | 0 | 9 | 7 | 9 | 0 | ✅ (Completed) | ai | [BUG-212: run subcommand stripping](completed/003_bug_212_run_subcommand_stripping.md) | Fix `clr run` treating "run" as message argument |
| 4 | 004 | 0 | 7 | 9 | 9 | 0 | ✅ (Completed) | ai | [BUG-213: verbosity test env isolation](completed/004_bug_213_test_env_isolation.md) | Fix 4 verbosity tests failing under ambient CLR_TRACE |
| 5 | 005 | 0 | 8 | 9 | 9 | 0 | ✅ (Completed) | ai | BUG-229: --subdir empty string identity | Guard `--subdir ""` as identity — `!sub.is_empty()` in `resolve_effective_dir()` |
| 6 | 006 | 0 | 8 | 9 | 9 | 0 | ✅ (Completed) | ai | BUG-230: --subdir slash validation | Reject `--subdir` values containing `/` at parse time |
| 7 | 007 | 0 | 8 | 9 | 9 | 0 | ✅ (Completed) | ai | BUG-231: --subdir dry-run creates dir | Skip `create_dir_all` when `--dry-run` is set |
| 8 | 008 | 0 | 7 | 9 | 9 | 0 | ✅ (Completed) | ai | BUG-233: CLR_SUBDIR env-var slash bypass | Extend BUG-230 slash validation to `apply_env_vars()` CLR_SUBDIR path |
| 9 | 010 | 0 | 8 | 8 | 9 | 0 | ✅ (Completed) | ai | [Optional --creds default fallback](completed/010_optional_creds_default.md) | Make `--creds` optional: fall back to `$HOME/.claude/.credentials.json` when absent |
| 10 | 011 | 0 | 9 | 7 | 9 | 0 | ✅ (Completed) | ai | BUG-214-reopen: session-existence guard uses wrong path | Initial fix checked `$HOME/.claude/` (always non-empty); re-fixed using `check_continuation()` for project-specific `$HOME/.claude/projects/{encoded(cwd)}/` |
| 11 | 012 | 0 | 7 | 8 | 9 | 0 | ✅ (Completed) | ai | [Error Classification in CLR](completed/012_error_classification.md) | Add `ErrorKind` enum and `classify_error()` to replace generic silent-failure message with labeled per-type diagnostics |
| 12 | 013 | 0 | 8 | 8 | 9 | 0 | ✅ (Completed) | ai | [Ask Alias Simplification](completed/013_ask_alias_simplification.md) | Remove 7 behavioral overrides from `dispatch_ask()` to make `ask` a pure semantic alias for `run` |
| 13 | 014 | 0 | 7 | 7 | 9 | 0 | ✅ (Completed) | ai | [Output File Parameter](completed/014_output_file_param.md) | Implement `--output-file <PATH>` tee behavior in `run_print_mode` |
| 14 | 015 | 0 | 8 | 6 | 9 | 0 | ✅ (Completed) | ai | [Expect Output Validation Group](completed/015_expect_validation_group.md) | Implement `--expect` / `--expect-strategy` / `--expect-retries` enum output validation in `run_print_mode` |
| 15 | 016 | 0 | 7 | 9 | 9 | 0 | ✅ (Completed) | ai | [BUG-247: stdout swallowed on failure](completed/016_bug247_stdout_swallowed.md) | Forward stdout to stderr when exit_code != 0 in `run_print_mode()` |
| 16 | 017 | 0 | 6 | 9 | 9 | 0 | ✅ (Completed) | ai | [BUG-248: --keep-claudecode no warning](completed/017_bug248_keep_claudecode_warning.md) | Emit warning when `--keep-claudecode` disables CLAUDECODE protection while env var is set |
| 17 | 018 | 0 | 8 | 7 | 9 | 0 | ✅ (Completed) | ai | [Session Concurrency Gate](completed/018_max_sessions_gate.md) | Implement `--max-sessions <N>` concurrency gate with 30s polling and 15-minute timeout |
| 18 | 019 | — | 8 | 7 | 9 | — | ✅ (Completed) | ai | [Retry on Rate Limit](completed/019_retry_on_rate_limit.md) | Auto-retry run/ask on transient rate-limit exit (exit code 2) up to N times |
| 19 | 020 | — | 7 | 6 | 9 | — | ✅ (Completed) | ai | [Subprocess Timeout for run/ask](completed/020_run_ask_timeout.md) | Kill subprocess after --timeout <SECS>; exit 2 on expiry; parity with isolated/refresh |
