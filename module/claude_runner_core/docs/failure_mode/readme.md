# Failure Mode Collection

### Scope

- **Purpose**: Catalogue known silent failure modes of the `claude` CLI that are not obvious from exit codes or standard output routing.
- **Responsibility**: Index of failure mode doc instances covering exit-code traps, channel misrouting, env-var leaks, and ambiguous non-zero exits.
- **In Scope**: Cases where `claude` fails without an obvious signal — empty output, wrong channel, inherited env, overloaded exit codes.
- **Out of Scope**: Expected failures with clear error text (→ `feature/`), error classification implementation (→ `src/types.rs`).

### Silent Fails Table

| # | Name | Fail Mode | Detection Channel | Sentinel | clr Response |
|---|------|-----------|-------------------|----------|--------------|
| 001 | [Rate-Limit Exit 2](001_rate_limit_exit_2.md) | Rate-limited — zero output, no message anywhere | exit code only | `exit_code == 2` | `classify_error()` returns `RateLimit`; labeled stderr diagnostic ✅ |
| 002 | [Diagnostic on Stdout](002_diagnostic_on_stdout.md) | Error text written to stdout, not stderr | stdout scan | pattern match | `classify_error()` scans both channels ✅; `run_print_mode()` forwards stdout to stderr on non-zero exit ✅ BUG-247 |
| 003 | [CLAUDECODE Env Leak](003_claudecode_env_leak.md) | Inherited env var silently changes child behavior | env inspection | `CLAUDECODE` present | `unset_claudecode` default-on removes var before spawn ✅; warning emitted when `--keep-claudecode` disables protection with CLAUDECODE in env ✅ BUG-248 |
| 004 | [Exit 1 Ambiguity](004_exit_1_ambiguity.md) | Multiple distinct failures all map to exit 1 | stdout + stderr | pattern priority | `classify_error()` with priority ordering emits labeled diagnostic ✅ |
| — | [procedure.md](procedure.md) | Workflow for adding failure mode doc instances | — | — | — |

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Rate-Limit Exit 2](001_rate_limit_exit_2.md) | Exit code 2 with empty output = rate-limited, no message | ✅ |
| 002 | [Diagnostic on Stdout](002_diagnostic_on_stdout.md) | Claude writes error diagnostics to stdout, not stderr | ✅ |
| 003 | [CLAUDECODE Env Leak](003_claudecode_env_leak.md) | CLAUDECODE env var inherited from parent Claude Code session | ✅ |
| 004 | [Exit 1 Ambiguity](004_exit_1_ambiguity.md) | Exit code 1 is overloaded across rate-limit, auth, API, and unknown failures | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating failure mode doc instances | ✅ |
