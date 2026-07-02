# CLI Command: tools

### Description

List all Claude Code built-in tools with name, category, and description. Use `clr tools` to discover available tools before constructing `--allowed-tools` or `--disallowed-tools` flags for a `run` invocation.

-- **Exit Codes:** 0 (success)

### Syntax

```sh
clr tools
```

### Parameters

None. The `tools` command accepts no command-specific flags or arguments. Universal help flags (`--help`, `-h`) are supported.

**Algorithm (1 step):** Read tool definitions from `contract/claude_code/docs/tool/`; render a plain-style table of name, category, and description; exit 0.

### Examples

```sh
# List all Claude Code tools
clr tools

# Pipe to grep for a specific tool
clr tools | grep "Bash"
```

### Notes

Tool data is sourced from the contract documentation at `contract/claude_code/docs/tool/`. The table is rendered in plain style matching `clr ps` output conventions.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`run`](01_run.md) | `--allowed-tools` and `--disallowed-tools` flags reference tool names listed here |
| 2 | [`help`](02_help.md) | Complementary discovery: `help` lists commands; `tools` lists tools |

### Referenced Parameter Groups

*None — `tools` accepts no parameters.*

### Referenced User Stories

*None — tool listing is an introspection utility without a dedicated user story.*

---

**Category:** CLI discoverability
**Complexity:** 0
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low
