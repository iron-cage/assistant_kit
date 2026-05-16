# Parameter :: 12. `--verbosity`

Control how much diagnostic output the runner itself emits. Does not
affect Claude Code subprocess output.

- **Type:** [`VerbosityLevel`](../type.md#type--5-verbositylevel)
- **Default:** 3 (normal)
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Runner Control](../param_group.md#group--2-runner-control)
- **Validation:** must be integer 0–5; out of range → error

```sh
clr --verbosity 0 "Silent run"    # suppress runner output
clr --verbosity 4 "Debug"         # verbose command preview
```
