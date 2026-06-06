# Output File Parameter

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** ✅ (Completed)
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** module/
- **Validated By:** null
- **Validation Date:** null

## Goal

CLR has no way to capture subprocess output to a file without shell redirection. Pipeline users must
run `clr -p "task" | tee out.txt` or `clr -p "task" > out.txt`. The goal is to add
`--output-file <PATH>` so the runner itself performs tee behavior: capture stdout, write to the
file, and print to stdout. Observable end-state: `clr -p --output-file /tmp/out.txt "Repeat hello"`
creates `/tmp/out.txt` containing the captured output AND prints to stdout; write errors exit 1
with OS error on stderr; dry-run does not create the file; `w3 .test l::3` passes.

## In Scope

- `module/claude_runner/src/cli/mod.rs`:
  - Add `output_file: Option<String>` field to `CliArgs`
  - Parse `--output-file <PATH>` in the hand-rolled CLI parser
  - Apply `CLR_OUTPUT_FILE` env var in `apply_env_vars()` when `--output-file` is absent
  - Implement tee behavior in `run_print_mode()`: after capture, if `output_file` is `Some(path)`,
    write captured text to `path` via `std::fs::write`; on `Err`, emit
    `"Error: failed to write output file '{path}': {err}"` to stderr and `std::process::exit(1)`;
    dry-run skips write and shows path in describe output
  - Add `--output-file <PATH>` to `print_run_help()` and `print_ask_help()`
- New integration tests in `module/claude_runner/tests/` covering T01–T06

## Out of Scope

- `--output-file` behavior in `run_interactive` — parameter is print-mode only
- Append mode — always overwrite
- Content diverging between stdout and file when `--strip-fences` is active (both receive the same
  stripped content per spec; no extra implementation needed)

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note codestyle (2-space indent, no `cargo fmt`),
   privacy invariant, test placement (`tests/` directory only).
2. **Write failing tests** — in `module/claude_runner/tests/`, add tests for T01–T06 (see Test
   Matrix). Use temp dirs (`tempfile` crate) for writable paths; `/nonexistent_dir/out.txt` for
   the error path. Confirm tests fail (field absent in `CliArgs`).
3. **Add `CliArgs` field** — add `output_file: Option<String>` to the `CliArgs` struct.
4. **Add CLI parser support** — parse `--output-file <PATH>` in the hand-rolled parser; store as
   `Some(path)`.
5. **Add env var support** — in `apply_env_vars()`, read `CLR_OUTPUT_FILE` and assign to
   `output_file` when the CLI field is `None`.
6. **Implement tee in `run_print_mode()`** — after capturing subprocess stdout, before returning:
   if `output_file` is `Some(path)` and `dry_run` is false, call `std::fs::write(&path, &captured)`;
   on `Err(e)`, write `"Error: failed to write output file '{path}': {e}"` to stderr,
   `std::process::exit(1)`; then print captured to stdout as usual.
7. **Update help text** — add `--output-file <PATH>` entry to `print_run_help()` and
   `print_ask_help()`.
8. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings before proceeding.
9. **Submit for Validation** — trigger SUBMIT transition. An independent validator executes the
   validation procedure. A NO or deviation triggers REJECT; fix all gaps, resubmit.
10. **Update task state** — on validation pass, update `task/readme.md` index, move file to
    `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `clr -p --output-file <tmp_path> "Repeat: hello"` (live) | file creation + stdout | `<tmp_path>` exists, contains captured output; stdout also contains output |
| T02 | `clr --dry-run "task"` (no `--output-file`) | no file artifact | No file created; stdout goes to terminal normally |
| T03 | `clr -p --output-file /nonexistent_dir/out.txt "task"` | write error path | Exit 1; stderr contains path and OS error |
| T04 | `clr -p --strip-fences --output-file <tmp_path> "task"` (mock/dry-run) | stripped content in both destinations | File content equals stdout content; neither contains fence markers |
| T05 | `clr --dry-run --output-file /tmp/should_not_exist_99999.txt "task"` | dry-run + output-file | Exit 0; file does NOT exist at the given path |
| T06 | `clr --help` | stdout | Contains `--output-file` |

## Acceptance Criteria

- `CliArgs` has `output_file: Option<String>` field
- `--output-file <PATH>` is parsed from CLI args
- `CLR_OUTPUT_FILE` env var is applied when `--output-file` is absent
- `run_print_mode()` writes captured output to the file when `output_file` is set and not dry-run
- Write errors exit 1 with OS error message on stderr — verified by T03
- Dry-run does NOT create the file — verified by T05
- `--help` output contains `--output-file` — verified by T06
- `w3 .test l::3` passes with zero failures and zero warnings

## Validation

### Checklist

**Implementation (positive)**
- [ ] `CliArgs.output_file: Option<String>` field present?
- [ ] `--output-file <PATH>` parsed in CLI?
- [ ] `CLR_OUTPUT_FILE` applied in `apply_env_vars()`?
- [ ] Tee write happens after subprocess capture, not before?
- [ ] Write error path exits 1 with path + OS error on stderr — T03?
- [ ] Dry-run skips write — T05?
- [ ] Help text updated with `--output-file` — T06?
- [ ] T01–T06 all pass?
- [ ] `w3 .test l::3` passes?

**Out of Scope (absence)**
- [ ] `run_interactive` is NOT modified?
- [ ] No append mode — always overwrite?

### Measurements

- New tests: ≥ 6 passing tests (T01–T06)
- `w3 .test l::3`: 0 failures, 0 warnings
- Help text: `grep "output-file" <(clr --help)` → ≥ 1 match

### Invariants

- File is only written in print mode — verify: no `output_file` write logic inside `run_interactive`
  function body

### Anti-faking checks

- `grep -n "output_file" module/claude_runner/src/cli/mod.rs` — must show field declaration,
  parser call, env var application, and tee write site (≥ 4 distinct match lines)
- T05: after running `clr --dry-run --output-file /tmp/should_not_exist_99999.txt "task"`,
  verify `test ! -f /tmp/should_not_exist_99999.txt` passes

## Related Documentation

- `docs/cli/param/029_output_file.md` — full parameter specification
- `tests/docs/cli/param/29_output_file.md` — test case index (EC-1 through EC-6)
- `module/claude_runner/src/cli/mod.rs` — implementation target

## Verification Record

| Dimension | Result | Notes |
|-----------|--------|-------|
| Scope Coherence | PASS | Bounded to `run_print_mode` + parser + env var + help text; meaningful Out of Scope. |
| MOST Goal Quality | PASS | Concrete end-state with specific invocation and verifiable file artifact; `w3 .test l::3` criterion. |
| Value / YAGNI | PASS | Addresses concrete documented gap (no native capture); no speculative modes added. |
| Implementation Readiness | PASS | 10 ordered steps; target file explicit; Test Matrix has 6 rows with correct columns. |

Verified: 2026-06-06. Transition: ❓ → 🎯.

## History

- **[2026-06-06]** `CREATED` — Implement `--output-file <PATH>` tee behavior in `run_print_mode`.
- **[2026-06-06]** `VERIFIED` — MAAV passed (4/4 dimensions). Transitioned ❓→🎯.
- **[2026-06-07]** `COMPLETED` — All tests pass (16/16 crates green, w3 .test l::3). Moved to completed/.
