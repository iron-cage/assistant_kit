# Implement Test Coverage for `--no-compact-window` and `CLAUDE_CODE_AUTO_COMPACT_WINDOW` Injection

## Execution State

- **Executor Type:** any
- **filed_by:** agent
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ❓ (Unverified)
- **closes:** null
- **dir:** module/claude_runner/tests/
- **validated_by:** null
- **validation_date:** null

## Goal

R5 added `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` as an injected default for all 4 running
commands (`run`, `ask`, `isolated`, `refresh`) and `--no-compact-window` / `CLR_NO_COMPACT_WINDOW`
as the opt-out mechanism. Three spec files were created defining 27 test cases but no Rust test
code was written.

Observable end-state: `tests/no_compact_window_test.rs` in the `claude_runner` crate exists and
contains exactly 12 named `#[test]` functions. Eight functions invoke `clr` / `clr ask` with
`--dry-run` and assert on stdout: (1) default injection present, (2) `--no-compact-window` flag
suppresses, (3) `CLR_NO_COMPACT_WINDOW=1` env suppresses, (4) `CLR_NO_COMPACT_WINDOW=true`
suppresses, (5) `CLR_NO_COMPACT_WINDOW=0` does NOT suppress (falsy), (6) WYSIWYG discovery case —
var visible in dry-run, (7) WYSIWYG suppression case — var absent in dry-run, (8) `ask` alias
suppression. Four functions invoke `clr isolated` / `clr refresh` with `--dry-run` and assert on
stderr: (9) isolated default injection present, (10) isolated `--no-compact-window` suppresses,
(11) refresh default injection present, (12) refresh `--no-compact-window` suppresses. All 12
tests pass under `RUSTFLAGS="-D warnings" cargo nextest run --all-features`; `tests/readme.md`
gains a new Responsibility Table row for `no_compact_window_test.rs`.

## In Scope

- **`tests/no_compact_window_test.rs`** (new file in `module/claude_runner/`) — implement these 12 test functions:
  - `default_injection_run` — `clr --dry-run "t"`: stdout contains `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` (RC-3/run, EC-1)
  - `flag_suppresses_for_run` — `clr --no-compact-window --dry-run "t"`: stdout does NOT contain the var (RC-4, EC-2)
  - `env_one_suppresses_for_run` — `CLR_NO_COMPACT_WINDOW=1 clr --dry-run "t"`: stdout does NOT contain the var (RC-7, EC-3)
  - `env_true_suppresses_for_run` — `CLR_NO_COMPACT_WINDOW=true clr --dry-run "t"`: stdout does NOT contain the var (EC-4)
  - `env_zero_does_not_suppress` — `CLR_NO_COMPACT_WINDOW=0 clr --dry-run "t"`: stdout CONTAINS the var (EC-9); falsy value leaves injection active
  - `dry_run_shows_var_when_active` — same as `default_injection_run` but explicitly named as the EC-5 discovery case (ensures round-trip: var shown in dry-run output)
  - `dry_run_shows_no_var_when_suppressed` — same as `flag_suppresses_for_run` but explicitly the EC-5 suppression discovery case (ensures WYSIWYG: absent var not shown)
  - `flag_suppresses_for_ask` — `clr ask --no-compact-window --dry-run "t"`: stdout does NOT contain the var (EC-5 for ask)
  - `default_injection_isolated` — `clr isolated --creds <tmp_creds> --dry-run`: stderr contains `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` (EC-7, RC-3/isolated)
  - `flag_suppresses_for_isolated` — `clr isolated --creds <tmp_creds> --no-compact-window --dry-run`: stderr does NOT contain the var (RC-5, EC-3 for isolated)
  - `default_injection_refresh` — `clr refresh --creds <tmp_creds> --dry-run`: stderr contains `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` (EC-8, RC-3/refresh)
  - `flag_suppresses_for_refresh` — `clr refresh --creds <tmp_creds> --no-compact-window --dry-run`: stderr does NOT contain the var (RC-6, EC-3 for refresh)

- **Helper pattern for `run`/`ask` tests:** use `run_dry(&[...])` (returns stdout; panics on non-zero exit) or `run_cli(&[..., "--dry-run", ...])` and convert stdout via `stdout_str`. For env-var tests, use `run_cli_with_env(&[..., "--dry-run", ...], &[("CLR_NO_COMPACT_WINDOW", "1")])` and convert stdout via `stdout_str`.

- **Helper pattern for `isolated`/`refresh` dry-run tests:** `clr isolated/refresh --dry-run` now emits to **stderr** (R5 implementation). Use `run_cli(&["isolated", "--creds", tmp_path, "--dry-run"])` and convert via `stderr_str`. Exit code is 0 (dry-run exits without spawning subprocess). Temporary credentials file: write `{}` to a temp path using `std::fs::write(tmp_path, "{}")` within the test body; no real credentials needed.

- **`tests/readme.md`** — add one new Responsibility Table row: `| no_compact_window_test.rs | Tests for --no-compact-window flag and CLAUDE_CODE_AUTO_COMPACT_WINDOW injection |`

- **Note:** The test helpers doc comment (`cli_binary_test_helpers.rs` line 29-30) states "These commands [`isolated`/`refresh`] lack `--dry-run`; use `--trace` instead" — this was written before R5. `--dry-run` IS now implemented for both (see `cred_parse.rs:149`, `cred_parse.rs:291`). Do NOT update the doc comment (out of scope); just use `--dry-run` directly.

## Out of Scope

