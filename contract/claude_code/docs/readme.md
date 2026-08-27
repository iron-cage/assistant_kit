# docs

### Responsibility Table

| Path | Responsibility |
|------|----------------|
| `behavior/` | Observed external behaviors of the `claude` binary — 38 instances (B1–B37 + B16h) |
| `storage/` | `~/.claude/` storage architecture — 3 instances (projects dir, support dirs, root files) |
| `filesystem/` | Runtime filesystem paths accessed by claude_version — 4 instances |
| `jsonl/` | Session JSONL entry format — 10 instances (common fields, entry types, content blocks, usage, threading, sidechain) |
| `envelope/` | Session-log top-level line kinds — 19 instances, one per `type` discriminator, with payload fields, structural class, and version lifecycle |
| `attachment/` | Harness context-injection payloads — 23 instances, one per `attachment.type` (the second dispatch level) |
| `system_event/` | Session lifecycle, telemetry, and error events — 10 instances, one per `system.subtype` (the third dispatch level) |
| `envelope_class/` | Common-field presence contract — 3 instances (Full Envelope / Session-Scoped / Detached) partitioning all 19 top-level kinds |
| `settings/` | Settings file structure and protocols — 3 instances (global, project, version lock) |
| `format/` | Ancillary file formats — 7 instances (history, credentials, debug, shell-snapshots, tasks, commands, JSON response) |
| `taxonomy/` | Four-level concept hierarchy (Project/Conversation/Session/Entry) — 3 instances |
| `endpoint/` | Anthropic HTTP endpoint wire contracts — 11 instances (URL, auth, schema, errors) plus one unnumbered cross-endpoint field index |
| `model/` | Claude API model catalog — 13 instances (model IDs, capabilities, workspace defaults) |
| `param/` | CLI parameter specifications — one file per runtime parameter (159 instances) |
| `tool/` | Built-in tools available in Claude Code sessions — 43 instances (14 categories) |
| `subcommand/` | CLI subcommands — 19 instances (12 listed in `claude --help`, 7 functional but hidden from it) |
| `version/` | Claude Code release changelog — 116 instances (2.1.74–2.1.220) |
| `fault/` | Fault collection — all known error, silent failure, and quirk conditions of the `claude` binary (index-only; no numbered instances by design) |
| `pattern/` | Reusable version-pinning design pattern documentation — 1 instance |
| [001_entity.md](001_entity.md) | Cross-entity index: all collection types with instance counts |

**Total**: **485 instances** across 19 collections, as of 2026-08-27 — 484 files matching `NNN_*.md`, plus `behavior/016h_b16h_tools_system_prompt.md`, whose suffixed number marks it a sub-instance of B16 rather than a numbered peer. Two files sit outside that count: `endpoint/account_field_index.md` (an unnumbered cross-endpoint field index, not an endpoint instance) and this collection's own master index [001_entity.md](001_entity.md), which carries the authoritative per-collection breakdown.

Re-derive from the filesystem:

```bash
cd contract/claude_code/docs
for d in */; do printf '%-12s %3d\n' "${d%/}" "$(ls "$d" | grep -cE '^[0-9]{3}_')"; done
find . -mindepth 2 -name '*.md' ! -name 'readme.md' | wc -l   # → 486 = 485 instances + 1 aux index
```

The per-collection loop counts strictly-numbered files, so it reports `behavior 37`; add `016h` back for the 38 the table states.
