# tests/

### Scope

**Responsibilities:** Automated integration tests for the `claude_runner` crate — CLI flag parsing, param edge cases, param group interactions, CLR_* env var fallbacks, execution mode routing, dry-run output, verbosity behavior, YAML structure, isolated subcommand, stale-reference guards, and library API.
**In Scope:** All crate functionality exercised via the compiled `clr` binary and the public library API; bug reproducers for tracked issues.
**Out of Scope:** Manual testing (→ `manual/`), test planning documents (→ `docs/`), performance benchmarks.

### Domain Map

| Domain | File | Tests What |
|--------|------|------------|
| CLI flags (T01–T49) | `cli_args_test.rs` | Core flag parsing and builder translation |
| Ultrathink (T50–T58) | `ultrathink_args_test.rs` | Message suffix injection and opt-out |
| Effort flags (T59–T70) | `effort_args_test.rs` | Default max injection, overrides, suppression |
| Param edge cases (EC-N) | `param_edge_cases_test.rs` | Per-param positive/negative edge cases for all 24 params |
| Param groups (CC-N) | `param_group_test.rs` | Combined-flag interaction tests for param groups |
| Dry-run output | `dry_run_test.rs` | Env vars, command line structure, message quoting |
| Execution modes | `execution_mode_test.rs` | Interactive/print routing, exit codes, stderr |
| Verbosity | `verbosity_test.rs` | Output gating levels 0–5, dry-run independence |
| YAML structure | `commands_yaml_test.rs` | `.claude` and `.claude.help` command definitions |
| Library API | `lib_test.rs` | `register_commands()` callability |
| Stale-ref guards | `stale_ref_guard_test.rs` | No `claude_runner_plugin` or `dream_agent` references |
| Isolated subcommand | `isolated.rs` | `clr isolated`: parsing, errors, exit codes, lim_it live runs |
| CLR_* env vars (E01–E24) | `env_var_test.rs` | CLR_* env var fallback for all 24 params, CLI-wins checks |
| Shared helpers | `common.rs` | Shared test helper: `run_cli()` and `run_cli_with_env()` invocation |

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `cli_args_test.rs` | CLI flag parsing: core flags T01–T49, correct translation to builder calls. |
| `ultrathink_args_test.rs` | Ultrathink suffix injection and `--no-ultrathink` opt-out (T50–T58). |
| `effort_args_test.rs` | Effort flag defaults, overrides, suppression, and corner cases (T59–T70). |
| `dry_run_test.rs` | Dry-run output: env vars and command line structure. |
| `execution_mode_test.rs` | Execution modes: interactive/print paths, error handling, exit codes. |
| `verbosity_test.rs` | Verbosity flag: output gating levels 0–5, default behavior. |
| `commands_yaml_test.rs` | Verify YAML defines `.claude`, `.claude.help`, and rejects `.please`. |
| `lib_test.rs` | Library API: `register_commands()` callable. |
| `stale_ref_guard_test.rs` | Guard against stale `claude_runner_plugin` and `dream_agent` references. |
| `isolated.rs` | `clr isolated` subcommand: parsing, error cases, exit codes, lim_it live runs. |
| `param_edge_cases_test.rs` | Per-param edge cases (EC-N): positive/negative for all 24 CLI params. |
| `param_group_test.rs` | Param group combined invocations (CC-N): multi-flag interaction tests. |
| `env_var_test.rs` | CLR_* env var fallback: E01–E24, one per param, CLI-wins verification. |
| `common.rs` | Shared test helpers: `run_cli()` and `run_cli_with_env()` binary invocation. |
| `docs/` | Test documentation mirroring `docs/` — test case planning for CLI commands, params, groups. |
| `manual/` | Manual testing plan for live Claude Code invocation. |
