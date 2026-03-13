# Integration: Willbe Integration

### Scope

- **Purpose**: Document the cross-workspace integration protocol between agent_kit and willbe.
- **Responsibility**: Specify the path dependency setup, co-location requirement, and publishing migration path.
- **In Scope**: Path dep declarations in willbe's Cargo.toml, required directory co-location, crates exposed to willbe, production publishing path.
- **Out of Scope**: Privacy invariant (→ `invariant/001_privacy_invariant.md`), crate layering (→ `pattern/001_crate_layering.md`).

### System Description

willbe is a private workspace that depends on two agent_kit crates for Claude Code integration: `claude_profile` (account management and storage paths) and `claude_runner_core` (process execution builder). These are the only crates willbe consumes directly from agent_kit.

### Integration Points

**willbe's `Cargo.toml` declarations:**
```toml
claude_profile     = { path = "../../claude_tools/dev/module/claude_profile",     version = "~1.0.0" }
claude_runner_core = { path = "../../claude_tools/dev/module/claude_runner_core", version = "~1.0.0" }
```

**Required co-location:** Both workspaces must be siblings under the same parent directory for these relative paths to resolve:
```
~/pro/lib/wip_core/
  claude_tools/dev/   ← agent_kit workspace
  willbe/dev/         ← consumer workspace
```

If either workspace is relocated, the path deps in willbe's Cargo.toml must be updated.

**Exposed crates:** Only `claude_profile` and `claude_runner_core` are consumed by willbe. Other agent_kit crates (`claude_manager`, `claude_storage`, `claude_tools`, etc.) are standalone CLI tools not used by willbe.

### Error Handling

If the co-location requirement is not met, `cargo build` in willbe fails with "no such file or directory" on the path dep. Fix: ensure both repos are siblings under the same parent, or update the path in willbe's `Cargo.toml`.

### Compatibility Requirements

**Production publishing path:** When crates are published to crates.io, replace path deps with registry deps:
```toml
claude_profile     = { version = "1.0.0" }
claude_runner_core = { version = "1.0.0" }
```

The path dep is a development convenience. Publishing agent_kit crates to crates.io removes the co-location requirement for production users.

**Version constraint:** willbe uses `~1.0.0` (patch-level flexibility). Patch updates to agent_kit crates should not require willbe's Cargo.toml to be updated. Minor and major version bumps require coordination.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| invariant | [invariant/001_privacy_invariant.md](../invariant/001_privacy_invariant.md) | One-way dependency rule (agent_kit ← willbe, never reversed) |
| pattern | [pattern/001_crate_layering.md](../pattern/001_crate_layering.md) | Layer 1 and Layer 2 crates that willbe consumes |
| source | `../../Cargo.toml` | Workspace manifest |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Dependency Flow, Workspace Structure, Cross-Workspace Protocol sections |
