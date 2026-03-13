# TSK-072 — Add typed builder methods for I/O parameters

## Status

✅ (Completed)

## Metadata

- **Value:** 9
- **Easiness:** 7
- **Priority:** 2
- **Safety:** 8
- **Advisability:** 1008

## Goal

Add typed `with_*` builder methods for the six I/O parameters: `print`, `output_format`, `input_format`, `include_partial_messages`, `replay_user_messages`, and `json_schema`.

**MOST criteria:**
- **Motivated:** JSON output parsing and non-interactive execution are the two most common programmatic use cases; all six params are needed to control them correctly. Currently only reachable via raw `with_arg()`.
- **Observable:** Six new typed methods on `ClaudeCommand`; `describe()` output reflects each flag correctly.
- **Scoped:** `src/command.rs` and `src/types.rs` only.
- **Testable:** `ctest3` green; new tests verify each flag appears in the built command exactly when set.

## Description

`--print` forces non-interactive mode (skips TTY check) and is the foundational flag for all programmatic `execute()` usage — without it, claude may behave unexpectedly when no TTY is attached. `--output-format` selects between `text`, `json`, and `stream-json` — critical for machine-readable output. `--input-format` selects `text` vs `stream-json` input. `--include-partial-messages` enables token-by-token streaming with `stream-json`. `--replay-user-messages` re-emits user turns on stdout. `--json-schema` constrains output to a JSON Schema for structured extraction.

All six are CLI-only flags; implementation follows the same `self.args.push()` pattern already used by `with_model()`, `with_verbose()`, etc.

A new `OutputFormat` and `InputFormat` enum should be added to `src/types.rs` to provide type safety instead of raw strings.

## In Scope

- `OutputFormat` enum: `Text`, `Json`, `StreamJson` — add to `src/types.rs`
- `InputFormat` enum: `Text`, `StreamJson` — add to `src/types.rs`
- `with_print(bool)` — adds `-p` when true
- `with_output_format(OutputFormat)` — adds `--output-format <fmt>`
- `with_input_format(InputFormat)` — adds `--input-format <fmt>`
- `with_include_partial_messages(bool)` — adds `--include-partial-messages` when true
- `with_replay_user_messages(bool)` — adds `--replay-user-messages` when true
- `with_json_schema<S: Into<String>>(schema: S)` — adds `--json-schema <schema>`
- Integration tests for all six methods
- Update `docs/claude_params/readme.md` Builder column for all six params

## Out of Scope

- Changes to `execute()` execution logic
- Any env-var-based I/O config

## Requirements

- Follow `code_design.rulebook.md` — TDD red-green-refactor
- Follow `codebase_hygiene.rulebook.md` — no mocking, real command inspection
- Follow `test_organization.rulebook.md` — tests in `tests/` only
- Follow `code_style.rulebook.md` — 2-space indent, custom codestyle

## Acceptance Criteria

- `with_print(true)` adds `-p` to the built command
- `with_output_format(OutputFormat::Json)` adds `--output-format json`
- `with_output_format(OutputFormat::StreamJson)` adds `--output-format stream-json`
- `with_input_format(InputFormat::StreamJson)` adds `--input-format stream-json`
- `with_include_partial_messages(true)` adds `--include-partial-messages`
- `with_replay_user_messages(true)` adds `--replay-user-messages`
- `with_json_schema("{\"type\":\"object\"}")` adds `--json-schema {"type":"object"}`
- Boolean methods with `false` add nothing to the command
- All six params appear in `docs/claude_params/readme.md` Builder column with their method names
- `ctest3` green; zero warnings

## Work Procedure

1. Read applicable rulebooks via `kbase .rulebooks`
2. Add `OutputFormat` and `InputFormat` enums to `src/types.rs`
3. Write failing tests for all six methods (RED)
4. Implement all six `with_*` methods in `src/command.rs`
5. All tests green (GREEN)
6. Refactor if needed; verify `ctest3`
7. Update `docs/claude_params/readme.md` Builder column
8. Update task status → ✅

## Validation Checklist

| # | Question | Desirable | Fail Action |
|---|----------|-----------|-------------|
| 1 | Does `with_print(true)` add `-p` to command args? | YES | Fix method |
| 2 | Does `with_output_format(Json)` add `--output-format json`? | YES | Fix method |
| 3 | Does `with_output_format(StreamJson)` add `--output-format stream-json`? | YES | Fix enum `as_str()` |
| 4 | Does `with_include_partial_messages(false)` add nothing? | YES | Fix boolean guard |
| 5 | Does `with_json_schema(s)` pass schema string verbatim? | YES | Fix method |
| 6 | Does `ctest3` pass with zero warnings? | YES | Fix failures |

## Validation Procedure

### Measurements

| # | Metric | Before | After | Command |
|---|--------|--------|-------|---------|
| M1 | Public `with_*` methods on `ClaudeCommand` | 23 | 29 | `grep -c 'pub fn with_' src/command.rs` |
| M2 | `ctest3` result | passing | passing | `ctest3` |

### Anti-faking Checks

- AF1: Build command with `with_output_format(OutputFormat::Json)` and assert `describe()` contains `--output-format json` — confirms flag string matches claude's expected input
- AF2: Build command with `with_print(false)` and assert `describe()` does NOT contain `-p` — confirms boolean guard is respected
