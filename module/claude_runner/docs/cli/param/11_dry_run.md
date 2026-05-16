# Parameter :: 11. `--dry-run`

Print the assembled command that would be executed without actually
invoking the Claude Code subprocess. Useful for debugging flag
combinations.

- **Type:** bool (standalone flag)
- **Default:** false
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Runner Control](../param_group.md#group--2-runner-control)

```sh
clr --dry-run "test" --model sonnet --max-tokens 50000
# Output includes: claude --dangerously-skip-permissions --chrome -c --print --model sonnet "test\n\nultrathink"
```
