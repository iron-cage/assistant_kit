# Aggregate claude_profile into claude_tools super-app

## Goal

After tasks 053 and 054, all four Layer 2 crates (`claude_manager`, `claude_profile`,
`claude_runner`, `claude_storage`) export `COMMANDS_YAML` and `register_commands()`.
But `claude_tools` only aggregates `claude_runner` and `claude_storage` — `claude_manager`
(task 053) and `claude_profile` are missing from the `clt` super-app.

After this task: `claude_tools/build.rs` aggregates YAML from all four Layer 2 crates;
`clt .help` lists commands from all four sources; `.account.*` command collisions between
`claude_manager` and `claude_profile` are resolved (profile owns canonical account commands;
manager delegates to profile_core); all 28+ commands are reachable via `clt`.

**Testable:** `w3 .test l::3` passes green workspace-wide; `clt .help` lists commands from
all four source crates; `clt .account.list`, `clt .token.status`, `clt .paths` work;
`clt .processes` and `clt .sessions` both work without collision.

## In Scope

- Add `claude_profile` as dependency and build-dependency in `claude_tools/Cargo.toml`
- Update `claude_tools/build.rs` to aggregate `claude_profile::COMMANDS_YAML` as fourth
  module in `MultiYamlAggregator` config
- Update `claude_tools/src/main.rs`:
  - Add `claude_profile::register_commands()` call in `build_registry()`
  - Add profile routine entries to PHF map (`.account.list`, `.account.status`,
    `.account.save`, `.account.switch`, `.account.delete`, `.token.status`, `.paths`)
- Resolve `.account.*` command collisions: profile crate owns the canonical `.account.*`
  commands; manager's duplicates are removed from manager's YAML (they were convenience
  wrappers calling `claude_profile_core` anyway)
- Update `claude_tools` integration tests to verify all four crate sources
- Update `claude_tools/build.rs` stale comment ("8 storage commands" → actual count)
- Update `spec.md`, `docs/`, and `module/readme.md` with full aggregation status

## Out of Scope

- Changing any command behavior or output format
- Adding new commands not already defined in Layer 2 crates
- Modifying `claude_runner_core`, `claude_storage_core`, or `claude_common`
- Changing binary names (`clt`, `clm`, `clp`, `cls`, `clr`)
- Resolving the `claude_session` stub (git-tracked placeholder, separate concern)

## Description

Task 045 established the four-layer architecture and created `claude_tools` as the Layer 3
super-app, but only partially: it aggregates `claude_runner` (YAML) and `claude_storage`
(YAML + `register_commands()`), with `claude_manager` contributing via programmatic
registration.  `claude_profile` was entirely absent.

Tasks 053 and 054 migrate both `claude_manager` and `claude_profile` to the standard
unilang YAML pattern.  This task completes the picture by wiring all four crates into
`claude_tools`.

The main complexity is the `.account.*` collision: three commands (`.account.list`,
`.account.status`, `.account.switch`) exist in both `claude_manager` and `claude_profile`.
In the current architecture, `claude_manager`'s versions are convenience wrappers that
call `claude_profile_core` — the canonical implementation lives in profile.  Resolution:
remove the three duplicates from `claude_manager`'s YAML (task 053 may have already
prepared for this), letting `claude_profile`'s versions take precedence in `clt`.
`clm` standalone binary keeps its copies for standalone use.

Related: Task 045 (four-layer architecture), Task 053 (claude_manager migration),
Task 054 (claude_profile migration).

**Dependency:** This task MUST NOT start until tasks 053 and 054 are both ✅ Complete.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Tasks 053 and 054 must be completed before starting this task
-   No command behavioral changes — only aggregation wiring
-   All Rust code uses 2-space indents and custom codestyle (NOT `cargo fmt`)
-   All new tests in `tests/` directory of the owning crate

## Work Procedure

Execute in order. Do not skip or reorder steps.

1.  **Verify prerequisites** — confirm tasks 053 and 054 are ✅ Complete.
    Verify `claude_manager::COMMANDS_YAML` and `claude_profile::COMMANDS_YAML` exist.
2.  **Read rulebooks** — `kbase .rulebooks`.
3.  **Read current state** — study `claude_tools/build.rs`, `claude_tools/src/main.rs`,
    and `claude_tools/Cargo.toml` to understand the current aggregation pipeline.
4.  **Write Test Matrix** — populate every row before opening any test file.
5.  **Write failing tests** — test that `clt .account.list`, `clt .token.status`,
    `clt .paths`, `clt .processes` all exit 0; test that `clt .help` lists all sources.
6.  **Update `Cargo.toml`** — add `claude_profile` as dependency and build-dependency;
    add `claude_manager` as build-dependency (if not already present from task 053).
7.  **Update `build.rs`** — add `claude_manager::COMMANDS_YAML` and
    `claude_profile::COMMANDS_YAML` as modules in `MultiYamlAggregator` config.
    Fix stale comment about storage command count.
8.  **Resolve `.account.*` collision** — ensure only one version of each `.account.*`
    command is registered in `clt`. Profile owns canonical versions; remove manager
    duplicates from aggregation (either via YAML exclusion or registration order).
9.  **Update `main.rs`** — add `claude_profile::register_commands()` call; add profile
    routine entries to PHF map; update `claude_manager` entries if needed.
