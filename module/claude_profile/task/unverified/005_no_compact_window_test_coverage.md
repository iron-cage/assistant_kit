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
contains exactly 12 named `#[test]` functions. The 12 functions cover the dry-run-testable subset
of the 27 spec cases — the remaining 15 require cross-invocation comparison (RC-1/RC-2 dry-run
vs trace equality), live subprocess or real credentials, PATH manipulation to remove `claude`
(EC-6 / EC-9 trace case), journaling behavior (RC-8), or timeout semantics (RC-9); none of these
are achievable with `--dry-run` alone. Note: two spec files share the EC-1..EC-9 numbering scheme
independently; where spec codes appear in this document they are qualified as `acw:EC-N` (from
`03_auto_compact_window.md`) or `ncw:EC-N` (from `075_no_compact_window.md`); `RC-N` always
refers to `06_running_commands.md`. Eight functions invoke `clr` / `clr ask` with `--dry-run`
and assert on stdout: (1) `default_injection_run` — default injection present, (2)
`flag_suppresses_for_run` — `--no-compact-window` flag suppresses, (3)
`env_one_suppresses_for_run` — `CLR_NO_COMPACT_WINDOW=1` env suppresses, (4)
`env_true_suppresses_for_run` — `CLR_NO_COMPACT_WINDOW=true` suppresses, (5)
`env_zero_does_not_suppress` — `CLR_NO_COMPACT_WINDOW=0` does NOT suppress (falsy; spec
`acw:EC-9`), (6) `dry_run_shows_var_when_active` — WYSIWYG check that dry-run output reveals the
active env var (`acw:EC-5` / `ncw:EC-8`); function body is identical to (1) but named distinctly
because the spec requires this scenario to be an explicitly labelled test case — distinct name IS
the verification criterion, identical assertion body is expected and correct, (7)
`dry_run_shows_no_var_when_suppressed` — WYSIWYG check that dry-run output omits the suppressed
var (`acw:EC-5` / `ncw:EC-8`); same relationship to (2) as (6) has to (1), (8)
`flag_suppresses_for_ask` — `ask` alias suppression. Four functions invoke `clr isolated` /
`clr refresh` with `--dry-run` and assert on stderr: (9) `default_injection_isolated` — isolated
default injection present, (10) `flag_suppresses_for_isolated` — isolated `--no-compact-window`
suppresses, (11) `default_injection_refresh` — refresh default injection present, (12)
`flag_suppresses_for_refresh` — refresh `--no-compact-window` suppresses. All 12 tests pass under
`RUSTFLAGS="-D warnings" cargo nextest run --all-features` from `module/claude_runner/`;
`tests/readme.md` gains a new Responsibility Table row for `no_compact_window_test.rs`.

## In Scope

- **`tests/no_compact_window_test.rs`** (new file in `module/claude_runner/`) — implement these 12 test functions:
  - `default_injection_run` — `clr --dry-run "t"`: stdout contains `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` (RC-3/run, acw:EC-1, ncw:EC-1)
  - `flag_suppresses_for_run` — `clr --no-compact-window --dry-run "t"`: stdout does NOT contain the var (RC-4, acw:EC-2, ncw:EC-2)
  - `env_one_suppresses_for_run` — `CLR_NO_COMPACT_WINDOW=1 clr --dry-run "t"`: stdout does NOT contain the var (RC-7, acw:EC-3, ncw:EC-6)
  - `env_true_suppresses_for_run` — `CLR_NO_COMPACT_WINDOW=true clr --dry-run "t"`: stdout does NOT contain the var (acw:EC-4)
  - `env_zero_does_not_suppress` — `CLR_NO_COMPACT_WINDOW=0 clr --dry-run "t"`: stdout CONTAINS the var (acw:EC-9); falsy value leaves injection active
  - `dry_run_shows_var_when_active` — same assertion as (1); distinct name for acw:EC-5/ncw:EC-8 WYSIWYG spec traceability (var shown in dry-run output)
  - `dry_run_shows_no_var_when_suppressed` — same assertion as (2); distinct name for acw:EC-5/ncw:EC-8 WYSIWYG spec traceability (absent var not shown)
  - `flag_suppresses_for_ask` — `clr ask --no-compact-window --dry-run "t"`: stdout does NOT contain the var (ncw:EC-5)
  - `default_injection_isolated` — `clr isolated --creds <tmp_creds> --dry-run`: stderr contains `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` (acw:EC-7, RC-3/isolated)
  - `flag_suppresses_for_isolated` — `clr isolated --creds <tmp_creds> --no-compact-window --dry-run`: stderr does NOT contain the var (RC-5, acw:EC-3/ncw:EC-3 for isolated)
  - `default_injection_refresh` — `clr refresh --creds <tmp_creds> --dry-run`: stderr contains `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` (acw:EC-8, RC-3/refresh)
  - `flag_suppresses_for_refresh` — `clr refresh --creds <tmp_creds> --no-compact-window --dry-run`: stderr does NOT contain the var (RC-6, acw:EC-3/ncw:EC-4 for refresh)

