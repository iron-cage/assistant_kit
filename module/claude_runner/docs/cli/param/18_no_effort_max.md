# Parameter :: 18. `--no-effort-max`

Suppress the automatic `--effort max` injection. When set, no `--effort` flag
is forwarded to the `claude` subprocess at all.

- **Type:** bool (standalone flag)
- **Default:** false (effort max injection is **ON** by default; this flag turns it **OFF**)
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Runner Control](../param_group.md#group--2-runner-control)

```sh
clr "Fix bug"                      # sends: --effort max (default)
clr --no-effort-max "Fix bug"      # sends: no --effort flag at all
```

**Note:** Use `--no-effort-max` when targeting models or configurations that
do not support the `--effort` flag, or when you need claude's native default
effort level without any override.

**Note:** `--effort <level>` and `--no-effort-max` are mutually exclusive.
If `--no-effort-max` is set, any `--effort` value is ignored.
