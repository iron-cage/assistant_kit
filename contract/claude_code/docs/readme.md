# docs

### Responsibility Table

| Path | Responsibility |
|------|----------------|
| `behavior/` | Observed external behaviors of the `claude` binary — 32 instances (B1–B31 + B16h) |
| `storage/` | `~/.claude/` storage architecture — 3 instances (projects dir, support dirs, root files) |
| `filesystem/` | Runtime filesystem paths accessed by claude_version — 4 instances |
| `jsonl/` | Session JSONL entry format — 10 instances (common fields, entry types, content blocks, usage, threading, sidechain) |
| `settings/` | Settings file structure and protocols — 3 instances (global, project, version lock) |
| `formats/` | Ancillary file formats — 7 instances (history, credentials, debug, shell-snapshots, todos, commands, JSON response) |
| `taxonomy/` | Four-level concept hierarchy (Project/Conversation/Session/Entry) — 3 instances |
| `endpoint/` | Anthropic HTTP endpoint wire contracts — 10 instances (URL, auth, schema, errors) |
| `params/` | CLI parameter specifications — one file per runtime parameter (120 instances) |
| `tool/` | Built-in tools available in Claude Code sessions — 40 instances (14 categories) |
| `subcommand/` | CLI subcommands — 9 instances (agents, auth, auto-mode, doctor, install, mcp, plugin, setup-token, update) |
| `001_entity.md` | Cross-entity index: all collection types with instance counts |
| `fault/` | Fault collection — all known error, silent failure, and quirk conditions of the `claude` binary |
