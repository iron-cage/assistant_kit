# bare

Minimal mode for scripted `-p` calls — skips discovery, hooks, and keychain reads.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--bare` |
| Env Var | — (but *sets* `CLAUDE_CODE_SIMPLE=1`; see [159_simple.md](159_simple.md)) |
| Config Key | — |

### Type

bool

### Default

`false`

### Since

v2.1.81 (2026-03-20) — [`../version/008_v2_1_81.md`](../version/008_v2_1_81.md)

### Description

The single most behavior-changing flag in the CLI surface. Full v2.1.220 help text:

> Minimal mode: skip hooks, LSP, plugin sync, attribution, auto-memory, background prefetches, keychain reads, and CLAUDE.md auto-discovery. Sets `CLAUDE_CODE_SIMPLE=1`. Anthropic auth is strictly `ANTHROPIC_API_KEY` or `apiKeyHelper` via `--settings` (OAuth and keychain are never read). 3P providers (Bedrock/Vertex/Foundry) use their own credentials. Skills still resolve via `/skill-name`. Explicitly provide context via: `--system-prompt[-file]`, `--append-system-prompt[-file]`, `--add-dir` (CLAUDE.md dirs), `--mcp-config`, `--settings`, `--agents`, `--plugin-dir`.

**What this means for automation.** Everything `--bare` skips is a form of *implicit* input — hooks, ambient `CLAUDE.md` files, plugin state, the OS keychain. Removing them makes a run reproducible from its arguments alone, which is why the flag exists for scripted `-p`. The cost is that every piece of context must now be passed explicitly through the eight flags the help text enumerates.

**Auth narrows to two paths.** Under `--bare`, OAuth and keychain are *never* read. A machine that authenticates interactively will fail under `--bare` unless `ANTHROPIC_API_KEY` is exported or an `apiKeyHelper` is supplied via `--settings`. This is the most common `--bare` surprise.

**Known fixes worth knowing.** v2.1.86 fixed `--bare` dropping MCP tools in interactive sessions and silently discarding messages enqueued mid-turn. v2.1.153 fixed subagent frontmatter MCP servers ignoring `--bare`. v2.1.152 fixed the Agent tool description referencing an agent list that is never delivered under `--bare`.

### Verification

```bash
claude --help | grep -A12 -- '--bare'   # → the full text quoted above

V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_SIMPLE "$V"        # → 20 (the env var --bare sets)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [159_simple.md](159_simple.md) | `CLAUDE_CODE_SIMPLE` — the env var `--bare` sets |
| doc | [007_api_key.md](007_api_key.md) | `ANTHROPIC_API_KEY` — one of the two auth paths left under `--bare` |
| doc | [051_print.md](051_print.md) | `-p` / `--print` — the mode `--bare` exists to support |
| doc | [../version/008_v2_1_81.md](../version/008_v2_1_81.md) | Release introducing the flag |
