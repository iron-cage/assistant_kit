# Migrate claude_profile to reusable unilang integration

## Goal

`claude_profile` (the `clp` binary) has 7 CLI commands wired directly in `main.rs` with
no exportable integration hooks — no `register_commands()`, no `COMMANDS_YAML`, and
`lib.rs` only exports `ClaudePaths`.  This means `claude_tools` (Layer 3) cannot aggregate
any of `claude_profile`'s commands, making `.account.*`, `.token.status`, and `.paths`
completely unreachable from the `clt` super-app.

After this task: `claude_profile` exports `pub const COMMANDS_YAML: &str` pointing to a
`unilang.commands.yaml` file with all 7 command definitions; `lib.rs` exports
`pub fn register_commands(registry: &mut CommandRegistry)` and the 7 routine functions
via a `pub mod cli` module; `claude_tools` can import and aggregate these commands.

**Testable:** `w3 .test l::3` passes green across `claude_profile`, `claude_profile_core`;
`claude_profile::COMMANDS_YAML` is accessible from a build script; all 7 `clp` commands
work identically before and after migration.

## In Scope

- Create `module/claude_profile/unilang.commands.yaml` with 7 command definitions
  (`.account.list`, `.account.status`, `.account.save`, `.account.switch`,
  `.account.delete`, `.token.status`, `.paths`)
- Export `pub const COMMANDS_YAML: &str` from `claude_profile/src/lib.rs`
- Create `pub mod cli` in `claude_profile/src/lib.rs` exposing all 7 routine functions
  with the standard signature `fn(VerifiedCommand, ExecutionContext) -> Result<OutputData, ErrorData>`
- Export `pub fn register_commands(registry: &mut CommandRegistry)` from `claude_profile/src/lib.rs`
- Refactor `claude_profile/src/main.rs` to use `register_commands()` instead of inline wiring
- Add `claude_profile` to `claude_tools/Cargo.toml` as both dependency and build-dependency
- Update `claude_profile/spec.md` to document the new exports

## Out of Scope

- Adding `claude_profile` commands to `claude_tools` aggregation (separate task 055)
- Resolving `.account.*` command name collisions with `claude_manager` (task 055)
- Changing any command behavior, parameters, or output format
- Modifying `claude_profile_core` domain logic
- Adding new commands to `claude_profile`
- Changing the `clp` binary name

## Description

The dream workspace follows a pattern where Layer 2 CLI crates expose their commands
for aggregation by `claude_tools` (Layer 3).  Both `claude_runner` and `claude_storage`
follow this pattern with `COMMANDS_YAML` constants and (for storage) a `register_commands()`
function.  `claude_manager` is being migrated in task 053.

`claude_profile` is the remaining holdout: its 7 commands are wired directly in `main.rs`
using inline `reg_cmd()` / programmatic registration, and `lib.rs` exports only
`ClaudePaths` (which was moved to `claude_common` in task 045 and is now a re-export).

The migration requires:
1. Creating YAML definitions for all 7 commands
2. Extracting routine functions into a `pub mod cli` module accessible from `lib.rs`
3. Adding the standard `register_commands()` and `COMMANDS_YAML` exports
4. Refactoring `main.rs` to use the new shared registration

Three of the 7 commands (`.account.list`, `.account.status`, `.account.switch`) also exist
in `claude_manager` — the collision resolution is deferred to task 055.

Related: Task 045 (four-layer architecture), Task 053 (claude_manager migration),
Task 055 (claude_tools full aggregation).

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   YAML command definitions must match the format used by `claude_storage/unilang.commands.yaml`
-   Routine function signatures must match: `fn(VerifiedCommand, ExecutionContext) -> Result<OutputData, ErrorData>`
-   No behavioral changes to any command — only the registration and export mechanism changes
-   All Rust code uses 2-space indents and custom codestyle (NOT `cargo fmt`)
-   All new tests in `tests/` directory of the owning crate

## Work Procedure

Execute in order. Do not skip or reorder steps.

1.  **Read rulebooks** — `kbase .rulebooks`; note constraints.
2.  **Read existing patterns** — study `claude_storage/src/lib.rs` for the exact
    `register_commands()` + `COMMANDS_YAML` + `pub mod cli` pattern.
3.  **Read `claude_profile/src/main.rs`** — catalog all 7 commands: name, help text,
    parameters, routine function locations.
4.  **Read `claude_profile/src/lib.rs`** — understand current exports and re-exports.
5.  **Write Test Matrix** — populate every row before opening any test file.
6.  **Write failing tests** — test that `claude_profile::COMMANDS_YAML` exists,
    `claude_profile::cli::account_list_routine` is accessible, `register_commands()`
    populates a registry with 7 commands.
