# Split `claude_runner_core/src/command.rs` into parameter-group modules

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 📥 (Backlog)

## Goal

Split the 1,828-line `claude_runner_core/src/command.rs` into focused modules organized by parameter group so that no single file exceeds the 1,500-line limit (Motivated: the file exceeds the hard limit and the builder already uses explicit tier comments as natural split points; Observable: `ClaudeCommand` struct definition + execution methods in `command/mod.rs`, with `with_*` builder methods distributed across 3-4 parameter-group files; Scoped: `src/` directory of `claude_runner_core` only — no behavior change, no API surface change; Testable: `w3 .test level::3` passes with zero regressions).

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner_core/src/command.rs` — split into:
  - `src/command/mod.rs` — `ClaudeCommand` struct definition, `new()`, `execute()`, `execute_interactive()`, `execute_dry()`, `build_command()`, and the `claude_version()` standalone function
  - `src/command/params_core.rs` — Tier 1 critical parameters: `with_message`, `with_working_directory`, `with_max_output_tokens`, `with_continue_conversation`, `with_bash_default_timeout_ms`, `with_bash_max_timeout_ms`, `with_auto_continue`, `with_telemetry`
  - `src/command/params_security.rs` — Tier 2 essential parameters: `with_auto_approve_tools`, `with_action_mode`, `with_log_level`, `with_temperature`, `with_skip_permissions`, permission allowlist/denylist methods
  - `src/command/params_extended.rs` — all remaining Tier 3+ parameters: I/O, session management, MCP, model/budget, debug, IDE integration, extension points
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner_core/src/readme.md` — update to list new `command/` subdirectory
- New `src/command/readme.md` — Responsibility Table for the new submodule files

## Out of Scope

- Any behavior change to `ClaudeCommand` or `claude_version()`
- Changes to tests in `tests/`
- Changes to any other crate

## Description

`claude_runner_core/src/command.rs` is a 1,828-line file containing the `ClaudeCommand` builder. The file already uses explicit tier comments (`// Tier 1: Critical parameters`, `// Tier 2: Essential parameters`) as natural split boundaries.

The natural split keeps the struct definition, execution methods (`execute`, `execute_interactive`, `execute_dry`, `build_command`), and the `claude_version()` utility in `command/mod.rs`, while distributing the `with_*` builder methods across parameter-group submodules. Each `with_*` method must use `impl ClaudeCommand` in its respective file; Rust allows `impl` blocks for the same type across multiple files in the same module.

The public API surface (`ClaudeCommand`, `claude_version`) is re-exported from `src/lib.rs` unchanged — no callers are affected.

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Zero behavior change — this is a pure structural refactor; every method moves verbatim
- Each resulting file must be under 700 lines
- `src/command/mod.rs` must contain the struct definition and all `impl ClaudeCommand` execution methods
- New `src/command/readme.md` required (new directory-level registration)
- `src/readme.md` must register `command/` subdirectory

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note module organization and file size constraints.
2. **Read source file** — Read `src/command.rs` fully; identify every tier boundary and `impl ClaudeCommand` block structure.
3. **Write Test Matrix** — populate Test Matrix confirming split boundaries and size targets.
4. **Create `command/` directory** — Create `src/command/mod.rs` with struct def + execution methods.
5. **Create `params_core.rs`** — Extract Tier 1 critical `with_*` methods.
6. **Create `params_security.rs`** — Extract Tier 2 essential `with_*` methods.
7. **Create `params_extended.rs`** — Extract remaining `with_*` methods.
8. **Update `src/lib.rs`** — Change `mod command;` to reference the new directory module (no re-export changes needed if `pub use` chain is intact).
9. **Delete `src/command.rs`** — Remove the original monolith.
10. **Create `src/command/readme.md`** — Responsibility Table.
11. **Update `src/readme.md`** — Register `command/` directory.
12. **Green state** — `w3 .test level::3` must pass with zero failures and zero warnings.
13. **Verify sizes** — `wc -l module/claude_runner_core/src/command/*.rs` — every file under 700 lines.
14. **Update task status** — set ✅ in `task/readme.md`, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `w3 .test level::3` after split | All 13 crates | 0 failures, 0 warnings |
| T02 | `wc -l src/command/*.rs` | All command module files | Every file ≤ 700 lines |
| T03 | `cargo check -p claude_runner_core` | Default features | 0 errors, 0 warnings |
| T04 | Doc tests: `cargo test --doc -p claude_runner_core` | ClaudeCommand doc examples | All pass |

## Acceptance Criteria

- `src/command.rs` is deleted; replaced by `src/command/` directory with 4 files
- Every file in `src/command/` is ≤ 700 lines
- `w3 .test level::3` passes with 0 regressions vs. pre-refactor baseline
- `src/command/readme.md` exists with Responsibility Table
- No test file is modified

## Validation

### Checklist

Desired answer for every question is YES.

**File existence**
- [ ] C1 — Is `src/command.rs` (monolith) absent?
- [ ] C2 — Do all 4 files in `src/command/` exist (`mod.rs`, `params_core.rs`, `params_security.rs`, `params_extended.rs`)?

**File size**
- [ ] C3 — Is every file in `src/command/` ≤ 700 lines?

**Behavior preservation**
- [ ] C4 — Does `w3 .test level::3` pass with 0 failures?
- [ ] C5 — Does `cargo clippy -p claude_runner_core` report 0 warnings?

**Documentation**
- [ ] C6 — Does `src/command/readme.md` exist with a Responsibility Table?
- [ ] C7 — Does `src/readme.md` list the `command/` directory?

**Out of Scope confirmation**
- [ ] C8 — Are no test files modified?
- [ ] C9 — Are no other crates modified?

### Measurements

- [ ] M1 — monolith absent: `ls module/claude_runner_core/src/command.rs 2>&1` → `No such file or directory`
- [ ] M2 — largest split file: `wc -l module/claude_runner_core/src/command/*.rs | sort -rn | head -2` → max ≤ 700 (was: 1828)

### Anti-faking checks

- [ ] AF1 — file count: `ls module/claude_runner_core/src/command/*.rs | wc -l` → 4
- [ ] AF2 — no dead code: `RUSTFLAGS="-D warnings" cargo check -p claude_runner_core --all-features` → 0 errors
- [ ] AF3 — struct in mod.rs: `grep -c "pub struct ClaudeCommand" module/claude_runner_core/src/command/mod.rs` → 1

## Outcomes

[Added upon task completion.]
