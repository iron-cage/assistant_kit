# Migrate claude_manager to unilang YAML command definitions

## Goal

`claude_manager` currently registers 14 commands programmatically in `lib.rs` via
inline `reg_cmd()` calls, making it the only Layer 2 crate without a declarative
`COMMANDS_YAML` constant.  This blocks proper aggregation in `claude_tools` (Layer 3)
and causes the `.sessions` name collision with `claude_storage` — the manager variant
silently shadows the storage variant because programmatic registration runs first.

After this task: `claude_manager` exports `pub const COMMANDS_YAML: &str` pointing to a
`unilang.commands.yaml` file containing all 14 command definitions; `register_commands()`
uses the YAML-sourced static registry (matching `claude_storage`'s pattern);
`.sessions` is renamed to `.processes` to resolve the name collision; `claude_tools/build.rs`
aggregates `claude_manager::COMMANDS_YAML` alongside the existing runner and storage YAML.

**Testable:** `w3 .test l::3` passes green across `claude_manager`, `claude_manager_core`,
and `claude_tools`; `clt .processes` lists running Claude Code processes; `clt .sessions`
routes to the storage variant (scope-based session listing).

## In Scope

- Create `module/claude_manager/unilang.commands.yaml` with all 14 command definitions
  (`.version`, `.version.check`, `.version.install`, `.version.guard`, `.settings`,
  `.settings.update`, `.status`, `.account.list`, `.account.status`, `.account.save`,
  `.account.switch`, `.account.delete`, `.processes` (renamed from `.sessions`), `.usage`)
- Rename `.sessions` command to `.processes` in `claude_manager` (semantically accurate:
  it lists OS processes, not storage sessions)
- Export `pub const COMMANDS_YAML: &str` from `claude_manager/src/lib.rs`
- Rewrite `register_commands()` in `claude_manager/src/lib.rs` to use YAML-sourced
  static registry pattern (matching `claude_storage/src/lib.rs` approach)
- Update `claude_manager` integration tests referencing `.sessions` → `.processes`
- Update `claude_tools/build.rs` to aggregate `claude_manager::COMMANDS_YAML`
- Update `claude_tools/src/main.rs` PHF map: remove `.sessions` manager entry,
  add `.processes` entry pointing to the renamed routine
- Update `claude_tools/Cargo.toml` build-dependencies to include `claude_manager`
- Update `claude_manager/spec.md` and related docs with new command name

## Out of Scope

- Changing `claude_manager_core` domain logic (it stays unchanged)
- Migrating `claude_profile` to YAML (separate task 054)
- Changing any command behavior or output format (only the registration mechanism changes)
- Changing the `clm` binary name or adding new commands
- Modifying `claude_storage`'s `.sessions` command (it keeps its name)
- Adding new parameters to existing commands

## Description

The assistant workspace has a design inconsistency: `claude_runner` and `claude_storage`
define their commands in `unilang.commands.yaml` files and export `COMMANDS_YAML` constants,
enabling `assistant/build.rs` to aggregate them at compile time into a PHF static
registry.  But `claude_manager` uses inline programmatic registration via `reg_cmd()` calls
in `lib.rs:86-99`, bypassing the YAML pipeline entirely.

This causes two problems:
1. `claude_tools` cannot aggregate manager commands through the standard YAML pipeline
2. The `.sessions` name is used by both `claude_manager` (list OS processes) and
   `claude_storage` (list storage sessions), causing a silent collision in `clt` where
   the manager variant always wins by registration order

The fix has two parts: (a) migrate all 14 commands to YAML definitions matching the
established pattern, and (b) rename the manager's `.sessions` to `.processes` since it
actually lists running OS processes, not storage sessions.

Related: Task 045 (four-layer architecture), Task 054 (claude_profile migration),
Task 055 (claude_tools full aggregation).

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   YAML command definitions must match the format used by `claude_storage/unilang.commands.yaml`
    and `claude_runner/unilang.commands.yaml`
-   No behavioral changes to any command — only the registration mechanism changes
-   The `.sessions` → `.processes` rename must be reflected in all tests, docs, and specs
-   All Rust code uses 2-space indents and custom codestyle (NOT `cargo fmt`)
-   All new tests in `tests/` directory of the owning crate; no inline `#[cfg(test)]` modules

## Work Procedure

Execute in order. Do not skip or reorder steps.

1.  **Read rulebooks** — `kbase .rulebooks`; note constraints on file layout, code style,
    YAML format.
2.  **Read existing YAML patterns** — study `claude_storage/unilang.commands.yaml` and
    `claude_runner/unilang.commands.yaml` for exact field names, argument attribute format,
    and command definition structure.
3.  **Read `claude_manager/src/lib.rs`** — catalog all 14 `reg_cmd()` calls: command name,
    help text, parameter list, routine function.
4.  **Write Test Matrix** — populate every row before opening any test file.
5.  **Write failing tests** — test that `clm .processes` is accepted, that
    `clt .processes` routes correctly, that `clt .sessions` now routes to storage variant.
6.  **Create `unilang.commands.yaml`** — define all 14 commands with proper argument
    attributes, using `.processes` instead of `.sessions`.
7.  **Export `COMMANDS_YAML`** — add `pub const COMMANDS_YAML: &str` to
    `claude_manager/src/lib.rs` pointing to the YAML file path.
8.  **Rewrite `register_commands()`** — replace programmatic `reg_cmd()` calls with
    YAML-sourced static registry pattern.
9.  **Update `claude_tools/build.rs`** — add `claude_manager::COMMANDS_YAML` as third
    module in the `MultiYamlAggregator` config.
10. **Update `claude_tools/src/main.rs`** — update PHF map: `.processes` entry for the
    renamed routine; ensure `.sessions` maps only to storage.
11. **Update `claude_tools/Cargo.toml`** — add `claude_manager` to `[build-dependencies]`.
12. **Update tests** — fix all integration tests referencing `.sessions` in manager context
    to use `.processes`.
13. **Update docs** — update `claude_manager/spec.md`, `docs/cli/commands.md`, and any
    other references to the renamed command.
14. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings
    across `claude_manager`, `claude_tools`, and affected crates.
15. **Walk Validation Checklist** — every item YES before proceeding.
16. **Update task status** — set ✅ in `task/readme.md`, move file to `task/completed/`.

## Test Matrix

| #   | Input Scenario | Config Under Test | Expected Behavior |
|-----|----------------|-------------------|-------------------|
| T01 | `clm .processes` | renamed command | Exits 0, lists running Claude processes |
| T02 | `clm .sessions` | old name removed | Exits non-zero (unknown command) |
| T03 | `clt .processes` | aggregated via YAML | Exits 0, lists running Claude processes |
| T04 | `clt .sessions` | storage variant now reachable | Exits 0, runs storage sessions routine |
| T05 | `clt .sessions scope::local` | storage variant with scope param | Exits 0, scope param accepted |
| T06 | `clm .version` | YAML-sourced registration | Exits 0, prints version |
| T07 | `clm .status` | YAML-sourced registration | Exits 0, prints status |
| T08 | `clm .help` | YAML command count | Lists all 14 commands |
| T09 | `clt .help` | full aggregation | Output contains `.processes` (not `.sessions` from manager) |
| T10 | `grep COMMANDS_YAML claude_manager/src/lib.rs` | YAML export | Returns match |

## Acceptance Criteria

-   `module/claude_manager/unilang.commands.yaml` exists with 14 command definitions
-   `claude_manager/src/lib.rs` exports `pub const COMMANDS_YAML: &str`
-   `register_commands()` uses YAML-sourced static registry (no `reg_cmd()` calls)
-   `.sessions` command renamed to `.processes` in all manager code, tests, and docs
-   `claude_tools/build.rs` aggregates `claude_manager::COMMANDS_YAML`
-   `clt .sessions` routes to storage variant (scope-based session listing)
-   `clt .processes` routes to manager variant (OS process listing)
-   Every Test Matrix row has a corresponding passing test
-   `w3 .test l::3` passes workspace-wide with zero failures and zero warnings

## Validation Checklist

Desired answer for every question is YES.

**YAML migration**
-   [ ] Does `module/claude_manager/unilang.commands.yaml` exist?
-   [ ] Does it contain exactly 14 command definitions?
-   [ ] Does `claude_manager/src/lib.rs` contain `pub const COMMANDS_YAML`?
-   [ ] Is `register_commands()` free of `reg_cmd()` calls?

**Name collision resolution**
-   [ ] Is `.sessions` absent from manager command definitions?
-   [ ] Does `.processes` exist in manager command definitions?
-   [ ] Does `clt .sessions` route to storage variant?
-   [ ] Does `clt .processes` route to manager variant?

**Aggregation**
-   [ ] Does `claude_tools/build.rs` reference `claude_manager::COMMANDS_YAML`?
-   [ ] Does `claude_tools/Cargo.toml` list `claude_manager` in `[build-dependencies]`?

**Out of Scope confirmation**
-   [ ] Is `claude_manager_core` source unchanged (no domain logic changes)?
-   [ ] Are all command behaviors identical (only registration mechanism changed)?
-   [ ] Is `claude_storage`'s `.sessions` command unchanged?

## Validation Procedure

### Measurements

**M1 — Manager YAML command count**
Before: 0 YAML commands (all programmatic). Expected after: 14 YAML commands.
Fewer than 14 = incomplete migration.

**M2 — `reg_cmd()` call count in lib.rs**
Before: 14 `reg_cmd()` calls. Expected after: 0.
Any positive number = incomplete migration.

**M3 — `.sessions` references in manager code**
Before: N > 0 references. Expected after: 0 (all renamed to `.processes`).
Any positive = incomplete rename.

**M4 — `clt .help` command list**
Before: `.sessions` listed (manager variant). Expected after: `.processes` listed AND
`.sessions` listed (storage variant). Missing either = aggregation failure.

### Anti-faking checks

**AF1 — No commented-out reg_cmd()**
`grep -r "// reg_cmd\|// *reg_cmd" module/claude_manager/src/` → expect zero matches.
Guards against removing registration by commenting out instead of YAML migration.

**AF2 — YAML file is actually parsed**
`cargo build -p claude_tools 2>&1 | grep "claude_manager"` → expect build.rs output
mentioning claude_manager YAML processing. Guards against YAML file existing but not
being aggregated.

**AF3 — `.processes` actually dispatches**
`cargo run --bin clt -- .processes 2>&1` → expect exit 0. Guards against command being
defined in YAML but missing a routine mapping.

**AF4 — `.sessions` collision resolved**
`cargo run --bin clt -- .sessions scope::local 2>&1` → expect exit 0 (storage variant
accepts `scope::`). Guards against `.sessions` still routing to manager variant.

## Outcomes

[To be filled upon completion.]
