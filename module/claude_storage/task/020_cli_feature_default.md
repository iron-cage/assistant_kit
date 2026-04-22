# Make `cli` feature default so `cargo install` builds binaries

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 🎯 (Available)

## Goal

Make the `cli` feature default in `claude_storage/Cargo.toml` so that `cargo install --path .` builds the `claude_storage` and `clg` binaries without requiring `--features cli` (Motivated: bare `cargo install --path .` currently silently skips both binaries because they require `required-features = ["cli"]` and the default feature set is empty; Observable: `Cargo.toml` `default` entry includes `cli` and `cargo install --path .` succeeds in building both binaries; Scoped: single `Cargo.toml` change, no code changes; Testable: `cargo install --path . && which clg && which claude_storage` succeeds, and `w3 .test level::3` passes).

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/Cargo.toml` — change `default = []` to `default = ["cli"]`

## Out of Scope

- Changes to `enabled` or `full` features
- Changes to any source files
- Changes to `Cargo.toml` at the workspace level

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   The `cli` feature activates `dep:unilang`; adding it to `default` means `unilang` is compiled even without `--features cli` on the library target — verify this doesn't break library-only usage

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note any constraints on Cargo feature design.
2. **Read source** — Read `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/Cargo.toml` to confirm current `[features]` section.
3. **Apply change** — Edit `default = []` → `default = ["cli"]` in `[features]`.
4. **Validate** — Run `w3 .test level::3` from `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage`. All tests must pass with zero warnings.
5. **Verify binary** — Run `cargo build --release` from the same directory; confirm both binaries built under `target/release/`.
6. **Walk Validation Checklist** — check every item; every answer must be YES.
7. **Update task status** — set ✅ in `task/readme.md`, recalculate Advisability=0, re-sort index, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cargo build --release` with no `--features` | `default = ["cli"]` | Both `claude_storage` and `clg` binaries built in `target/release/` |
| T02 | `w3 .test level::3` | Unchanged test suite | All 289+ tests pass, zero warnings |

## Acceptance Criteria

- `Cargo.toml` line `default = []` replaced by `default = ["cli"]`
- `cargo build --release` (no extra flags) produces `target/release/claude_storage` and `target/release/clg`
- `w3 .test level::3` exits 0 with zero failures and zero code warnings

## Validation

### Checklist

Desired answer for every question is YES.

**Feature configuration**
- [ ] C1 — Is `default = ["cli"]` present in `Cargo.toml` `[features]` section?
- [ ] C2 — Does `cli = ["dep:unilang"]` remain unchanged?
- [ ] C3 — Does `full = ["enabled", "cli"]` remain unchanged?

**Binary build**
- [ ] C4 — Does `cargo build --release` (no flags) produce `target/release/claude_storage`?
- [ ] C5 — Does `cargo build --release` (no flags) produce `target/release/clg`?

**Out of Scope confirmation**
- [ ] C6 — Are all `src/**/*.rs` files unchanged (no source edits)?

### Measurements

- [ ] M1 — test count: `w3 .test level::3 2>&1 | grep "test result"` → `test result: ok. 289 passed` (was: same count, but now verifying no regressions)

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

**AF1 — Verify default feature value**
Check: `grep "^default" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/Cargo.toml`
Expected: `default = ["cli"]`. Why: catches a no-op edit that changes a comment instead of the actual value.

## Outcomes

[Added upon task completion]
