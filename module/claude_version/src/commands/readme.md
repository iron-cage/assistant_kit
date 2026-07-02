# commands/

Command handler sub-modules for the `claude_version` crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `mod.rs` | Module root: sub-module declarations, re-exports, and shared private helpers |
| `status.rs` | `.status` — installation state, process count, active account |
| `version.rs` | `.version.*` — show, install, guard, list |
| `history.rs` | `.version.history` — GitHub Releases API fetch, cache, and render |
| `process.rs` | `.processes` and `.processes.kill` — list and terminate Claude processes |
| `settings.rs` | `.settings.*` — read and write `~/.claude/settings.json` |
| `config.rs` | `.config` — 4-layer resolution, show, get, set, and unset |
| `params.rs` | `.params` — inspect the Claude Code parameter catalog; show forms and current values |
| `runtime_files.rs` | `.runtime_files` — enumerate all paths managed by clv at runtime |
