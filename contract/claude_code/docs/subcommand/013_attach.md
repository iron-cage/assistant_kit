# Subcommand: attach

Open a background session in this terminal.

> **Hidden subcommand** — functional in v2.1.220 but absent from `claude --help`. See [readme.md](readme.md) § Hidden Subcommands.

### Usage

```
claude attach <id>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<id>` | Background session identifier |

### Options

None. `--help` prints the usage line only; there is no options table.

### Sub-subcommands

None.

### Description

Attaches the current terminal to a running background session, switching the
terminal into the agent view. `←` returns to the agent view and `Ctrl+Z` drops
back to the shell. The session keeps running either way — detaching does not
stop it.

This is also the documented way to resume a session previously halted with
[`claude stop`](019_stop.md), whose conversation is retained.

Part of the background-session command family — see [readme.md](readme.md)
§ Hidden Subcommands for the full set and their relationships.

### Since

Unverified. `claude attach` appears in changelog entries from v2.1.198 onward,
but those are bug-fix references rather than an introduction entry, so the
command predates them by an unknown margin.

### Verification

```bash
claude attach --help      # → Usage: claude attach <id>
claude --help | grep -c attach   # → 0 (confirms it is hidden, not documented)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [019_stop.md](019_stop.md) | Stop a session; resume it with this command |
| doc | [016_logs.md](016_logs.md) | Read output without attaching |
| doc | [014_daemon.md](014_daemon.md) | Supervisor that hosts background sessions |
| behavior | [../behavior/036_b36_background_task_lifecycle.md](../behavior/036_b36_background_task_lifecycle.md) | Background-task lifecycle env vars |
