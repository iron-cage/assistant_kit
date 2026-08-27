# mcp_debug

> ❌ **Removed** — not present in v2.1.220. Use `--debug` (or `--debug=mcp`) instead.

Formerly enabled MCP debug mode, showing errors and diagnostic output from MCP servers. The flag no longer exists.

### Forms

| | Value |
|-|-------|
| CLI Flag | ~~`--mcp-debug`~~ — removed; rejected as an unknown option |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off`

### Since

pre-v1.0 (unverified) — **removed by v2.1.220**

### Description

Formerly enabled MCP (Model Context Protocol) debug mode, showing errors and diagnostic output from MCP servers. The previous revision of this doc described the flag as deprecated but still accepted "for backwards compatibility"; that is no longer true. In v2.1.220 the flag is gone: it appears nowhere in `claude --help`, and passing it produces `error: unknown option '--mcp-debug'` — byte-identical to the response for a fabricated flag name.

**Replacement:** `--debug`, optionally filtered with `--debug=mcp`.

**Verify:**

```bash
claude --help 2>&1 | grep -c 'mcp-debug'          # → 0
claude -p --mcp-debug </dev/null                  # → error: unknown option '--mcp-debug'
claude -p --nonexistent-flag-xyz </dev/null       # → same error shape (control)
```

Note that `claude --mcp-debug --version` is **not** a valid probe — `--version` short-circuits before option parsing and exits 0 for any flag, including fabricated ones. The `-p <flag> </dev/null` form parses options first and is the reliable test.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [019_debug.md](019_debug.md) | `--debug` supersedes this flag |
| doc | [039_mcp_config.md](039_mcp_config.md) | MCP server configuration |