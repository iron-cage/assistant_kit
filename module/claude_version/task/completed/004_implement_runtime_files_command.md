# Task 004: Implement .runtime_files CLI Command

## Execution State

- **State:** ✅ (Closed)
- **Executor:** any

## MOST Goal

- **Motivated:** Creating `docs/runtime_file/` in the normalization session triggers `l0_gov.rulebook.md § CLI : Runtime File Discovery Mandate` — the `.runtime_files` command must be created in the same implementation cycle. Currently `clv .runtime_files` is unregistered and produces an error. The mandate requires a machine-readable discovery command so operators can locate, clean, and monitor runtime files without reading source code.
- **Observable:** `clv .runtime_files` outputs `$HOME/.claude/.transient/version_history_cache.json` followed by a newline to stdout; exits 0 on success; exits 2 when HOME is unset. Output is pipeline-composable: `clv .runtime_files | xargs ls -la` works.
- **Scoped:** New file `src/commands/runtime_files.rs`; registration in `src/commands/mod.rs`; test spec `tests/docs/feature/008_runtime_file_discovery.md`; implementation tests in `tests/cli/`.
- **Testable:** `./verb/test` exits 0; IT-01 through IT-03 and FT-01 through FT-03 appear in the passing output.

## In Scope

- Create `src/commands/runtime_files.rs` — command handler that resolves `$HOME` and prints the version history cache path
- Register `.runtime_files` in the command routing system (`src/commands/mod.rs` and wherever commands are dispatched)
- Create `tests/docs/feature/008_runtime_file_discovery.md` — test spec with FT-01 through FT-03 acceptance criteria
- Create integration tests in `tests/cli/` covering IT-01 through IT-03 and FT-01 through FT-03
- Verify `clv .runtime_files | xargs` pipeline works

## Out of Scope

- Adding new runtime files to the enumeration (only the version history cache is currently documented)
- JSON format output for `.runtime_files` (not in spec — single-path output is sufficient)
- Modifying `src/commands/history.rs` or any cache-writing logic
- Tests for other commands

## Null Hypothesis

The cache path is documented in `runtime_file/001_version_history_cache.md`; a dedicated command duplicates that information without adding value.

**Disproof:** `l0_gov.rulebook.md § CLI : Runtime File Discovery Mandate` explicitly requires a machine-readable command when `docs/runtime_file/` exists. Documentation cannot be used in shell pipelines (`xargs`, `ls`, cleanup scripts). The path also depends on `$HOME` which may vary per user — a command resolves it dynamically while documentation only shows a template.

## Work Procedure

1. Read `docs/feature/008_runtime_file_discovery.md` and `docs/runtime_file/001_version_history_cache.md` to confirm spec
2. Read `src/commands/mod.rs` to understand command registration pattern
3. Create `tests/docs/feature/008_runtime_file_discovery.md` test spec with acceptance criteria:
   - FT-01: default output is one absolute path per line; exits 0
   - FT-02: path is output even when file does not exist on disk
   - FT-03: missing HOME exits 2
4. Create failing tests in `tests/cli/` as `runtime_files_test.rs`:
   - IT-01: `clv .runtime_files` with HOME set → expected path on stdout; exits 0
   - IT-02: `clv .runtime_files` when cache file absent → same path; exits 0
   - IT-03: `clv .runtime_files` with HOME unset → exits 2
   - FT-01 through FT-03 (acceptance criteria as integration tests)
5. Create `src/commands/runtime_files.rs`:
   - Read `$HOME`; return exit 2 if absent
   - Construct absolute path: `$HOME/.claude/.transient/version_history_cache.json`
   - Print path to stdout followed by newline; return exit 0
6. Register `.runtime_files` in command routing (`src/commands/mod.rs` and CLI dispatch)
7. Run `./verb/test` and fix any failures
8. Verify pipeline composability: `clv .runtime_files | xargs ls` works when file exists

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `clv .runtime_files` | HOME set, cache exists | `$HOME/.claude/.transient/version_history_cache.json\n`; exits 0 |
| `clv .runtime_files` | HOME set, `.transient/` absent | same path output; exits 0 (path is theoretical) |
| `clv .runtime_files` | HOME unset | exits 2 |
| `clv .runtime_files \| xargs ls` | HOME set, cache exists | ls output for the file; pipeline exit 0 |

## Validation

- **Pass criterion:** `./verb/test` exits 0; all IT-01–IT-03 and FT-01–FT-03 appear in the PASSED section
- **Fail criterion:** Any test in the matrix appears in FAILED; or `clv .runtime_files` output differs from `$HOME/.claude/.transient/version_history_cache.json`

## Related Documentation

- `docs/feature/008_runtime_file_discovery.md` — feature spec for this task
- `docs/runtime_file/001_version_history_cache.md` — the enumerated runtime file
- `docs/cli/command/root.md` — command reference (command 15)
- `tests/docs/feature/008_runtime_file_discovery.md` — test spec (created in step 3)

**Closes:** null

## History

- **[2026-07-02]** `CREATED` — Implement `.runtime_files` command per l0_gov Runtime File Discovery Mandate, triggered by creation of `docs/runtime_file/` collection.
- **[2026-07-02]** `VERIFIED` — MAAV gate passed: 4/4 independent agents (Scope Coherence, MOST Quality, Value/YAGNI, Implementation Readiness) returned PASS.
- **[2026-07-02]** `CLOSED` — `VERB_LAYER=l0 cargo nextest run --test cli runtime_files` exited 0: 14/14 tests passed (IT-1 through IT-9, FT-1 through FT-5). `src/commands/runtime_files.rs` created; registered in `mod.rs`, `lib.rs`, and help group; all spec tables populated; build and clippy clean.

## Verification Record

**Date:** 2026-07-02
**Method:** MAAV — 4 independent subagents (`general-purpose`), parallel dispatch, Round 1 CONVERGED

| Gate | Name | Prev | Now | Issues |
|------|------|------|-----|--------|
| G1 | Scope Coherence | — | ✅ | — |
| G2 | MOST Goal Quality | — | ✅ | — |
| G3 | Value / YAGNI | — | ✅ | — |
| G4 | Implementation Readiness | — | ✅ | — |
| **Total** | | — | ✅ | — |

All 4 gates passed. State promoted to 🎯 (Verified).
