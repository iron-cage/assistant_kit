# Create `claude_assets` crate pair — multi-artifact installer CLI (`cla`)

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 🎯 (Available)

## Goal

Create two new dedicated crates — `claude_assets_core` (Layer 1 domain logic) and `claude_assets` (Layer 2 CLI binary `cla`) — that let the user list, install, and uninstall Claude Code artifacts (rules, skills, commands, agents, plugins, hooks) from a central dev-environment source (`$PRO_CLAUDE`) into project-local `.claude/<kind>/` via symlinks, and integrate the binary into `claude_tools` (Motivated: eliminate manual symlink management when sharing Claude Code customizations across projects — single source of truth in `$PRO_CLAUDE`, one command to deploy; Observable: `cla .list kind::rule` shows available + installed rules, `cla .install kind::rule name::rust` creates `.claude/rules/rust.md → $PRO_CLAUDE/rules/rust.md` symlink, `clt .list kind::rule` works via aggregation; Scoped: two new crates + workspace registration + claude_tools integration, zero changes to existing crate internals; Testable: symlink presence verified by `readlink`; `w3 .test level::3` passes for full workspace with no regressions).

## In Scope

**New crate — `module/claude_assets_core/`** (Layer 1, no CLI deps):
- `Cargo.toml` — lib-only; deps: `claude_common` (optional/enabled), `error_tools` (optional/enabled)
- `src/lib.rs` — crate root with `cfg_attr` feature gate `enabled`
- `src/artifact.rs` — `ArtifactKind` enum (Rule, Skill, Command, Agent, Plugin, Hook); `ArtifactLayout` enum (File, Directory); inherent methods: `source_subdir()`, `target_subdir()`, `layout()`, `file_extension()` (for File layout kinds)
- `src/paths.rs` — `AssetPaths` struct: reads `$PRO_CLAUDE` env var (fallback: `$PRO/genai/claude/` when `$PRO` is set); resolves target from current working dir as `.claude/<kind>/`; returns typed error when env var unset
- `src/registry.rs` — `InstallStatus` enum (Installed, Available); `list_available(kind)` → names; `list_installed(kind)` → names; `list_all(kind)` → `Vec<(name, InstallStatus)>`
- `src/install.rs` — `install(paths, kind, name)` creates symlink in target subdir (creates subdir if missing); `uninstall(paths, kind, name)` removes symlink only (refuses to remove non-symlinks); both operations are idempotent with appropriate messaging
- `readme.md` — responsibility table entry
- `tests/install.rs` — real-fs tests via `tempfile`; no mocking

**New crate — `module/claude_assets/`** (Layer 2, binary `cla`):
- `Cargo.toml` — `[lib]` + `[[bin]] name="claude_assets"` + `[[bin]] name="cla"`; deps: `claude_assets_core`, `unilang`, `error_tools`
- `src/lib.rs` — `pub const COMMANDS_YAML`, `pub fn register_commands()`, `pub fn run_cli()` (same hybrid pattern as `claude_manager/src/lib.rs`)
- `src/commands.rs` — `list_routine`, `install_routine`, `uninstall_routine`, `kinds_routine`
- `src/main.rs` — calls `run_cli()`
- `src/bin/cla.rs` — thin alias binary calling `run_cli()`
- `unilang.commands.yaml` — `.list` (args: `kind::`, `installed::`), `.install` (args: `kind::`, `name::`), `.uninstall` (args: `kind::`, `name::`), `.kinds` (no args)
- `readme.md` — responsibility table entry
- `tests/cli.rs` — integration tests via `assert_cmd`

**Workspace and registration changes:**
- `/home/user1/pro/lib/wip_core/claude_tools/dev/Cargo.toml` — add `"module/claude_assets_core"` and `"module/claude_assets"` to `workspace.members`; add `[workspace.dependencies.claude_assets_core]` and `[workspace.dependencies.claude_assets]` path-dep blocks
- `module/readme.md` — add 2 rows for new crates in responsibility table