- **Helper pattern for `run`/`ask` tests:** For non-env tests (1, 2, 6, 7, 8), use `run_dry(&[...])` (returns stdout; panics on non-zero exit) or `run_cli(&[..., "--dry-run", ...])` and convert stdout via `stdout_str`. For env-var tests (3, 4, 5), `run_dry` is not available — use `run_cli_with_env(&[..., "--dry-run", ...], &[("CLR_NO_COMPACT_WINDOW", "1")])` and convert stdout via `stdout_str`. Note: `run_cli_with_env` does not strip `CLR_DIR`/`CLR_SESSION_DIR` the way `run_cli` does; this is acceptable for these dry-run tests because our assertions check only for `CLAUDE_CODE_AUTO_COMPACT_WINDOW` presence and are not affected by ambient `CLR_DIR` in typical CI environments.

- **Helper pattern for `isolated`/`refresh` dry-run tests:** `clr isolated/refresh --dry-run` now emits to **stderr** (R5 implementation). Use `run_cli(&["isolated", "--creds", tmp_path, "--dry-run"])` and convert via `stderr_str`. Exit code is 0 (dry-run exits without spawning subprocess). Temporary credentials file: use `make_creds_file("{}")` from `cli_binary_test_helpers` (returns a `tempfile::NamedTempFile`); call `.path().to_str().unwrap()` to get the path string — the `NamedTempFile` must remain live for the duration of the test body to prevent early deletion. Include `make_creds_file` in the import line (see step 5).

- **`tests/readme.md`** — add one new Responsibility Table row: `| no_compact_window_test.rs | Tests for --no-compact-window flag and CLAUDE_CODE_AUTO_COMPACT_WINDOW injection |`

- **Note:** The test helpers doc comment (`cli_binary_test_helpers.rs` line 29-30) states "These commands [`isolated`/`refresh`] lack `--dry-run`; use `--trace` instead" — this was written before R5. `--dry-run` IS now implemented for both (see `cred_parse.rs:149`, `cred_parse.rs:291`). Do NOT update the doc comment (out of scope); just use `--dry-run` directly.

## Out of Scope

- RC-1 / RC-2 (WYSIWYG dry-run vs trace output equality) — requires comparing `--dry-run` and `--trace` stderr output across two separate invocations (cross-invocation equality check); distinct from `acw:EC-5`/`ncw:EC-8` WYSIWYG in scope (single-invocation present/absent check only); deferred to a separate test
- RC-8 (`--journal off` for isolated) — journaling behavior tested in `journal_integration_test.rs`
- RC-9 (`--timeout 0` semantics) — tested in `timeout_test.rs`
- Tests requiring live credentials or subprocess spawn (all 12 functions are dry-run based)
- `--trace` stderr tests (`acw:EC-6`, `ncw:EC-9` trace case) — require `claude` binary absent via PATH override; deferred. Note: `acw:EC-9` (`CLR_NO_COMPACT_WINDOW=0` does NOT suppress) IS in scope as test 5; only the trace-output variant `ncw:EC-9` is excluded here.
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
5. Create `module/claude_runner/tests/no_compact_window_test.rs` with all 12 test functions from In Scope. Structure: top-level doc comment with purpose and spec refs → `use cli_binary_test_helpers::{ run_cli, run_cli_with_env, stdout_str, stderr_str, make_creds_file }` → 12 `#[test]` functions.
6. For the 8 `run`/`ask` functions: see the helper pattern in In Scope. For non-env tests (1, 2, 6, 7, 8), prefer `run_dry(&["t"])` (prepends `--dry-run` automatically; returns stdout directly) or `run_cli(&["--dry-run", "t"])` + `stdout_str`. For env-var tests (3, 4, 5), use `run_cli_with_env(&["--dry-run", "t"], &[("CLR_NO_COMPACT_WINDOW", "1")])` + `stdout_str`. Assert contains / does not contain `"CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000"`.
7. For the 4 `isolated`/`refresh` functions: create temp creds file (`let creds = make_creds_file("{}"); let tmp_path = creds.path().to_str().unwrap();`), call `run_cli(&["isolated", "--creds", tmp_path, "--dry-run"])`, convert via `stderr_str`, assert on stderr; same for refresh.
8. Add `| no_compact_window_test.rs | Tests for --no-compact-window flag and CLAUDE_CODE_AUTO_COMPACT_WINDOW injection |` to `module/claude_runner/tests/readme.md` Responsibility Table.
9. Run `VERB_LAYER=l0 RUSTFLAGS="-D warnings" cargo nextest run --all-features` (host bypass) or `cd module/claude_runner && ./verb/test` (container) to confirm all 12 new tests pass and zero existing tests regress; confirm zero compile warnings.

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
