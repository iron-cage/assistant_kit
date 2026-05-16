# Parameter :: 3. `--model`

Select the Claude model for this invocation.

- **Type:** [`ModelName`](../type.md#type--4-modelname)
- **Default:** — (Claude Code default)
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Claude-Native Flags](../param_group.md#group--1-claude-native-flags)
- **Validation:** requires a value; `--model` at end of argv → error

```sh
clr "Explain" --model sonnet
clr --model opus "Fix bug"
```