**`agent_kit` extension (2 files):**
- `module/agent_kit/Cargo.toml` — add `assets = [ "dep:claude_assets_core" ]` to `[features]`; update `full` and `enabled` to include `"assets"`; add `claude_assets_core` as `optional = true` dep; add `[workspace.dependencies.claude_assets_core]` reference
- `module/agent_kit/src/lib.rs` — add `#[cfg(feature = "assets")] pub mod assets { pub use claude_assets_core::*; }` block; update crate-level doc feature table to include the `assets` row

**`claude_tools` integration (3 files):**
- `module/claude_tools/Cargo.toml` — add `claude_assets` to `[features] enabled`, `[dependencies]` (optional), and `[build-dependencies]` (non-optional for COMMANDS_YAML watch)
- `module/claude_tools/build.rs` — add `println!("cargo:rerun-if-changed={}", claude_assets::COMMANDS_YAML);` (watch only; no YAML aggregation — programmatic registration)
- `module/claude_tools/src/main.rs` — add `claude_assets::register_commands(&mut registry);` call inside `build_registry()` (after existing registrations)

## Out of Scope

- Creating artifact content in `$PRO_CLAUDE` source directories — code only, no config provisioning
- MCP server lifecycle management
- Authentication or credential handling
- Any modification to internal logic of existing crates (`claude_common`, `claude_manager`, `claude_storage`, etc.)
- Documentation under workspace-level `docs/` beyond crate-local `readme.md` files

## Description

Claude Code customizations — rules (`.claude/rules/*.md`), skills (`.claude/skills/<name>/`), custom commands (`.claude/commands/*.md`), agent definitions (`.claude/agents/*.md`), plugins (`.claude/plugins/<name>/`), hooks (`.claude/hooks/`) — must currently be manually symlinked from a developer's central source into each project's `.claude/` directory. There is no tooling for this.

`claude_assets` fills that gap. `cla` is a CLI that treats `$PRO_CLAUDE` as a package registry:

```
$PRO_CLAUDE/              ← source (git-managed dev env)
├── rules/rust.md
├── skills/tsk/SKILL.md
├── commands/commit.md
├── agents/planner.md
├── plugins/my_plugin/
└── hooks/pre_commit.yaml
```

`cla .install kind::rule name::rust` installs by creating a symlink:

```
.claude/rules/rust.md → $PRO_CLAUDE/rules/rust.md
```

Symlinks preserve the single source of truth: edits in `$PRO_CLAUDE` propagate to every project instantly. `cla .uninstall` removes only symlinks it created (refuses to remove regular files, guarding against accidental data loss).

