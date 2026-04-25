# agent_inventory

Layer 1 domain crate providing agent-agnostic asset discovery across AI coding agents.

Normalises asset enumeration from multiple agents (Claude Code, Codex, Cursor, …) into a
unified flat-table view via the `AgentAdapter` trait. Each agent is represented by an
optional feature-gated adapter — only required agents add compile-time cost.

## Quick Start

```rust
use agent_inventory::inventory::Inventory;

#[cfg(feature = "claude_code")]
{
  use agent_inventory::claude_code::ClaudeCodeAdapter;
  let mut inv = Inventory::new();
  inv.register( Box::new( ClaudeCodeAdapter::new() ) );
  let entries = inv.list_all()?;
  for e in &entries {
    println!( "{:20} {:10} {:10} {}", e.agent, e.kind.as_str(), e.name, e.status );
  }
}
```

## Features

| Feature | Activates | Dependency |
|---------|-----------|------------|
| `claude_code` | Claude Code adapter | `claude_assets_core` |
| `enabled` | All features (default for binaries) | — |
| `full` | Alias for `enabled` | — |

## Crate Layout

| File | Responsibility |
|------|----------------|
| `src/error.rs` | `InventoryError` type |
| `src/entry.rs` | `AssetEntry`, `AssetKind`, `SyncStatus` |
| `src/adapter.rs` | `AgentAdapter` trait |
| `src/inventory.rs` | `Inventory` registry |
| `src/claude_code.rs` | Claude Code adapter (feature `claude_code`) |
