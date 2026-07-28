# Invariant: Pin Blocks Uninstall

### Scope

- **Purpose**: Guarantee that a pinned patch component can never be uninstalled without an explicit, separate unpin step — preventing accidental removal of a protected patch.
- **Governs**: The `uninstall()` operation in `claude_patch_core` for every `PatchKind`.
- **In Scope**: All calls to `uninstall()` against any component currently in the Pinned state.
- **Out of Scope**: The `pin()`/`unpin()` operations themselves (see `feature/001_patch_component_model.md` for their semantics), CLI-level confirmation prompts (→ `claude_patch/docs/feature/001_patch_cli.md`).

### Invariant Statement

`uninstall()` MUST reject any component whose current state is Pinned. It MUST NOT uninstall after implicitly unpinning, and MUST NOT offer a combined "force" path that skips the explicit `unpin()` step.

### Enforcement Mechanism

`uninstall()` checks the component's state before performing any kind-specific removal behavior. If the state is Pinned, it returns an error immediately — no kind-specific uninstall logic runs. The only way to clear the Pinned state is a separate, explicit call to `unpin()`, which the CLI surfaces as `.patch.unpin` (see `claude_patch/docs/feature/001_patch_cli.md`).

**Rationale:** A patch component is typically pinned because uninstalling it would be disruptive or unsafe at that moment (e.g. a version-lock actively relied on by a running session). Requiring a separate, explicit `unpin` before `uninstall` can proceed forces a deliberate two-step action, making accidental removal via a single automated or scripted `uninstall` call structurally impossible.

### Violation Consequences

If `uninstall()` were to silently unpin-then-uninstall, or offer an unpin-bypassing force flag, a script or automation calling `uninstall` against every listed component would silently remove protected patches with no separate confirmation step — defeating the purpose of pinning entirely. Any implementation found to do so is non-compliant and must be corrected before release.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_patch_component_model.md](../feature/001_patch_component_model.md) | Full component model including pin/unpin semantics |

### Sources

| File | Relationship |
|------|--------------|
| `src/component.rs` (to create) | `uninstall()` — must check Pinned state before proceeding |

### Tests

| File | Relationship |
|------|--------------|
| `tests/component.rs` (to create) | Test asserting `uninstall()` on a Pinned component returns an error |
