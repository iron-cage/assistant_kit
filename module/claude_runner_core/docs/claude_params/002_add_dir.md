# add_dir

Grant Claude tool access to additional directories beyond the working directory.

## Type

**CLI** — one or more path values

## Syntax

```
claude --add-dir <directory> [<directory> ...]
```

## Default

None (only the working directory is accessible)

## Description

By default, Claude Code's file tools (Read, Write, Edit, Glob, Bash) are scoped to the current working directory. `--add-dir` expands that scope to include additional directories.

The flag accepts multiple directories as space-separated arguments. Each directory is added to the allowed path list for the session.

Use cases:
- Projects with files outside the main repo (shared libraries, configs)
- Monorepos where tools need to cross package boundaries
- Referencing `/tmp` or other system directories

## Builder API

Use `with_add_dir()` — Repeated-flag: each call adds one directory path.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_add_dir( "/home/user/project" )
  .with_add_dir( "/home/user/data" )
  .with_message( "Analyze files in both directories" );
```

## Examples

```bash
# Add a shared library directory
claude --add-dir /home/user/shared-lib --print "How does the auth module work?"

# Add multiple directories
claude --add-dir /tmp /etc/myapp --print "Check config consistency"

# In a monorepo, access sibling package
claude --add-dir ../packages/ui --print "Update the component import"
```

## Notes

- Each added directory is included recursively
- Paths should be absolute for predictability
- Does not grant network or process execution access beyond the sandbox
