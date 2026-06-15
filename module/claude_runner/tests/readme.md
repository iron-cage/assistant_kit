# tests/

### Scope

**Responsibilities:** Automated integration tests for the `claude_runner` crate — CLI flag parsing, param edge cases, param group interactions, CLR_* env var fallbacks, execution mode routing, dry-run output, verbosity behavior, YAML structure, isolated subcommand, stale-reference guards, and library API.
**In Scope:** All crate functionality exercised via the compiled `clr` binary and the public library API; bug reproducers for tracked issues.
**Out of Scope:** Manual testing (→ `manual/`), test planning documents (→ `docs/`), performance benchmarks.

### Domain Map

| Domain | File | Tests What |
|--------|------|------------|
| ask subcommand (IT-1–IT-8) | `ask_command_test.rs` | `clr ask` pure-alias equivalence, param passthrough, and live-trace path |
| Trace Universality invariant (IT-1–IT-5) | `invariant_trace_universality_test.rs` | `--trace` on all subprocess-executing commands |
| CLI flags (T01–T35) | `cli_args_test.rs` | Core flag parsing and builder translation |
| CLI flags extended (T36–T49, S58–S79, BUG-212, BUG-215) | `cli_args_ext_test.rs` | Positional edge cases, session combos, new flags, bug reproducers |
| Ultrathink (T50–T58) | `ultrathink_args_test.rs` | Message suffix injection and opt-out |
| Effort flags (T59–T70) | `effort_args_test.rs` | Default max injection, overrides, suppression |
| Param edge cases (EC-N) | `param_edge_cases_test.rs` | Per-param positive/negative edge cases: help, model, verbose, no-skip-permissions, interactive, new-session, dir, session-dir, dry-run, verbosity, print, system-prompt, append-system-prompt, no-effort-max, invariant |
| Trace edge cases (S04–S06, S58–S60) | `param_trace_edge_cases_test.rs` | `--trace` parameter edge cases including isolated/refresh credential trace format |
| Extended flag edge cases (S34–S57) | `param_extended_flags_test.rs` | Per-param edge cases for `--no-chrome`, `--no-persist`, `--json-schema`, `--mcp-config` |
| Param groups (CC-N) | `param_group_test.rs` | Combined-flag interaction tests for param groups |
| Dry-run output | `dry_run_test.rs` | Env vars, command line structure, message quoting |
| Execution modes | `execution_mode_test.rs` | Interactive/print routing, exit codes, stderr |
| Verbosity | `verbosity_test.rs` | Output gating levels 0–5, dry-run independence |
| YAML structure | `commands_yaml_test.rs` | `.claude` and `.claude.help` command definitions |
| Library API | `lib_test.rs` | `register_commands()` callability |
| Stale-ref guards + dep constraints (IT-2, IT-3, IT-4) | `stale_ref_guard_test.rs` | No `claude_runner_plugin` or `dream_agent` refs; dep constraint invariants |
| Isolated subcommand | `isolated_test.rs` | `clr isolated`: parsing, errors, exit codes, lim_it live runs, unknown-subcommand detection |
| Isolated/refresh defaults (DT-1–DT-6) | `isolated_defaults_test.rs` | Invariant 005: model constants, effort injection, passthrough override, effort-before-print order |
| Isolated/refresh correctness (CT-1–CT-6) | `isolated_correctness_test.rs` | Correctness gaps S2–S6: no-session-persistence, skip-perms with/without message, no-chrome for refresh, timeout-0 unlimited, CLAUDE.md provisioning |
| Refresh subcommand | `refresh_test.rs` | `clr refresh`: error cases, timeout exit 2, help text (IT-2, IT-4, IT-6, IT-8) |
| Credential defaults (T1–T5) | `creds_default_test.rs` | `--creds` 3-tier resolution: HOME default, CLR_CREDS tier, and refresh path |
| Bug reproducers BUG-239–244 | `bug_reproducers_239_244_test.rs` | Silent-failure: exit code propagation, signal codes, verbosity gate, install hint, mirror sync (BUG-243 moved to claude_runner_core) |
| Bug reproducers BUG-246 | `bug_reproducers_246_test.rs` | WYSIWYG: CLAUDECODE removal visible in trace/dry-run; `--keep-claudecode` suppresses prefix |
| Bug reproducers BUG-037 (T09–T10) | `error_classification_test.rs` | Labeled per-type CLR stderr diagnostics via classify_error() |
| Strip-fences unit (sf01–sf08) | `fence_test.rs` | `strip_fences` correctness: pair stripping, pass-through, edge cases |
| CLR_* env vars (E01–E17) | `env_var_test.rs` | CLR_* env var fallback for vars 1–17, CLI-wins checks |
| CLR_* env vars extended (E18–E57, BUG-233) | `env_var_ext_test.rs` | CLR_* env var fallback for vars 18–57, BUG-233 subdir slash guard |
| Output file capture (T01–T06) | `output_file_test.rs` | `--output-file` tee behavior, write errors, dry-run skip |
| Expect output validation (T01–T17) | `expect_validation_test.rs` | `--expect`/`--expect-strategy`/`--retry-on-validation` validation loop |
| Bug reproducers BUG-247 | `bug_reproducers_247_test.rs` | Stdout-to-stderr forwarding on subprocess failure |
| Bug reproducers BUG-248 | `bug_reproducers_248_test.rs` | `--keep-claudecode` warning when CLAUDECODE present |
| Retry on Transient (EC-1–EC-10, EC-1–EC-7) | `retry_transient_test.rs` | `--retry-on-transient` and `--transient-delay` parse, env var, retry, exhaustion, quota exclusion |
| CLR-layer exit codes (EC-1–EC-3) | `exit_code_contract_test.rs` | timeout→exit 4, expect mismatch→exit 3, gate bypass→exit 0 |
| Retry on Service (EC-1–EC-10, EC-1–EC-7) | `retry_service_test.rs` | `--retry-on-service` and `--service-delay` parse, env var, retry, exhaustion, quota exclusion |
| Retry on Validation (EC-1–EC-10, EC-1–EC-6) | `retry_validation_test.rs` | `--retry-on-validation` and `--validation-delay` parse, env var, retry, exhaustion, old-flag rejected |
| Retry on Account (EC-1–EC-8, EC-1–EC-6) | `retry_account_test.rs` | `--retry-on-account` and `--account-delay` parse, env var, retry, exhaustion |
| Retry on Auth (EC-1–EC-8, EC-1–EC-6) | `retry_auth_test.rs` | `--retry-on-auth` and `--auth-delay` parse, env var, retry, exhaustion |
| Retry on Process (EC-1–EC-8, EC-1–EC-6) | `retry_process_test.rs` | `--retry-on-process` and `--process-delay` parse, env var, retry, exhaustion |
| Retry on Runner (EC-1–EC-6, EC-1–EC-6) | `retry_runner_test.rs` | `--retry-on-runner` and `--runner-delay` parse-only (Runner exits before retry loop) |
| Retry on Unknown (EC-1–EC-10) | `retry_unknown_test.rs` | `--retry-on-unknown` parse, env var, retry, exhaustion, old-flag rejected |
| Retry Override (EC-1–EC-10, EC-1–EC-6) | `retry_override_test.rs` | `--retry-override` and `--retry-override-delay` parse, env var, 3-tier priority (Tier 1) |
| Retry Default (EC-1–EC-8, EC-1–EC-6) | `retry_default_test.rs` | `--retry-default` and `--retry-default-delay` parse, env var, 3-tier fallback (Tier 3) |
| Timeout run/ask (EC-1–EC-8) | `timeout_test.rs` | `--timeout` parse, env var, watchdog kill, fast-exit no-fire |
| User stories (US01–US09) | `user_story_test.rs` | End-to-end user story workflows: core run/ask/model/verbose stories |
| User stories (US10–US18) | `user_story_creds_isolated_test.rs` | End-to-end user story workflows: credential, isolated, and refresh stories |
| User stories (US19–US25) | `user_story_output_test.rs` | End-to-end user story workflows: MCP config, output file, concurrency stories |
| User stories (US26) | `user_story_ps_test.rs` | End-to-end user story workflows: session listing via `clr ps` |
| User stories (US27) | `user_story_kill_test.rs` | End-to-end user story workflows: session termination via `clr kill` |
| `clr ps` subcommand (IT-1–IT-13) | `ps_command_test.rs` | `clr ps` table output, no-sessions message, help listing, typo guard, self-exclusion, path shortening, orphan filtering (BUG-293) |
| `clr kill` subcommand (IT-1–IT-9) | `kill_command_test.rs` | `clr kill` PID validation, SIGTERM delivery, error handling, help text, typo guard |
| Shared helpers | `cli_binary_test_helpers.rs` | Shared test helper: `run_cli()` and `run_cli_with_env()` invocation |

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `ask_command_test.rs` | `clr ask` subcommand: pure-alias equivalence, param passthrough, and live-trace tests IT-1–IT-8. |
| `invariant_trace_universality_test.rs` | Trace Universality invariant (INV-004): `--trace` on all subprocess-executing commands IT-1–IT-5. |
| `cli_args_test.rs` | CLI flag parsing: core flags T01–T35, correct translation to builder calls. |
| `cli_args_ext_test.rs` | CLI flag parsing extended: T36–T49, S58–S79, BUG-212, BUG-215 reproducers. |
| `ultrathink_args_test.rs` | Ultrathink suffix injection and `--no-ultrathink` opt-out (T50–T58). |
| `effort_args_test.rs` | Effort flag defaults, overrides, suppression, and corner cases (T59–T70). |
| `dry_run_test.rs` | Dry-run output: env vars and command line structure. |
| `execution_mode_test.rs` | Execution modes: interactive/print paths, error handling, exit codes. |
| `verbosity_test.rs` | Verbosity flag: output gating levels 0–5, default behavior. |
| `commands_yaml_test.rs` | Verify YAML defines `.claude`, `.claude.help`, and rejects `.please`. |
| `lib_test.rs` | Library API: `register_commands()` callable. |
| `stale_ref_guard_test.rs` | Guard against stale `claude_runner_plugin` and `dream_agent` references; dep constraint invariants IT-2, IT-3, IT-4. |
| `isolated_test.rs` | `clr isolated` subcommand: parsing, error cases, exit codes, lim_it live runs, unknown-subcommand detection. |
| `isolated_defaults_test.rs` | Invariant 005 model and effort defaults: DT-1–DT-6 covering constants, trace injection, passthrough override, arg order. |
| `isolated_correctness_test.rs` | Isolated/refresh correctness gaps CT-1–CT-6: no-session-persistence, skip-perms condition, no-chrome, timeout-0 unlimited, CLAUDE.md provisioning. |
| `refresh_test.rs` | `clr refresh` subcommand: error cases, timeout exit 2, and help text (IT-2, IT-4, IT-6, IT-8). |
| `creds_default_test.rs` | `--creds` 3-tier resolution: HOME default, CLR_CREDS tier, and refresh path (T1–T5). |
| `bug_reproducers_239_244_test.rs` | Bug reproducers BUG-239–244: exit code passthrough, signal codes, verbosity gate, install hint, mirror sync (BUG-243 in claude_runner_core). |
| `bug_reproducers_246_test.rs` | Bug reproducer BUG-246: CLAUDECODE removal visible in trace/dry-run output; `--keep-claudecode` suppresses prefix. |
| `error_classification_test.rs` | Bug reproducer BUG-037: labeled per-type stderr diagnostics from classify_error() (T09–T10). |
| `param_edge_cases_test.rs` | Per-param edge cases (S01–S33): help, core param flags, and invariant checks. |
| `param_trace_edge_cases_test.rs` | Trace edge cases (S04–S06, S58–S60): basic trace behavior and credential trace format. |
| `param_extended_flags_test.rs` | Extended flag edge cases (S34–S57): `--no-chrome`, `--no-persist`, `--json-schema`, `--mcp-config`. |
| `param_group_test.rs` | Param group combined invocations (CC-N): multi-flag interaction tests. |
| `fence_test.rs` | `strip_fences` unit tests: fence-pair stripping, pass-through, edge cases (sf01–sf08). |
| `output_file_test.rs` | `--output-file` tee behavior, write-error exit 1, dry-run file skip (T01–T06). |
| `expect_validation_test.rs` | `--expect`/`--expect-strategy`/`--retry-on-validation` validation loop: match, mismatch, retry, default (T01–T17). |
| `bug_reproducers_247_test.rs` | Bug reproducer BUG-247: stdout forwarded to stderr on subprocess failure. |
| `bug_reproducers_248_test.rs` | Bug reproducer BUG-248: `--keep-claudecode` warning when CLAUDECODE is set in env. |
| `exit_code_contract_test.rs` | CLR-layer exit code contract: timeout→exit 4 (EC-1), expect mismatch→exit 3 (EC-2), gate bypass→exit 0 (EC-3). |
| `retry_transient_test.rs` | `--retry-on-transient` and `--transient-delay` integration: parse, env var, CLI-wins, fake-subprocess retry/exhaustion/quota-excluded, old-flag rejected (EC-1–EC-10 param 34, EC-1–EC-7 param 35). |
| `retry_service_test.rs` | `--retry-on-service` and `--service-delay` integration: parse, env var, CLI-wins, fake-subprocess retry/exhaustion/quota-excluded, old-flag rejected (EC-1–EC-10 param 44, EC-1–EC-7 param 45). |
| `retry_validation_test.rs` | `--retry-on-validation` and `--validation-delay` integration: parse, env var, CLI-wins, fake-subprocess retry/exhaustion, old-flag rejected (EC-1–EC-10 param 48, EC-1–EC-6 param 49). |
| `retry_account_test.rs` | `--retry-on-account` and `--account-delay` integration: parse, env var, CLI-wins, fake-subprocess retry/exhaustion (EC-1–EC-8 param 40, EC-1–EC-6 param 41). |
| `retry_auth_test.rs` | `--retry-on-auth` and `--auth-delay` integration: parse, env var, CLI-wins, fake-subprocess retry/exhaustion (EC-1–EC-8 param 42, EC-1–EC-6 param 43). |
| `retry_process_test.rs` | `--retry-on-process` and `--process-delay` integration: parse, env var, CLI-wins, fake-subprocess retry/exhaustion (EC-1–EC-8 param 46, EC-1–EC-6 param 47). |
| `retry_runner_test.rs` | `--retry-on-runner` and `--runner-delay` parse-only: parse, env var, CLI-wins (EC-1–EC-6 param 50, EC-1–EC-6 param 51). Runner exits before retry loop — no integration tests. |
| `retry_unknown_test.rs` | `--retry-on-unknown` integration: parse, env var, CLI-wins, fake-subprocess retry/exhaustion, old-flag rejected (EC-1–EC-10 param 52, EC-1–EC-7 param 53). |
| `retry_override_test.rs` | `--retry-override` and `--retry-override-delay` integration: parse, env var, CLI-wins, Tier 1 disables/overrides class-specific, cross-class application (EC-1–EC-10 param 54, EC-1–EC-6 param 55). |
| `retry_default_test.rs` | `--retry-default` and `--retry-default-delay` integration: parse, env var, CLI-wins, Tier 3 fallback fires when no override/class-specific (EC-1–EC-8 param 56, EC-1–EC-6 param 57). |
| `timeout_test.rs` | `--timeout` (run/ask) integration: parse, env var, CLI-wins, fake-subprocess watchdog kill and fast-exit no-fire (EC-1–EC-8). |
| `env_var_test.rs` | CLR_* env var fallback: E01–E17, one per CLR_* variable, CLI-wins verification. |
| `env_var_ext_test.rs` | CLR_* env var fallback extended: E18–E57, BUG-233 subdir slash validation. |
| `user_story_test.rs` | User story end-to-end workflows: US01–US09 (core run/ask/model/verbose stories). |
| `user_story_creds_isolated_test.rs` | User story end-to-end workflows: US10–US18 (credential, isolated, refresh stories). |
| `user_story_output_test.rs` | User story end-to-end workflows: US19–US25 (MCP config, output file, concurrency gate). |
| `user_story_ps_test.rs` | User story end-to-end workflows: US26 (session listing via `clr ps`). |
| `user_story_kill_test.rs` | User story end-to-end workflows: US27 (session termination via `clr kill`). |
| `ps_command_test.rs` | `clr ps` subcommand integration tests: table output, no-sessions, help, typo guard, self-exclusion, path shortening, orphan filtering (IT-1–IT-13). |
| `kill_command_test.rs` | `clr kill` subcommand integration tests: PID validation, SIGTERM delivery, error handling, help text, typo guard (IT-1–IT-9). |
| `cli_binary_test_helpers.rs` | Shared test helpers: `run_cli()` and `run_cli_with_env()` binary invocation. |
| `docs/` | Test documentation mirroring `docs/` — test case planning for CLI commands, params, groups. |
| `manual/` | Manual testing plan for live Claude Code invocation. |
