# Subcommand: daemon

Manage the background-session supervisor.

> **Hidden subcommand** — functional in v2.1.220 but absent from `claude --help`. See [readme.md](readme.md) § Hidden Subcommands.

### Usage

```
claude daemon [subcommand] [options]
```

### Options

| Flag | Description |
|------|-------------|
| `--json-path <p>` | Config file (default: `~/.claude/daemon.json`) |
| `--log-file <p>` | Log file (default: `~/.claude/daemon.log`) |
| `--help`, `-h` | Show help |

### Sub-subcommands

| Sub-subcommand | Description |
|----------------|-------------|
| `run [json-path]` | Run the supervisor in the foreground (default when piped) |
| `status` | Show daemon pid, version, uptime |
| `logs` | Tail the daemon log (Ctrl-C to stop) |
| `uninstall` | Remove the background service (launchctl/systemd) |
| `stop` | Shut down the supervisor and terminate background sessions |

`stop` takes two further flags:

| Flag | Description |
|------|-------------|
| `--any` | Also stop a transient (non-service) daemon |
| `--keep-workers` | Leave detached sessions running |

### Description

The supervisor process that hosts background sessions. Its own help text states
that **service install is disabled in this version** — the daemon runs on demand
and exits when the last client disconnects, rather than being installed as a
persistent launchctl/systemd service. `uninstall` therefore exists to remove a
service registered by an earlier version.

Note the two distinct stop verbs: `claude daemon stop` shuts down the
*supervisor* (and, unless `--keep-workers` is given, the sessions it hosts),
whereas [`claude stop <id>`](019_stop.md) halts a *single* background session
and leaves the supervisor running.

Two files carry its state, both directly under `~/.claude/`: `daemon.json`
(config) and `daemon.log`.

### Since

Unverified. `claude daemon status` appears in a v2.1.141 bug-fix entry, so the
command exists at least by that release; no introduction entry is recorded in
the changelog collection.

### Verification

```bash
claude daemon --help      # → Usage: claude daemon [subcommand] [options]
claude daemon status      # → pid, version, uptime (or reports no daemon running)
claude --help | grep -c daemon   # → 0 (confirms it is hidden)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [013_attach.md](013_attach.md) | Attach to a session this daemon hosts |
| doc | [019_stop.md](019_stop.md) | Stop one session rather than the supervisor |
| doc | [017_respawn.md](017_respawn.md) | Restart sessions onto the current binary |
| behavior | [../behavior/036_b36_background_task_lifecycle.md](../behavior/036_b36_background_task_lifecycle.md) | Background-task lifecycle env vars |
| doc | [../storage/003_root_files.md](../storage/003_root_files.md) | `~/.claude/` root files including `daemon.json` and `daemon.log` |
