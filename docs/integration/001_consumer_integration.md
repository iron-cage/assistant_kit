# Integration: Consumer Workspace Integration

### Scope

- **Purpose**: Document the cross-workspace integration protocol between agent_kit and a consumer workspace.
- **Responsibility**: Specify the path dependency setup, co-location requirement, and publishing migration path.
- **In Scope**: Path dep declarations in the consumer workspace's Cargo.toml, required directory co-location, crates exposed to consumers, production publishing path.
- **Out of Scope**: Privacy invariant (→ `invariant/001_privacy_invariant.md`), crate layering (→ `pattern/001_crate_layering.md`).

### System Description

A consumer workspace is a private workspace that depends on one or more agent_kit crates for Claude Code integration. Typical entry points are `claude_profile` (account management and storage paths) and `claude_runner_core` (process execution builder).

### Integration Points

**Consumer workspace `Cargo.toml` declarations:**
```toml
claude_profile     = { path = "../../claude_tools/dev/module/claude_profile",     version = "~1.0.0" }
claude_runner_core = { path = "../../claude_tools/dev/module/claude_runner_core", version = "~1.0.0" }
```

**Required co-location:** Both workspaces must be siblings under the same parent directory for these relative paths to resolve:
```
~/pro/lib/wip_core/
  claude_tools/dev/   ← agent_kit workspace
  consumer/dev/       ← consumer workspace
```

If either workspace is relocated, the path deps in the consumer workspace's `Cargo.toml` must be updated.

**Exposed crates:** Layer 1 core crates (`claude_profile_core`, `claude_runner_core`, `claude_manager_core`, `claude_assets_core`) and Layer 2 library facade (`agent_kit`) are the natural consumer entry points. The Layer 2 CLI crates (`claude_manager`, `claude_storage`, `claude_tools`, etc.) are standalone CLI tools not intended for library consumers.

**Phase migration (kbase consumer):** kbase (in willbe/kbase3 workspace) consumes `claude_assets_core` directly for asset discovery (Phase 1). When `agent_inventory` is implemented (Phase 2), kbase migrates from `claude_assets_core` to `agent_inventory` for multi-agent support — with zero CLI API changes. See [feature/002_agent_inventory.md](../feature/002_agent_inventory.md) § Consumer integration.

### Error Handling

If the co-location requirement is not met, `cargo build` in the consumer workspace fails with "no such file or directory" on the path dep. Fix: ensure both repos are siblings under the same parent, or update the path in the consumer workspace's `Cargo.toml`.

### Compatibility Requirements

**Production publishing path:** When crates are published to crates.io, replace path deps with registry deps:
```toml
claude_profile     = { version = "1.0.0" }
claude_runner_core = { version = "1.0.0" }
```

The path dep is a development convenience. Publishing agent_kit crates to crates.io removes the co-location requirement for production users.

**Version constraint:** Use `~1.0.0` (patch-level flexibility). Patch updates to agent_kit crates should not require the consumer workspace's Cargo.toml to be updated. Minor and major version bumps require coordination.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| invariant | [invariant/001_privacy_invariant.md](../invariant/001_privacy_invariant.md) | One-way dependency rule (agent_kit ← consumer, never reversed) |
| pattern | [pattern/001_crate_layering.md](../pattern/001_crate_layering.md) | Layer 1 and Layer 2 crates that consumers may depend on |
| source | `../../Cargo.toml` | Workspace manifest |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Dependency Flow, Workspace Structure, Cross-Workspace Protocol sections |
