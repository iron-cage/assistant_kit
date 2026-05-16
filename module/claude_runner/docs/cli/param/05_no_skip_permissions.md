# Parameter :: 5. `--no-skip-permissions`

Disable the automatic `--dangerously-skip-permissions` flag that `clr` injects into every
invocation by default. Without this flag, every `clr` call silently passes
`--dangerously-skip-permissions` to the `claude` subprocess, bypassing all tool permission
prompts.

- **Type:** bool (standalone flag)
- **Default:** false (bypass is **ON** by default; this flag turns it **OFF**)
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Runner Control](../param_group.md#group--2-runner-control)

```sh
clr --no-skip-permissions "Fix bug"   # bypass disabled — claude will prompt for tool approvals
```

**Note:** `--dangerously-skip-permissions` is no longer a user-facing flag. It is injected
automatically unless `--no-skip-permissions` is given. See the
[Default Flags Invariant](../../invariant/001_default_flags.md#invariant-statement) in the invariant.
