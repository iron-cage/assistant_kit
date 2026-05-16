# Parameter :: 2. `--print`

Explicit print mode flag. When a message is given, print mode is the
default — this flag is a backward-compatible explicit alias.
Captures Claude's stdout and prints it instead of passing through
the TTY.

- **Aliases:** `-p`
- **Type:** bool (standalone flag)
- **Default:** auto (active when message given; inactive for bare REPL)
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Claude-Native Flags](../param_group.md#group--1-claude-native-flags)

```sh
clr "Explain this function"        # print mode by default
clr -p "Explain this function"     # same — explicit alias
output=$(clr "List files" --model sonnet)
```

**Note:** Print mode without a message exits with error code 1.