The utility exposes 4 commands: `.list` (survey what's available and installed), `.install` (symlink from source), `.uninstall` (remove symlink), `.kinds` (show supported artifact types with their source/target path mappings).

Integration into `claude_tools` (clt) follows the **programmatic registration pattern** already established by `claude_manager` and `claude_profile`: `claude_assets::register_commands(&mut registry)` at runtime. The YAML file is metadata-only; `build.rs` only adds a `rerun-if-changed` watch, no compile-time aggregation.

**Artifact kinds and layouts:**

| Kind | Source subdir | Target subdir | Layout | Note |
|------|---------------|---------------|--------|------|
| Rule | `rules/` | `.claude/rules/` | File (`.md`) | Unconditional or globs: frontmatter |
| Skill | `skills/` | `.claude/skills/` | Directory | `<name>/SKILL.md` inside |
| Command | `commands/` | `.claude/commands/` | File (`.md`) | Slash commands |
| Agent | `agents/` | `.claude/agents/` | File (`.md`) | Subagent definitions |
| Plugin | `plugins/` | `.claude/plugins/` | Directory | Plugin dir structure |
| Hook | `hooks/` | `.claude/hooks/` | File | YAML/JSON hook configs |

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- `claude_assets_core` must have **zero CLI dependencies** — no `unilang`, no `clap`, no `structopt`; only `claude_common` and `error_tools`
- Install mechanism is **symlink only** — never copy files; symlinks preserve single source of truth
- `uninstall` must refuse to remove non-symlinks with a clear error (prevent accidental data loss)
- `$PRO_CLAUDE` env var resolution: if set, use directly; if unset but `$PRO` is set, try `$PRO/genai/claude/`; if neither, return typed `AssetPathsError::EnvVarNotSet`
- Graceful degradation: if source kind subdir doesn't exist, `list_available` returns empty vec (not error); if target `.claude/kind/` doesn't exist, `install` creates it
- All tests use real filesystems via `tempfile` — zero mocking
- Error handling via `error_tools` exclusively (no `anyhow`, no `thiserror` mixing)
- No `cargo fmt` — follow custom codestyle from applicable rulebooks (2-space indent, spaced brackets, etc.)
- `register_commands()` in `claude_assets/src/lib.rs` follows the identical pattern to `claude_manager::register_commands()` for consistency

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_design.rulebook.md`, `files_structure.rulebook.md`, `organizational_principles.rulebook.md`, `code_style.rulebook.md` for formatting invariants.
2. **Read `claude_manager` as the template** — Read `module/claude_manager/Cargo.toml`, `module/claude_manager/src/lib.rs`, `module/claude_manager/src/commands.rs` to internalize the exact pattern for dual-binary crates with `register_commands()`.
3. **Read `claude_tools` integration points** — Read `module/claude_tools/Cargo.toml`, `module/claude_tools/build.rs`, `module/claude_tools/src/main.rs` to understand exactly which lines to add for integration.
4. **Read `claude_common/src/paths.rs`** — Confirm `ClaudePaths` is scoped to `~/.claude/` only; verify artifact paths (`$PRO_CLAUDE`) do NOT belong there and must live in `claude_assets_core/src/paths.rs`.
5. **Write failing tests first (TDD)**:
   a. `module/claude_assets_core/tests/install.rs` — unit tests for `install()`, `uninstall()`, `list_available()`, `list_installed()` using `tempfile::TempDir` for both source and target. Tests must compile-fail until implementation exists.
   b. `module/claude_assets/tests/cli.rs` — integration tests via `assert_cmd` for `cla .list`, `cla .install`, `cla .uninstall`, `cla .kinds`. Tests must fail until binary exists.
6. **Create `claude_assets_core` skeleton** — `Cargo.toml` with `[lib]` only, no `[[bin]]`; feature gate `enabled` matching existing crate pattern; path deps for `claude_common` and `error_tools`.
7. **Implement `src/artifact.rs`** — `ArtifactKind` enum (6 variants) with `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`; inherent methods for `source_subdir()`, `target_subdir()`, `layout()`, `file_extension()`; `ArtifactLayout` enum (File, Directory); doc comments on each variant explaining Claude Code's usage.
8. **Implement `src/paths.rs`** — `AssetPaths` struct holding resolved source root path and cwd; constructor resolves `$PRO_CLAUDE` with `$PRO/genai/claude/` fallback; `source_dir(kind)` → `PathBuf`; `target_dir(kind)` → `PathBuf` (relative to cwd `.claude/<kind>/`); typed `AssetPathsError` via `error_tools`.
9. **Implement `src/registry.rs`** — `InstallStatus { Installed, Available }`; `list_available(paths, kind)` scans source dir; `list_installed(paths, kind)` scans target dir for symlinks; `list_all(paths, kind)` merges both with status.
10. **Implement `src/install.rs`** — `install(paths, kind, name)`: resolves source path, verifies it exists, creates target dir if missing, creates symlink (removes stale symlink if target already exists and is a symlink); `uninstall(paths, kind, name)`: verifies target is a symlink before removing (returns typed error for non-symlinks); both return `Result<InstallReport, AssetError>`.
11. **Create `claude_assets` CLI crate** — `Cargo.toml` (two `[[bin]]` targets: `claude_assets` and `cla`, both `path` pointing to distinct files); `src/lib.rs` with `COMMANDS_YAML` const, `register_commands()`, `run_cli()` following `claude_manager` pattern exactly; `src/commands.rs` with four routines; `src/main.rs` and `src/bin/cla.rs` each calling `run_cli()`; `unilang.commands.yaml` defining `.list`, `.install`, `.uninstall`, `.kinds`.
12. **Create `readme.md` for each new crate** — single-sentence responsibility in responsibility table format.
13. **Register in workspace** — add both new crates to root `Cargo.toml` `workspace.members` (alphabetical) and add `[workspace.dependencies.*]` blocks.
14. **Update `module/readme.md`** — add 2 rows for new crates.
15. **Extend `agent_kit`** — Read `module/agent_kit/Cargo.toml` and `module/agent_kit/src/lib.rs` to understand the existing feature-gated pattern; add `assets` feature following the same structure; verify `--no-default-features` still compiles and `--features assets` exposes `agent_kit::assets::ArtifactKind`.
16. **Integrate with `claude_tools`** — (a) add `claude_assets` to `Cargo.toml` features, deps, build-deps; (b) add `rerun-if-changed` line to `build.rs`; (c) add `claude_assets::register_commands(&mut registry);` to `build_registry()` in `main.rs`.
17. **Verify tests pass** — run core unit tests first: `cargo nextest run -p claude_assets_core`; then CLI integration: `cargo nextest run -p claude_assets`; then `cargo check -p agent_kit --features assets`; then full workspace: `w3 .test level::3`.
18. **Walk Validation Checklist** — every item must answer YES.
19. **Update task status** — mark complete in `task/readme.md`.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `cla .kinds` | PRO_CLAUDE set to temp dir | Prints all 6 kinds with source and target paths |
| `cla .list` | PRO_CLAUDE unset, PRO unset | Error: env var not set with actionable remediation message |
| `cla .list` | PRO_CLAUDE set, all kind dirs empty/absent | Empty output per kind; exit 0 |
| `cla .list kind::rule` | 3 .md files in source/rules/, none installed | Shows 3 entries with "○ available" marker |
| `cla .list kind::rule` | 2 installed (symlinks in .claude/rules/), 1 not | Shows ● for installed, ○ for available |
| `cla .install kind::rule name::rust` | rust.md in source, .claude/rules/ absent | Creates .claude/rules/, symlinks rust.md; exit 0 |
| `cla .install kind::rule name::rust` | Already installed (symlink exists) | Idempotent: re-links or reports "already installed"; exit 0 |
| `cla .install kind::rule name::nonexistent` | Source file absent | Err: "rule 'nonexistent' not found in $PRO_CLAUDE/rules/" |
| `cla .install kind::skill name::tsk` | tsk/ dir in source/skills/ | Creates .claude/skills/tsk → source symlink (Directory layout) |
| `cla .uninstall kind::rule name::rust` | Symlink present in .claude/rules/ | Removes symlink; exit 0 |
| `cla .uninstall kind::rule name::rust` | Not installed | Reports "not installed"; exit 0 (not error) |
| `cla .uninstall kind::rule name::rust` | Target is a regular file (not symlink) | Err: refuses to remove non-symlink |
| `clt .list kind::rule` | claude_tools built with claude_assets integration | Delegates correctly, identical output to `cla .list kind::rule` |
| `cargo check -p agent_kit --features assets` | agent_kit extended with assets feature | Compiles; `agent_kit::assets::ArtifactKind` resolves |
| `cargo check -p agent_kit --no-default-features` | agent_kit no-default-features still clean | Compiles without pulling in claude_assets_core |
| `cargo check -p claude_assets_core` | No CLI deps in Cargo.toml | Compiles with zero warnings; no unilang symbols |
| `w3 .test level::3` | Full workspace | All pass; zero regressions in previously-passing crates |

## Acceptance Criteria

- Both `cla` and `claude_assets` binaries are built; both respond to `.kinds` without error when `$PRO_CLAUDE` is set
- `.list`, `.install`, `.uninstall`, `.kinds` all produce correct output as per Test Matrix
- `install()` creates symlinks — confirmed by `std::fs::read_link()` in tests (not file copies)
- `uninstall()` on a non-symlink returns a typed error without removing the file
- `$PRO_CLAUDE` unset produces a clear, actionable error message (not a panic)
- `claude_assets_core/Cargo.toml` has zero `unilang` dependency (confirmed by `grep`)
- All 6 `ArtifactKind` variants are implemented with correct `source_subdir()` / `target_subdir()` / `layout()`
- `clt .list kind::rule` works after `claude_tools` integration
- `RUSTFLAGS="-D warnings" cargo check -p claude_assets_core` passes with zero warnings
- `RUSTFLAGS="-D warnings" cargo check -p claude_assets` passes with zero warnings
- `w3 .test level::3` passes for the full workspace with no regressions in other crates
- `module/readme.md` has rows for both `claude_assets_core` and `claude_assets`
- Root `Cargo.toml` includes both new crates in `workspace.members`

## Validation

### Checklist

Desired answer for every question is YES.

**claude_assets_core — structure**
- [ ] Does `Cargo.toml` contain `[lib]` only (no `[[bin]]`)?
- [ ] Is `unilang` absent from `[dependencies]` and `[features]`?
- [ ] Are all 6 `ArtifactKind` variants implemented in `src/artifact.rs`?
- [ ] Does `AssetPaths` read `$PRO_CLAUDE` env var (with `$PRO/genai/claude/` fallback)?
- [ ] Does `AssetPaths` return a typed error (not panic) when both env vars are unset?

**claude_assets_core — behavior**
- [ ] Does `install()` create a symlink (not a file copy)?
- [ ] Does `install()` create the target subdir if it doesn't exist?
- [ ] Is `install()` idempotent (re-linking or clear message if already installed)?
- [ ] Does `uninstall()` refuse to remove non-symlinks with a typed error?
- [ ] Does `list_available()` return empty vec (not error) when source kind dir is absent?

**claude_assets — CLI**
- [ ] Are both `cla` and `claude_assets` binaries declared in `Cargo.toml`?
- [ ] Does `src/lib.rs` expose `COMMANDS_YAML`, `register_commands()`, and `run_cli()`?
- [ ] Does `unilang.commands.yaml` define `.list`, `.install`, `.uninstall`, `.kinds`?
- [ ] Does `.list kind::rule` filter output to rules only?
- [ ] Does `.install` require both `kind::` and `name::` arguments?
- [ ] Does `.uninstall` require both `kind::` and `name::` arguments?
- [ ] Does `.kinds` require no arguments?

**Workspace registration**
- [ ] Are both new crates listed in root `Cargo.toml` `workspace.members` (alphabetical)?
- [ ] Do both `[workspace.dependencies.*]` blocks exist with correct path and `default-features = false`?
- [ ] Does `module/readme.md` have rows for both new crates?

**agent_kit extension**
- [ ] Does `module/agent_kit/Cargo.toml` have `assets = [ "dep:claude_assets_core" ]` in `[features]`?
- [ ] Is `claude_assets_core` listed as `optional = true` in `module/agent_kit/[dependencies]`?
- [ ] Do `full` and `enabled` features in `agent_kit` now include `"assets"`?
- [ ] Does `module/agent_kit/src/lib.rs` have the `#[cfg(feature = "assets")] pub mod assets` block?
- [ ] Does `cargo check -p agent_kit --features assets` pass?
- [ ] Does `cargo check -p agent_kit --no-default-features` still pass (feature isolation preserved)?
- [ ] Does the `agent_kit` crate-level doc feature table include the `assets` row?

**claude_tools integration**
- [ ] Is `claude_assets` in the `enabled` feature list of `claude_tools/Cargo.toml`?
- [ ] Is `claude_assets` in `[build-dependencies]` of `claude_tools/Cargo.toml`?
- [ ] Does `build.rs` have a `rerun-if-changed` line for `claude_assets::COMMANDS_YAML`?
- [ ] Does `main.rs` `build_registry()` call `claude_assets::register_commands(&mut registry)`?

**Testing and verification**
- [ ] Are tests using real `tempfile::TempDir` (no mocking)?
- [ ] Does `cargo nextest run -p claude_assets_core` pass?
- [ ] Does `cargo nextest run -p claude_assets` pass?
- [ ] Does `w3 .test level::3` pass for full workspace?

### Measurements

**M1 — Kinds command output**
Command: `PRO_CLAUDE=/tmp/test cla .kinds 2>&1 | wc -l`
Before: binary does not exist. Expected: ≥6 output lines (one per artifact kind). Deviation: <6 lines or error exit.

**M2 — Install creates symlink, not copy**
Command: (in tests) `std::fs::read_link(".claude/rules/rust.md").is_ok()`
Before: file does not exist. Expected: `true` (path is a symlink). Deviation: `false` (regular file — install used copy).

**M3 — claude_assets_core has zero CLI deps**
Command: `grep -c 'unilang' module/claude_assets_core/Cargo.toml`
Before: file does not exist. Expected: 0. Deviation: ≥1 (CLI dep leaked into core layer).

**M4 — All 6 ArtifactKind variants present**
Command: `grep -cE 'Rule|Skill|Command|Agent|Plugin|Hook' module/claude_assets_core/src/artifact.rs`
Before: file does not exist. Expected: ≥6. Deviation: <6 (incomplete enum).

**M5 — claude_tools integration wired**
Command: `grep -c 'claude_assets::register_commands' module/claude_tools/src/main.rs`
Before: 0. Expected: 1. Deviation: 0 (integration not wired; `clt .list kind::rule` would fail).

**M6 — Full workspace verification**
Command: `w3 .test level::3 2>&1 | tail -10`
Before: baseline passing. Expected: all tests pass, no regressions. Deviation: any failure in previously-passing crate.

### Anti-faking checks

**AF1 — Symlink confirmed, not copy**
Check: `stat -c '%F' .claude/rules/<name>.md` (or Rust equivalent in tests)
Expected: `"symbolic link"`. Why: install must use `std::os::unix::fs::symlink()`, not `std::fs::copy()`; a copied file would appear correct in `list_installed()` but would defeat the single-source-of-truth invariant.

**AF2 — $PRO_CLAUDE env var is the lookup mechanism**
Check: `grep -c 'PRO_CLAUDE' module/claude_assets_core/src/paths.rs`
Expected: ≥1. Why: confirms runtime env resolution, not a hardcoded path that would break on other machines.

**AF3 — All 6 kinds wired in enum**
Check: `grep -cE '^\s+Rule|^\s+Skill|^\s+Command|^\s+Agent|^\s+Plugin|^\s+Hook' module/claude_assets_core/src/artifact.rs`
Expected: ≥6. Why: confirms all variants are enum members, not just mentioned in comments.

**AF4 — claude_tools integration is real**
Check: `grep -c 'claude_assets::register_commands' module/claude_tools/src/main.rs`
Expected: 1. Why: without this line, `clt .list kind::rule` silently fails (command not found), but `cla .list kind::rule` still works — integration appears to work but isn't actually wired.

**AF5 — No mocking in tests**
Check: `grep -rE 'Mock|Fake|Stub' module/claude_assets_core/tests/ module/claude_assets/tests/ 2>/dev/null | wc -l`
Expected: 0. Why: mocked FS tests can pass while real FS behavior is broken (permission errors, symlink resolution, cross-device links).

**AF6 — Uninstall refuses non-symlinks**
Check: In `tests/install.rs` create a regular file at the target path, call `uninstall()`, assert `Err` variant and file still present.
Expected: error returned, file unchanged. Why: silent deletion of regular files would be data-loss; this guard is load-bearing for safe operation in shared environments.

**AF7 — agent_kit extension is real, not cosmetic**
Check: `cargo check -p agent_kit --features assets 2>&1 | grep -c 'error'`
Expected: 0. Why: it's possible to add the feature flag in Cargo.toml without adding the `pub mod assets` block in `src/lib.rs` — the feature compiles but `agent_kit::assets::ArtifactKind` silently doesn't exist; this check confirms the re-export is actually wired.

## Outcomes

[Empty — populated upon task completion]
