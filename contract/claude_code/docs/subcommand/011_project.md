# Subcommand: project

Manage Claude Code project state.

### Usage

```
claude project [options] [command]
```

### Options

| Flag | Description |
|------|-------------|
| `-h`, `--help` | Display help for command |

### Sub-subcommands

| Sub-subcommand | Description |
|----------------|-------------|
| `purge [options] [path]` | Delete all Claude Code state for a project — transcripts, tasks, file history, and config entry |
| `help [command]` | Display help for a sub-subcommand |

`purge` accepts `--dry-run`, `-y`/`--yes`, `-i`/`--interactive`, and `--all`
per the v2.1.126 changelog entry that introduced it.

### Description

Operates on the per-project state Claude Code accumulates outside the working
directory itself. `purge` is the destructive half: it removes the project's
transcripts under `~/.claude/projects/{path-encoded}/`, its task state, its
file history, and its entry in the config — everything keyed to that project
path.

This is the only documented mechanism for deleting a project's accumulated
session history. Sessions otherwise accumulate one file per invocation without
rotation or compaction (→ [B6](../behavior/006_b6_session_accumulation.md)),
bounded only by the `cleanupPeriodDays` retention setting.

**Destructive.** `purge` deletes conversation history irreversibly. Use
`--dry-run` first to see what would be removed.

### Since

v2.1.126 for `claude project purge`. The parent `claude project` command was
introduced no later than that release; whether it predates `purge` is not
recorded in the changelog collection.

### Verification

```bash
claude project --help                    # → lists the purge sub-subcommand
claude project purge --dry-run <path>    # → shows what would be deleted, writes nothing
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| behavior | [../behavior/006_b6_session_accumulation.md](../behavior/006_b6_session_accumulation.md) | Sessions accumulate without rotation — what `purge` clears |
| behavior | [../behavior/009_b9_storage_path_encoding.md](../behavior/009_b9_storage_path_encoding.md) | How `[path]` maps to a storage directory name |
| doc | [../storage/001_projects_directory.md](../storage/001_projects_directory.md) | Projects directory layout |
