# Subcommand: import

Import config from another AI coding agent into Claude Code.

> **Hidden subcommand** — functional in v2.1.220 but absent from `claude --help`. See [readme.md](readme.md) § Hidden Subcommands.

### Usage

```
claude import [options] [source]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `[source]` | Which agent to import from — `codex` or `gemini` |

### Options

| Flag | Description |
|------|-------------|
| `--dry-run` | Show what would be imported without writing anything |
| `--yes` | Import everything without the interactive picker (headless surfaces) |
| `-h`, `--help` | Display help for command |

### Sub-subcommands

None.

### Description

Migration helper that reads another coding agent's configuration and translates
it into Claude Code's own. Two sources are accepted, both named explicitly in
the help output: `codex` and `gemini`.

Omitting `[source]` or the flags launches an interactive picker; `--yes` skips
it for headless use. `--dry-run` reports the plan without writing, which is the
safe way to inspect what a migration would touch before committing to it.

This is the only subcommand in the hidden set that is unrelated to background
sessions — the other six all operate on the session/daemon lifecycle.

### Since

Unverified. No changelog entry in the `version/` collection records this
subcommand's introduction. Present in v2.1.220.

### Verification

```bash
claude import --help              # → Usage: claude import [options] [source]
claude import --dry-run codex     # → reports what would be imported, writes nothing
claude --help | grep -c import    # → 0 (confirms it is hidden)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [../settings/001_global_settings.md](../settings/001_global_settings.md) | Global settings this command writes into |
| doc | [../settings/002_project_settings.md](../settings/002_project_settings.md) | Project settings this command writes into |
