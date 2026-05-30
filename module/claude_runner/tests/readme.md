# tests/

### Scope

**Responsibilities:** Automated integration tests for the `claude_runner` crate — CLI flag parsing, param edge cases, param group interactions, CLR_* env var fallbacks, execution mode routing, dry-run output, verbosity behavior, YAML structure, isolated subcommand, stale-reference guards, and library API.
**In Scope:** All crate functionality exercised via the compiled `clr` binary and the public library API; bug reproducers for tracked issues.
**Out of Scope:** Manual testing (→ `manual/`), test planning documents (→ `docs/`), performance benchmarks.

### Domain Map

| Domain | File | Tests What |
|--------|------|------------|
| ask subcommand (IT-1–IT-9) | `ask_command_test.rs` | `clr ask` defaults, CLI overrides, and live-trace path |
| Trace Universality invariant (IT-1–IT-5) | `invariant_trace_universality_test.rs` | `--trace` on all subprocess-executing commands |
| CLI flags (T01–T49) | `cli_args_test.rs` | Core flag parsing and builder translation |
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
| Strip-fences unit (sf01–sf08) | `fence_test.rs` | `strip_fences` correctness: pair stripping, pass-through, edge cases |
| CLR_* env vars (E01–E28) | `env_var_test.rs` | CLR_* env var fallback for all 28 params, CLI-wins checks |
| User stories (US-1–US-4 × 21) | `user_story_test.rs` | End-to-end user story workflows for all 21 user stories |
| Shared helpers | `cli_binary_test_helpers.rs` | Shared test helper: `run_cli()` and `run_cli_with_env()` invocation |

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `ask_command_test.rs` | `clr ask` subcommand: Q&A defaults, CLI overrides, and live-trace tests IT-1–IT-9. |
| `invariant_trace_universality_test.rs` | Trace Universality invariant (INV-004): `--trace` on all subprocess-executing commands IT-1–IT-5. |
| `cli_args_test.rs` | CLI flag parsing: core flags T01–T49, correct translation to builder calls. |
| `ultrathink_args_test.rs` | Ultrathink suffix injection and `--no-ultrathink` opt-out (T50–T58). |
| `effort_args_test.rs` | Effort flag defaults, overrides, suppression, and corner cases (T59–T70). |
| `dry_run_test.rs` | Dry-run output: env vars and command line structure. |
| `execution_mode_test.rs` | Execution modes: interactive/print paths, error handling, exit codes. |
| `verbosity_test.rs` | Verbosity flag: output gating levels 0–5, default behavior. |
| `commands_yaml_test.rs` | Verify YAML defines `.claude`, `.claude.help`, and rejects `.please`. |
| `lib_test.rs` | Library API: `register_commands()` callable. |
| `stale_ref_guard_test.rs` | Guard against stale `claude_runner_plugin` and `dream_agent` references; dep constraint invariants IT-2, IT-3, IT-4. |
| `isolated_test.rs` | `clr isolated` subcommand: parsing, error cases, exit codes, lim_it live runs, unknown-subcommand detection. |
| `param_edge_cases_test.rs` | Per-param edge cases (S01–S33): help, core param flags, and invariant checks. |
| `param_trace_edge_cases_test.rs` | Trace edge cases (S04–S06, S58–S60): basic trace behavior and credential trace format. |
| `param_extended_flags_test.rs` | Extended flag edge cases (S34–S57): `--no-chrome`, `--no-persist`, `--json-schema`, `--mcp-config`. |
| `param_group_test.rs` | Param group combined invocations (CC-N): multi-flag interaction tests. |
| `fence_test.rs` | `strip_fences` unit tests: fence-pair stripping, pass-through, edge cases (sf01–sf08). |
| `env_var_test.rs` | CLR_* env var fallback: E01–E28, one per param, CLI-wins verification. |
| `user_story_test.rs` | User story end-to-end workflows: US-1–US-4 for all 21 stories from `tests/docs/cli/user_story/`. |
| `cli_binary_test_helpers.rs` | Shared test helpers: `run_cli()` and `run_cli_with_env()` binary invocation. |
| `docs/` | Test documentation mirroring `docs/` — test case planning for CLI commands, params, groups. |
| `manual/` | Manual testing plan for live Claude Code invocation. |