10. **Green state** — `w3 .test l::3` workspace-wide with zero failures, zero warnings.
11. **Update docs** — update `spec.md` command count, `module/readme.md`, `build.rs`
    doc comments, and any stale documentation.
12. **Walk Validation Checklist** — every item YES.
13. **Update task status** — set ✅ in `task/readme.md`, move file to `task/completed/`.

## Test Matrix

| #   | Input Scenario | Config Under Test | Expected Behavior |
|-----|----------------|-------------------|-------------------|
| T01 | `clt .account.list` | profile commands aggregated | Exits 0, lists accounts |
| T02 | `clt .account.status` | profile commands aggregated | Exits 0 or appropriate error |
| T03 | `clt .token.status` | profile commands aggregated | Exits 0 or appropriate error |
| T04 | `clt .paths` | profile commands aggregated | Exits 0, prints paths |
| T05 | `clt .processes` | manager commands aggregated (from task 053) | Exits 0 |
| T06 | `clt .sessions` | storage variant (collision resolved) | Exits 0 |
| T07 | `clt .sessions scope::local` | storage variant accepts scope param | Exits 0 |
| T08 | `clt .help` | full aggregation from 4 crates | Lists 28+ commands |
| T09 | `clt .version` | manager commands via YAML | Exits 0 |
| T10 | `clt .list` | storage commands still work | Exits 0 |
| T11 | `clt .claude` | runner commands still work | Exits 0 (stub message) |

## Acceptance Criteria

-   `claude_tools/Cargo.toml` lists `claude_profile` as dependency and build-dependency
-   `claude_tools/build.rs` aggregates YAML from all four Layer 2 crates
-   `claude_tools/src/main.rs` calls `register_commands()` from all four Layer 2 crates
-   `clt .help` lists commands from all four sources (runner, storage, manager, profile)
-   `clt .account.list`, `clt .token.status`, `clt .paths` work (profile commands reachable)
-   `clt .processes` works (manager's renamed command)
-   `clt .sessions scope::local` works (storage variant, no collision)
-   No `.account.*` command collision (single canonical version per command)
-   Every Test Matrix row has a corresponding passing test
-   `w3 .test l::3` passes workspace-wide with zero failures and zero warnings

## Validation Checklist

Desired answer for every question is YES.

**Profile aggregation**
-   [ ] Does `claude_tools/Cargo.toml` list `claude_profile` in `[dependencies]`?
-   [ ] Does `claude_tools/Cargo.toml` list `claude_profile` in `[build-dependencies]`?
-   [ ] Does `claude_tools/build.rs` reference `claude_profile::COMMANDS_YAML`?
-   [ ] Does `build_registry()` call `claude_profile::register_commands()`?

**Manager aggregation (from task 053)**
-   [ ] Does `claude_tools/build.rs` reference `claude_manager::COMMANDS_YAML`?

**Command reachability**
-   [ ] Does `clt .account.list` exit 0?
-   [ ] Does `clt .token.status` produce output?
-   [ ] Does `clt .paths` exit 0?
-   [ ] Does `clt .processes` exit 0?
-   [ ] Does `clt .sessions scope::local` exit 0?

**Collision resolution**
-   [ ] Is each `.account.*` command registered exactly once in `clt`?
-   [ ] Does `.sessions` route to storage variant (accepts `scope::` param)?
-   [ ] Does `.processes` route to manager variant (lists OS processes)?

**Documentation**
-   [ ] Does `spec.md` reflect the full command count?
-   [ ] Is `build.rs` doc comment updated with all four crate sources?
-   [ ] Is `module/readme.md` accurate?

**Out of Scope confirmation**
-   [ ] Are all command behaviors unchanged?
-   [ ] Are Layer 0 and Layer 1 crates unmodified?
-   [ ] Are binary names unchanged (`clt`, `clm`, `clp`, `cls`, `clr`)?

## Validation Procedure

### Measurements

**M1 — `clt .help` command count**
Before: ~21 commands (manager + runner + storage). Expected after: 28+ commands
(all four crates). Fewer than 28 = incomplete aggregation.

**M2 — `build.rs` module count**
Before: 2 modules (runner, storage). Expected after: 4 modules (runner, storage,
manager, profile). Fewer than 4 = incomplete build-time aggregation.

**M3 — `build_registry()` register_commands() call count**
Before: 3 calls (manager, runner, storage). Expected after: 4 calls.
Fewer than 4 = crate not integrated.

**M4 — PHF map entry count in main.rs**
Before: 11 entries. Expected after: 18+ entries (add 7 profile entries).
Fewer = missing routine mappings.

### Anti-faking checks

**AF1 — Profile commands actually dispatch**
`cargo run --bin clt -- .paths 2>&1` → expect exit 0 with path output.
Guards against commands defined in YAML but missing routine mappings.

**AF2 — No duplicate `.account.*` registrations**
`cargo run --bin clt -- .account.list 2>&1` → expect clean output (no warnings about
duplicate commands). Guards against both manager and profile registering same command.

**AF3 — `.sessions` collision truly resolved**
`cargo run --bin clt -- .sessions scope::local 2>&1` → expect exit 0. If exit non-zero
with "Unknown parameter 'scope'", the manager variant is still shadowing storage.

**AF4 — All four crates in build output**
`cargo build -p claude_tools 2>&1 | grep -c "COMMANDS_YAML\|register_commands"` →
expect references to all four crates in build output.

## Outcomes

[To be filled upon completion.]
