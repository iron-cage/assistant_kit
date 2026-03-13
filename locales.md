> **Generated.** Do not edit manually. Maintained by `.locale.doc.generate`.
> Source of truth: `locales.config.yml` + `.persistent/locale.toml`.

# Locales — agent_kit

A **locale** is a named, bounded directory representing a self-contained unit of development work. See [`willbe/locate/module/locate/docs/locale.md`](../../willbe/locate/module/locate/docs/locale.md) for the full specification.

All paths are relative to `~/pro/lib/wip_core/claude_tools/dev`. `task` — Y = `task/` directory initialized.

## Summary

| # | rel-path | name | type | lang | purpose | task | last_active |
|---|----------|------|------|------|---------|------|-------------|
| 1 | `module/claude_storage_core` | claude_storage_core | rust_crate | rs | Core library for Claude Code filesystem storage access | N | 2026-03-14 |
| 2 | `module/claude_storage` | claude_storage | rust_crate | rs | CLI tool for exploring Claude Code filesystem storage | N | 2026-03-14 |
| 3 | `module/claude_profile` | claude_profile | rust_crate | rs | Account credential management, token status, ~/.claude/ paths | N | 2026-03-28 |
| 4 | `module/claude_runner_core` | claude_runner_core | rust_crate | rs | Claude Code process execution with builder pattern | N | 2026-03-14 |
| 5 | `module/claude_runner` | claude_runner | rust_crate | rs | CLI for executing Claude Code with configurable builder | N | 2026-03-14 |

---

## Profile

### workspace :: agent_kit

| field | value |
|-------|-------|
| path | `lib/wip_core/claude_tools/dev` |
| parent | `lib/wip_core` |
| type | rust_workspace |
| lang | rs |
| canonical | Y |
| task | Y |
| last_active | 2026-03-14 |

**Purpose.** AI agent tooling infrastructure. Provides credential management, session state persistence (`claude_profile`, `claude_storage`, `claude_storage_core`) and execution management (`claude_runner_core`, `claude_runner`) for autonomous multi-turn workflows — currently focused on Claude Code, designed to expand to other agents.
