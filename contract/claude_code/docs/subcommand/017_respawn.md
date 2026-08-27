# Subcommand: respawn

Restart a background session so it picks up the current Claude binary.

> **Hidden subcommand** — functional in v2.1.220 but absent from `claude --help`. See [readme.md](readme.md) § Hidden Subcommands.

### Usage

```
claude respawn <id>|--all
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<id>` | Background session identifier |
| `--all` | Restart every background session instead of one |

### Options

None beyond `--all` above, which occupies the argument position rather than
being a modifier flag.

### Sub-subcommands

None.

### Description

Restarts one background session, or all of them with `--all`, so they run on
the currently installed `claude` binary. Background sessions outlive the
process that launched them, so after an update they keep running the older
binary until explicitly respawned.

Pairs naturally with [`claude update`](009_update.md): update installs the new
binary, `respawn --all` moves existing background sessions onto it.

### Since

Unverified. `claude respawn` appears in a v2.1.144 bug-fix entry (a stopped
session reporting "stopped" instead of running), so the command exists at least
by that release; no introduction entry is recorded.

### Verification

```bash
claude respawn --help              # → Usage: claude respawn <id>|--all
claude --help | grep -c respawn    # → 0 (confirms it is hidden)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [009_update.md](009_update.md) | Installs the binary that respawn adopts |
| doc | [014_daemon.md](014_daemon.md) | Supervisor hosting the sessions being respawned |
| doc | [019_stop.md](019_stop.md) | Stop rather than restart |
