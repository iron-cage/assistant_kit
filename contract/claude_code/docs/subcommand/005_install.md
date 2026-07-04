# Subcommand: install

Install Claude Code native build.

### Usage

```
claude install [options] [target]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `[target]` | Version to install: `stable`, `latest`, or specific version string |

### Options

| Flag | Description |
|------|-------------|
| `--force` | Force installation even if already installed |
| `-h`, `--help` | Display help |

### Sub-subcommands

None.

### Description

Installs a specific version of the Claude Code native build. The `[target]`
argument accepts `stable` (production release), `latest` (most recent build),
or a specific version number. The `--force` flag reinstalls even if the
target version is already installed.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [009_update.md](009_update.md) | Update/upgrade subcommand |
| doc | [../param/050_preferred_version_spec.md](../param/050_preferred_version_spec.md) | Preferred version specification |
| doc | [../param/049_preferred_version_resolved.md](../param/049_preferred_version_resolved.md) | Resolved version tracking |
| doc | [../pattern/001_version_pinning.md](../pattern/001_version_pinning.md) | Synthesis: full version-pinning landscape |
