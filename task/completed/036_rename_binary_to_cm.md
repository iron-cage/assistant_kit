# Rename Binary to `cm` and Fix Empty Command Help

## Goal

`claude_manager` binary is renamed to `cm` and `cm .` shows help instead of erroring.
Users type `cm .version.show` instead of `claude_manager .version.show`. The shorter
name reduces friction in daily use. Motivated by user feedback that the full name is
too long and that `cm .` (bare dot) should show help like `cm` (no args) already does.
Testable via `cargo run -p claude_manager -- .` printing help text with exit 0, and
`which cm` resolving to `~/.cargo/bin/cm` after install.

## In Scope

- `module/claude_manager/Cargo.toml`: `[[bin]] name = "cm"`
- `module/claude_manager/src/main.rs`: dispatch `""` → help; update help text references
- `module/claude_manager/tests/integration/helpers.rs`: `CARGO_BIN_EXE_cm`
- `spec.md`: all CLI examples from `claude_manager` → `cm`
- `docs/cli/*.md`: all CLI references
- `docs/cli/testing/**/*.md`: all CLI references

## Out of Scope

- Crate package name change (stays `claude_manager` in `[package]`)
- Library API changes
- Keeping backward-compatible `claude_manager` symlink

## Description

Two closely related usability fixes:
1. **Binary rename:** `[[bin]] name` in Cargo.toml from `claude_manager` to `cm`.
   All help text, docs, spec, and test helpers reference the old name and must be updated.
2. **Bare dot help:** `cm .` currently parses as command `.` → strips dot → subcmd `""` →
   `dispatch("")` → "unknown command". Fix: match `""` in dispatch to show help.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note constraints affecting file layout or code style.
2. **Write Test Matrix** — populate every row before opening any test file.
3. **Write failing tests** — implement test cases from the Test Matrix.
   Confirm test failures.
4. **Implement** — minimum code to make tests pass.
5. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings.
6. **Refactor if needed** — ensure no function exceeds 50 lines, no duplication.
7. **Walk Validation Checklist** — every answer must be YES.
8. **Update task status** — set status in `task/readme.md`, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cm .` (bare dot) | dispatch empty subcmd | exits 0, stdout contains "COMMANDS:" |
| T02 | `cm` (no args) | empty argv | exits 0, stdout contains "COMMANDS:" |
| T03 | `cm .help` | explicit help | exits 0, stdout contains "cm" not "claude_manager" |
| T04 | `cm .nonexistent` | unknown command | exits 1, stderr contains "unknown command" |

## Acceptance Criteria

- `[[bin]] name = "cm"` in Cargo.toml
- `cm .` exits 0 and prints help (same as `cm` with no args)
- All help text says `cm` not `claude_manager`
- All test helpers use `CARGO_BIN_EXE_cm`
- All docs and spec reference `cm` not `claude_manager`
- `w3 .test l::3` passes clean

## Validation Checklist

Desired answer for every question is YES.

**Binary name**
- [ ] Is `[[bin]] name = "cm"` in Cargo.toml?
- [ ] Does help output contain `cm` and NOT `claude_manager`?
- [ ] Does `CARGO_BIN_EXE_cm` compile in test helpers?

**Bare dot behavior**
- [ ] Does `cm .` exit 0 with help output?
- [ ] Does `dispatch("")` route to help?

**Documentation**
- [ ] Does spec.md use `cm` in all CLI examples?
- [ ] Do docs/cli/*.md use `cm` in all CLI examples?
- [ ] Do docs/cli/testing/**/*.md use `cm` in all CLI examples?

**Out of Scope confirmation**
- [ ] Is `[package] name` still `claude_manager` (unchanged)?
- [ ] Is there no backward-compatible symlink created?

## Validation Procedure

### Measurements

**M1 — Binary name in Cargo.toml**
Before: `name = "claude_manager"`. Expected after: `name = "cm"`.
Deviation means: rename not applied.

**M2 — `claude_manager` references in docs**
Before: N occurrences of `claude_manager` as CLI command in docs/spec.
Expected after: 0 occurrences. Deviation means: incomplete migration.

### Anti-faking checks

**AF1 — Test compilation**
`CARGO_BIN_EXE_cm` must resolve at compile time. If old name used, tests won't compile.

**AF2 — Bare dot test**
Run `cargo run -p claude_manager -- .` and verify exit 0 + "COMMANDS:" in stdout.

## Outcomes

**Completed 2026-03-25.**

- `[[bin]] name = "cm"` in `Cargo.toml`
- All `CARGO_BIN_EXE_claude_manager` references in `tests/` updated to `CARGO_BIN_EXE_cm`
- All CLI docs (`docs/cli/*.md`, `docs/cli/testing/**/*.md`) updated: 203 occurrences replaced
- `spec.md` all CLI examples updated; Architecture table and Known Limitations updated
- TC-01 (`cm .` → help) and TC-02 (`cm` → help) added to `read_commands_test.rs`
- `help.md` created for `.help` / `.` / empty argv test planning
- `ctest3` (nextest + doc-tests + clippy) passes clean across `claude_manager`, `claude_runner_core`, `claude_session`
