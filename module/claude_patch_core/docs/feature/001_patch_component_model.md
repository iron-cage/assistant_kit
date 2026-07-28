# Feature: Patch Component Model

### Scope

- **Purpose**: Document the domain model for installable/uninstallable Claude Code patch components — the kind taxonomy, component state, and the install/uninstall/pin/unpin operations `claude_patch_core` provides.
- **Responsibility**: Define what a patch component is, what kinds exist, and the state transitions between available, installed, pinned, and uninstalled.
- **In Scope**: `PatchKind` taxonomy (extensible; `version_lock` as the founding kind), component state machine, install/uninstall/pin/unpin operation semantics, delegation to `claude_version_core` for the `version_lock` kind.
- **Out of Scope**: CLI command surface and argument parsing (→ `claude_patch/docs/feature/001_patch_cli.md`), parameter provenance inspection (→ `claude_patch/docs/feature/002_param_cli.md`), the version-lock mechanism's own internals (→ `claude_version/docs/pattern/001_version_lock.md`).

### Design

**Status:** Design settled across a multi-turn planning conversation; no implementation exists yet (🔄 Planned — see `feature/readme.md` Overview Table). Struct/function names below are conceptual pending Cargo scaffolding, not committed signatures.

**Purpose:** Claude Code accumulates behavior-changing "patches" over time — version-lock enforcement, config overrides, and similar interventions — with no single inventory of what is installed, what could be installed, or how to safely remove one. `claude_patch_core` is the Layer 1 domain crate that owns this inventory: a list of patch components, each with a kind, that can be installed, uninstalled, pinned (protected from uninstall), and unpinned.

**`PatchKind` taxonomy:** Open and extensible — new kinds are added as concrete patch behaviors are identified, each free to delegate to whatever existing machinery already implements it rather than duplicating logic. One kind is settled as the founding member:

| Kind | Delegates to | Behavior |
|------|--------------|----------|
| `version_lock` | `claude_version_core`'s existing 8-layer version-lock pattern | Installing pins the Claude Code binary to a specific version and chmod-locks it; uninstalling reverses the lock |
| *(additional kinds)* | TBD | Added as needed — no other kind is committed yet |

**Component state machine:**

| State | Meaning | Reachable via |
|-------|---------|----------------|
| Available | Kind is known but not installed | initial state |
| Installed | Kind's patch is active | `install()` from Available or Uninstalled |
| Pinned | Installed and protected from uninstall | `pin()` from Installed |
| Uninstalled | Patch removed, equivalent to Available | `uninstall()` from Installed (not Pinned) |

**Operations:**

| Operation | Precondition | Effect |
|-----------|--------------|--------|
| install | Component is Available or previously Uninstalled | Applies the kind's patch behavior; component becomes Installed |
| uninstall | Component is Installed and NOT Pinned | Reverses the patch behavior; component becomes Available |
| pin | Component is Installed | Component becomes Pinned; `uninstall` is rejected until `unpin` |
| unpin | Component is Pinned | Component returns to plain Installed; `uninstall` becomes possible again |
| list_all | — | Returns every known component with its current state |
| status | Component exists | Returns the component's current state and kind-specific detail |

**Pin/uninstall interaction:** See `invariant/001_pin_blocks_uninstall.md` for the enforced constraint — `uninstall` on a Pinned component MUST fail outright, never silently unpin-then-uninstall.

**Persistence:** Where component state (installed/pinned) is durably recorded is not yet settled — TBD pending Cargo scaffolding. Candidates include a dedicated state file under the existing `ClaudePaths` topology (`claude_core`) or delegation to each kind's own existing persistence (e.g. `version_lock` already persists via `claude_version_core`'s lock file).

### Features

| File | Relationship |
|------|--------------|
| [claude_patch/docs/feature/001_patch_cli.md](../../../claude_patch/docs/feature/001_patch_cli.md) | CLI surface consuming this domain model |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_pin_blocks_uninstall.md](../invariant/001_pin_blocks_uninstall.md) | Enforced constraint: pin blocks uninstall |

### Patterns

| File | Relationship |
|------|--------------|
| [claude_version/docs/pattern/001_version_lock.md](../../../claude_version/docs/pattern/001_version_lock.md) | Existing 8-layer version-lock mechanism the `version_lock` kind delegates to |

### Sources

| File | Relationship |
|------|--------------|
| `src/component.rs` (to create) | `PatchComponent`, `PatchKind`, state machine |
| `src/lib.rs` (to create) | `install()`, `uninstall()`, `pin()`, `unpin()`, `list_all()`, `status()` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/component.rs` (to create) | State machine and pin/uninstall-rejection tests |
