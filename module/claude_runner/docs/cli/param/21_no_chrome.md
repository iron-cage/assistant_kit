# Parameter :: 21. `--no-chrome`

Suppress the automatic `--chrome` injection. When set, no `--chrome` flag
is forwarded to the `claude` subprocess.

- **Type:** bool (standalone flag)
- **Default:** false (chrome injection is **ON** by default; this flag turns it **OFF**)
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Runner Control](../param_group.md#group--2-runner-control)

```sh
clr "Fix bug"              # sends: --chrome (default)
clr --no-chrome "Fix bug"  # sends: no --chrome flag
```

**Note:** Use `--no-chrome` when running in headless or CI environments
where no Chrome instance is available, or when you want to prevent the
Claude-in-Chrome browser integration from activating.
