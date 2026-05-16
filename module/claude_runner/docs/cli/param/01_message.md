# Parameter :: 1. `[MESSAGE]`

Free-form prompt text sent to Claude Code. Multiple positional words are
joined with spaces. When a message is given, print mode is the default;
use `--interactive` to override to TTY passthrough.

- **Type:** [`MessageText`](../type.md#type--1-messagetext)
- **Default:** — (none; interactive REPL when absent)
- **Command:** [`run`](../command.md#command--1-run)

```sh
clr "Fix the bug in auth.rs"
clr Fix the bug       # equivalent — words joined with space
```
