# Parameter :: 13. `--trace`

Print the assembled environment variables and command to stderr before executing the
Claude Code subprocess. Unlike `--dry-run`, execution still proceeds — the command is
shown as a diagnostic prefix, then the subprocess is launched. Mirrors shell `set -x`
semantics.

- **Type:** bool (standalone flag)
- **Default:** false
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Runner Control](../param_group.md#group--2-runner-control)

```sh
clr --trace "Fix bug"
# Stderr: CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000
# Stderr: claude --dangerously-skip-permissions --chrome -c --print "Fix bug\n\nultrathink"
# Then: subprocess executes normally
```

**Note:** `--trace` prints to stderr so it does not pollute captured stdout in print mode.
Combine with `--dry-run` if you want to preview without executing.
