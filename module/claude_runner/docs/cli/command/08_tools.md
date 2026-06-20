# CLI Command: tools

List all Claude Code built-in tools with name, category, and description.

**Syntax:**

```sh
clr tools
```

Displays a comprehensive table of all Claude Code tools with columns for tool name,
category, and description.

**Parameters:**

None. The `tools` command accepts no command-specific flags or arguments. Universal help flags (`--help`, `-h`) are supported.

**Exit Codes:**

| Code | Meaning |
|------|---------|
| 0 | Success |

**Examples:**

```sh
# List all Claude Code tools
clr tools

# Pipe to grep for a specific tool
clr tools | grep "Bash"
```

**Notes:**

Tool data is sourced from the contract documentation at `contract/claude_code/docs/tool/`.
The table is rendered in plain style matching `clr ps` output conventions.