- RC-1 / RC-2 (WYSIWYG dry-run vs trace output equality) — requires captured stderr comparison across two invocations; more complex assertion; deferred to a separate test
- RC-8 (`--journal off` for isolated) — journaling behavior tested in `journal_integration_test.rs`
- RC-9 (`--timeout 0` semantics) — tested in `timeout_test.rs`
- Tests requiring live credentials or subprocess spawn (all 12 functions are dry-run based)
- `--trace` stderr tests (EC-6, EC-9 trace case) — require `claude` binary absent via PATH override; deferred
- Updating `cli_binary_test_helpers.rs` doc comment about `--dry-run` for isolated/refresh
- Source code changes to `src/` — all implementation is already complete
- Documentation edits — completed directly before this task was created

## Work Procedure

1. Read the 3 spec files to internalize all RC-N / EC-N cases:
   - `module/claude_runner/tests/docs/cli/param_group/06_running_commands.md` (RC-1..RC-9)
   - `module/claude_runner/tests/docs/cli/env_param/03_auto_compact_window.md` (EC-1..EC-9)
   - `module/claude_runner/tests/docs/cli/param/075_no_compact_window.md` (EC-1..EC-9)
2. Read `module/claude_runner/tests/readme.md` to understand the Responsibility Table format.
3. Read `module/claude_runner/tests/dry_run_test.rs` to see the `run_dry` / `run_cli` helper usage pattern and assertion style.
4. Read `module/claude_runner/tests/cli_binary_test_helpers.rs` to understand `run_cli`, `run_cli_with_env`, `stdout_str`, `stderr_str` signatures.
5. Create `module/claude_runner/tests/no_compact_window_test.rs` with all 12 test functions from In Scope. Structure: top-level doc comment with purpose and spec refs → `use cli_binary_test_helpers::{ run_cli, run_cli_with_env, stdout_str, stderr_str }` → 12 `#[test]` functions.
6. For the 8 `run`/`ask` functions: call `run_cli(&["--dry-run", "t"])` or `run_cli_with_env(...)`, convert via `stdout_str`, assert contains / does not contain `"CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000"`.
7. For the 4 `isolated`/`refresh` functions: create temp creds file (`std::fs::write(&tmp_path, "{}")`), call `run_cli(&["isolated", "--creds", &tmp_path, "--dry-run"])`, convert via `stderr_str`, assert on stderr; same for refresh.
8. Add `| no_compact_window_test.rs | Tests for --no-compact-window flag and CLAUDE_CODE_AUTO_COMPACT_WINDOW injection |` to `module/claude_runner/tests/readme.md` Responsibility Table.
9. Run `VERB_LAYER=l0 cargo nextest run --all-features` (host bypass) or `cd module/claude_runner && ./verb/test` (container) to confirm all 12 new tests pass and zero existing tests regress; confirm zero `RUSTFLAGS="-D warnings"` compile warnings.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `clr --dry-run "t"` | no env, no flag | stdout contains `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` |
| `clr --no-compact-window --dry-run "t"` | flag present | stdout does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW` |
| `CLR_NO_COMPACT_WINDOW=1 clr --dry-run "t"` | env fallback `1` | stdout does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW` |
| `CLR_NO_COMPACT_WINDOW=true clr --dry-run "t"` | env fallback `true` | stdout does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW` |
| `CLR_NO_COMPACT_WINDOW=0 clr --dry-run "t"` | falsy env var | stdout CONTAINS `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` |
| `clr --dry-run "t"` (EC-5 discovery case) | default injection, dry-run WYSIWYG | stdout contains `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` (same predicate as row 1; distinct named function) |
| `clr --no-compact-window --dry-run "t"` (EC-5 suppressed) | suppressed, dry-run WYSIWYG | stdout does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW` (same predicate as row 2; distinct named function) |
| `clr ask --no-compact-window --dry-run "t"` | ask alias | stdout does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW` |
| `clr isolated --creds {} --dry-run` | isolated default | stderr contains `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` |
| `clr isolated --creds {} --no-compact-window --dry-run` | isolated opt-out | stderr does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW` |
| `clr refresh --creds {} --dry-run` | refresh default | stderr contains `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` |
| `clr refresh --creds {} --no-compact-window --dry-run` | refresh opt-out | stderr does NOT contain `CLAUDE_CODE_AUTO_COMPACT_WINDOW` |

## Related Documentation

- `module/claude_runner/tests/docs/cli/param_group/06_running_commands.md` — RC-1..RC-9 spec (running commands group interaction tests)
- `module/claude_runner/tests/docs/cli/env_param/03_auto_compact_window.md` — EC-1..EC-9 spec (CLAUDE_CODE_AUTO_COMPACT_WINDOW edge cases)
- `module/claude_runner/tests/docs/cli/param/075_no_compact_window.md` — EC-1..EC-9 spec (--no-compact-window parameter edge cases)
- `module/claude_runner/docs/cli/param/075_no_compact_window.md` — parameter spec
- `module/claude_runner/docs/cli/param_group/06_running_commands.md` — group spec with invariants
- `module/claude_runner/docs/cli/env_param.md` — full env var spec (Section 1 row 63, Section 2 row 12)
- `module/claude_runner/docs/cli/command_defaults.md` — Parameter Matrix (CLAUDE_CODE_AUTO_COMPACT_WINDOW row)

## History

- **[2026-07-02]** `CREATED` — Implement test coverage for `--no-compact-window` / `CLAUDE_CODE_AUTO_COMPACT_WINDOW` injection (R5 feature) across all 4 running commands.
