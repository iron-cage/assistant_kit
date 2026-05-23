# Parameter :: 13. `--trace`

Print diagnostic details to stderr before executing the subprocess. Unlike `--dry-run`,
execution still proceeds — the trace is shown as a diagnostic prefix, then the
subprocess is launched. Mirrors shell `set -x` semantics.

- **Type:** bool (standalone flag)
- **Default:** false
- **Command:** [`run`](../command.md#command--1-run), [`isolated`](../command.md#command--2-isolated), [`refresh`](../command.md#command--3-refresh)
- **Group:** [Runner Control](../param_group.md#group--2-runner-control)

What `--trace` shows depends on the command:

- **`run`**: assembled env vars + full `claude` subprocess command
- **`isolated`**: creds path, temp HOME path, timeout, forwarded args, `claude` invocation
- **`refresh`**: creds path, temp HOME path, timeout, fixed args `["--print", "."]`

```sh
# Trace on run
clr --trace "Fix bug"
# Stderr: CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000
# Stderr: claude --dangerously-skip-permissions --chrome -c --print "Fix bug\n\nultrathink"
# Then: subprocess executes normally

# Trace on isolated
clr isolated --creds creds.json --trace "Fix bug"
# Stderr: creds=/path/to/creds.json timeout=30 args=["--print", "Fix bug"]
# Then: run_isolated() executes

# Trace on refresh
clr refresh --creds creds.json --trace
# Stderr: creds=/path/to/creds.json timeout=45 args=["--print", "."]
# Then: run_isolated() executes
```

**Note:** `--trace` prints to stderr so it does not pollute captured stdout in print mode.
Combine with `--dry-run` if you want to preview without executing (`run` command only).
