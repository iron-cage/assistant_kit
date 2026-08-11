# Feature: Patch CLI

### Scope

- **Purpose**: Document the `.patch.*` command subject of the `clt`/`claude_patch` binary — the CLI surface for installing, uninstalling, pinning, and inspecting Claude Code patch components.
- **Responsibility**: Describe command signatures, argument semantics, and delegation to `claude_patch_core` for every `.patch.*` command.
- **In Scope**: `.patch.list`, `.patch.show`, `.patch.install`, `.patch.uninstall`, `.patch.pin`, `.patch.unpin`, `.patch.status`, `.patch.kinds`; the `.subject.verb` command format; binary alias.
- **Out of Scope**: Patch component domain semantics and state machine (→ `claude_patch_core/docs/feature/001_patch_component_model.md`), the `.param.*` subject (→ `feature/002_param_cli.md`).

### Design

**Status:** Design settled across a multi-turn planning conversation; no implementation exists yet (🔄 Planned — see `feature/readme.md` Overview Table).

**Binary alias:** `clt` ("Claude-Tune"). `clp` was considered first but is already occupied by `claude_profile`'s existing binary.

**Command format:** Commands follow a `.subject.verb` format — `.patch.list`, `.patch.pin`, etc. — chosen because `claude_patch` exposes two distinct subjects (`patch` and `param`, see `feature/002_param_cli.md`) that would otherwise collide on verb names alone (both plausibly want a `.list`/`.show`); the subject prefix disambiguates unambiguously. This differs from sibling CLI crates in this workspace (e.g. `claude_assets`'s flat `.list`/`.install`, `claude_version`'s flat `.config`/`.params`), which use single flat command names since each exposes only one implicit subject.

**Commands:**

| Command | Purpose | Required args | Optional args |
|---------|---------|----------------|-----------------|
| `.patch.list` | Survey all known patch components and their state | — | `kind::` |
| `.patch.show` | Show full detail for one component | `name::` | — |
| `.patch.install` | Install (apply) a patch component | `name::` | — |
| `.patch.uninstall` | Uninstall (reverse) a patch component; rejected if pinned | `name::` | — |
| `.patch.pin` | Pin an installed component, protecting it from uninstall | `name::` | — |
| `.patch.unpin` | Reverse `.patch.pin`; required before `.patch.uninstall` can proceed on a pinned component | `name::` | — |
| `.patch.status` | Print a one-line status summary of every component | — | — |
| `.patch.kinds` | List all known `PatchKind` values and what each delegates to | — | — |

**`.patch.list` behavior:** Without `kind::`, all known components are shown regardless of kind. With `kind::version_lock` (or any other valid kind), only that kind's components are shown. Output includes each component's name, kind, and current state (Available/Installed/Pinned).

**`.patch.show` behavior:** `name::` is required. Prints full detail for one component: kind, current state, and kind-specific detail (e.g. for `version_lock`, the locked version and lock-file path).

**`.patch.install` behavior:** `name::` is required. Delegates to `claude_patch_core`'s `install()`. Idempotent — re-installing an already-installed component is not an error.

**`.patch.uninstall` behavior:** `name::` is required. Delegates to `claude_patch_core`'s `uninstall()`. Rejected with an error if the component is currently Pinned (see `claude_patch_core/docs/invariant/001_pin_blocks_uninstall.md`) — `.patch.unpin` must be run first.

**`.patch.pin` behavior:** `name::` is required. Component must be Installed. Delegates to `claude_patch_core`'s `pin()`.

**`.patch.unpin` behavior:** `name::` is required. Component must be Pinned. Delegates to `claude_patch_core`'s `unpin()`. Added specifically to close a design gap noticed while drafting this CLI: without an explicit unpin command, a pinned component could never be uninstalled at all.

**`.patch.status` behavior:** No arguments. Prints the same per-component information as `.patch.list` with no `kind::` filter, framed as a status overview rather than a browsable listing.

**`.patch.kinds` behavior:** No arguments. Lists every `PatchKind` variant known to `claude_patch_core`, alongside what it delegates to (e.g. `version_lock → claude_version_core`).

### Features

| File | Relationship |
|------|--------------|
| [claude_patch_core/docs/feature/001_patch_component_model.md](../../../claude_patch_core/docs/feature/001_patch_component_model.md) | Domain model and state machine these commands drive |
| [feature/002_param_cli.md](002_param_cli.md) | Sibling `.param.*` subject in the same binary |

### Invariants

| File | Relationship |
|------|--------------|
| [claude_patch_core/docs/invariant/001_pin_blocks_uninstall.md](../../../claude_patch_core/docs/invariant/001_pin_blocks_uninstall.md) | Why `.patch.uninstall` rejects Pinned components |

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` (to create) | `register_commands()`, `run_cli()` |
| `src/commands/patch.rs` (to create) | `.patch.*` command handlers |

### Tests

| File | Relationship |
|------|--------------|
| `tests/patch_cli.rs` (to create) | Command-level integration tests, including pin/uninstall rejection |
