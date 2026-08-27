# Subcommand: logs

Print a background session's recent terminal output.

> **Hidden subcommand** — functional in v2.1.220 but absent from `claude --help`. See [readme.md](readme.md) § Hidden Subcommands.

### Usage

```
claude logs <id>
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

Prints the recent terminal output of a background session without attaching to
it. This is the read-only counterpart to [`claude attach`](013_attach.md):
`attach` takes over the terminal and switches to the agent view, `logs` just
dumps output and returns.

Distinct from [`claude daemon logs`](014_daemon.md), which tails the
*supervisor's* log at `~/.claude/daemon.log` rather than any one session's
output. Same verb, different subject.

### Since

Unverified. No changelog entry in the `version/` collection records this
subcommand's introduction. Present in v2.1.220.

### Verification

```bash
claude logs --help              # → Usage: claude logs <id>
claude --help | grep -c logs    # → 0 (confirms it is hidden)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [013_attach.md](013_attach.md) | Attach interactively instead of dumping output |
| doc | [014_daemon.md](014_daemon.md) | `claude daemon logs` tails the supervisor log instead |
