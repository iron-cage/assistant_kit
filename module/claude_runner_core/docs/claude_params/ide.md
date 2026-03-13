# ide

Automatically connect to an IDE on startup if exactly one valid IDE is available.

## Type

**CLI** — boolean flag

## Syntax

```
claude --ide
```

## Default

off

## Description

When set, Claude Code attempts to connect to an active IDE integration on startup. If exactly one valid IDE connection is available (e.g., a VS Code or JetBrains instance with the Claude Code extension installed and running), it connects automatically.

If multiple IDEs are available, connection is ambiguous and the flag has no effect (or prompts for selection depending on the version).

Use cases:
- Launching Claude from a terminal within a development workflow where IDE context should be injected
- Automated scripts that should augment an active editor session

## Builder API

Use `with_ide()` — Boolean flag: auto-connect to IDE on startup.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_ide( true )
  .with_message( "Run with IDE integration" );
```

## Examples

```bash
# Launch with IDE context if one is running
claude --ide "Help me with this file"

# In a development alias
alias cide='claude --ide'
```

## Notes

- Requires the Claude Code IDE extension installed in the connected IDE
- Only auto-connects when exactly one valid IDE is running; multiple IDEs require manual selection
- The IDE integration provides file context, cursor position, and active editor state to Claude
