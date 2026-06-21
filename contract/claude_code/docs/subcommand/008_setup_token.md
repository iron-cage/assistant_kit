# Subcommand: setup-token

Set up a long-lived authentication token.

### Usage

```
claude setup-token [options]
```

### Options

| Flag | Description |
|------|-------------|
| `-h`, `--help` | Display help |

### Sub-subcommands

None.

### Description

Sets up a long-lived authentication token for Claude Code. Requires an active
Claude subscription. This token persists across sessions and avoids the need
for repeated OAuth login flows. Useful for CI/CD environments and headless
server setups where browser-based OAuth is impractical.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [002_auth.md](002_auth.md) | Authentication management |
| doc | [../storage/003_root_files.md](../storage/003_root_files.md) | `.credentials.json` storage |
