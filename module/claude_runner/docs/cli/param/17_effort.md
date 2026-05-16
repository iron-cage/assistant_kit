# Parameter :: 17. `--effort`

Override the reasoning effort level passed to the `claude` subprocess. `clr`
injects `--effort max` automatically on every invocation; this flag overrides
that default to any supported level.

- **Type:** [`EffortLevel`](../type.md#type--7-effortlevel)
- **Default:** max (injected automatically; override with this flag or suppress entirely with `--no-effort-max`)
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Claude-Native Flags](../param_group.md#group--1-claude-native-flags)
- **Validation:** requires a value; unknown level → error listing valid values (`low`, `medium`, `high`, `max`)

```sh
clr "Fix the bug"                  # sends: --effort max (default)
clr --effort medium "Fix the bug"  # sends: --effort medium
clr --effort high "Fix the bug"    # sends: --effort high
```

**Note:** `max` is the default because `clr` is designed for agentic automation
tasks where full reasoning capacity is the correct default. The claude binary's
own default (`medium`) is intentionally overridden here.

**Note:** To suppress the `--effort` flag entirely (pass no effort flag to claude),
use `--no-effort-max`.