7.  **Create `unilang.commands.yaml`** — define all 7 commands with proper argument
    attributes.
8.  **Create `pub mod cli`** — extract routine functions from `main.rs` into a module
    accessible via `lib.rs`.
9.  **Export `COMMANDS_YAML` and `register_commands()`** — add both to `lib.rs`.
10. **Refactor `main.rs`** — use `register_commands()` to build the registry instead
    of inline wiring.
11. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings
    across `claude_profile` and `claude_profile_core`.
12. **Walk Validation Checklist** — every item YES.
13. **Update task status** — set ✅ in `task/readme.md`, move file to `task/completed/`.

## Test Matrix

| #   | Input Scenario | Config Under Test | Expected Behavior |
|-----|----------------|-------------------|-------------------|
| T01 | `clp .account.list` | command still works after refactor | Exits 0, lists accounts |
| T02 | `clp .token.status` | command still works after refactor | Exits 0 or appropriate error |
| T03 | `clp .paths` | command still works after refactor | Exits 0, prints paths |
| T04 | `clp .help` | YAML-sourced command count | Lists all 7 commands |
| T05 | `use claude_profile::COMMANDS_YAML` in build script | YAML export | Compiles, constant is valid path |
| T06 | `claude_profile::cli::account_list_routine` | pub mod cli | Function accessible from lib |
| T07 | `claude_profile::register_commands()` | register function | Populates registry with 7 commands |

## Acceptance Criteria

-   `module/claude_profile/unilang.commands.yaml` exists with 7 command definitions
-   `claude_profile/src/lib.rs` exports `pub const COMMANDS_YAML: &str`
-   `claude_profile/src/lib.rs` exports `pub fn register_commands()`
-   `claude_profile/src/lib.rs` exports `pub mod cli` with all 7 routine functions
-   `clp` binary works identically before and after (all 7 commands functional)
-   `claude_profile/src/main.rs` uses `register_commands()` (no inline command wiring)
-   Every Test Matrix row has a corresponding passing test
-   `w3 .test l::3` passes with zero failures and zero warnings

## Validation Checklist

Desired answer for every question is YES.

**YAML definitions**
-   [ ] Does `module/claude_profile/unilang.commands.yaml` exist?
-   [ ] Does it contain exactly 7 command definitions?
-   [ ] Does `claude_profile/src/lib.rs` contain `pub const COMMANDS_YAML`?

**Public API exports**
-   [ ] Does `claude_profile/src/lib.rs` export `pub mod cli`?
-   [ ] Does `claude_profile/src/lib.rs` export `pub fn register_commands`?
-   [ ] Are all 7 routine functions accessible via `claude_profile::cli::*`?

**Binary equivalence**
-   [ ] Does `clp .account.list` exit 0?
-   [ ] Does `clp .paths` exit 0?
-   [ ] Does `clp .help` list 7 commands?

**Out of Scope confirmation**
-   [ ] Is `claude_profile_core` source unchanged?
-   [ ] Are all command behaviors identical (no output format changes)?
-   [ ] Is `claude_tools` NOT yet aggregating profile commands?

## Validation Procedure

### Measurements

**M1 — Profile YAML command count**
Before: 0 YAML commands. Expected after: 7 YAML commands.
Fewer than 7 = incomplete migration.

**M2 — Inline command wiring in main.rs**
Before: 7 inline command registrations. Expected after: 0 (uses `register_commands()`).
Any positive = incomplete migration.

**M3 — Public exports from lib.rs**
Before: only `ClaudePaths` re-export. Expected after: `ClaudePaths` + `COMMANDS_YAML` +
`register_commands()` + `cli` module. Missing any = incomplete export.

### Anti-faking checks

**AF1 — YAML file is parseable**
`python3 -c "import yaml; yaml.safe_load(open('module/claude_profile/unilang.commands.yaml'))"` or
equivalent serde_yaml parse → expect success. Guards against syntactically invalid YAML.

**AF2 — Routine functions have correct signatures**
`grep "pub fn.*_routine.*VerifiedCommand.*ExecutionContext" module/claude_profile/src/` →
expect 7 matches. Guards against exported functions with wrong signatures.

**AF3 — main.rs actually calls register_commands()**
`grep "register_commands" module/claude_profile/src/main.rs` → expect match.
Guards against main.rs still using inline wiring while lib.rs has an unused export.

## Outcomes

[To be filled upon completion.]
