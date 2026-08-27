# Subcommand: stop

Stop a background session, keeping its conversation.

> **Hidden subcommand** — functional in v2.1.220 but absent from `claude --help`. See [readme.md](readme.md) § Hidden Subcommands.

### Usage

```
claude stop <id>
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

Halts a running background session while retaining its conversation, which can
be resumed later with [`claude attach <id>`](013_attach.md).

Three commands in this family terminate something, and the distinctions matter:

| Command | Stops | Conversation kept | Works on an exited session |
|---------|-------|-------------------|----------------------------|
| `claude stop <id>` | One background session | Yes | No |
| [`claude rm <id>`](018_rm.md) | One background session, plus its worktree | No — deleted | Yes |
| [`claude daemon stop`](014_daemon.md) | The supervisor, and its sessions unless `--keep-workers` | — | — |

### Since

Unverified. No changelog entry in the `version/` collection records this
subcommand's introduction. Present in v2.1.220.

### Verification

```bash
claude stop --help              # → Usage: claude stop <id>
claude --help | grep -cw stop   # → 0 (confirms it is hidden)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [013_attach.md](013_attach.md) | Resume a stopped session |
| doc | [018_rm.md](018_rm.md) | Delete rather than stop |
| doc | [014_daemon.md](014_daemon.md) | `claude daemon stop` shuts down the supervisor instead |
