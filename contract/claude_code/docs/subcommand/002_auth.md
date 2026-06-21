# Subcommand: auth

Manage authentication.

### Usage

```
claude auth [command]
```

### Sub-subcommands

| Command | Description |
|---------|-------------|
| `login [options]` | Sign in to your Anthropic account |
| `logout` | Log out from your Anthropic account |
| `status [options]` | Show authentication status |

### Description

Manages Claude Code authentication state. Supports OAuth-based sign-in to
Anthropic accounts, sign-out, and status checking. Authentication tokens are
stored in `~/.claude/.credentials.json`.

The `login` sub-subcommand opens a browser-based OAuth flow. The `status`
sub-subcommand shows the current authentication state including account
identity and token validity.

### Sub-subcommand Options

#### `claude auth login`

| Option | Description |
|--------|-------------|
| `--email <email>` | Pre-populate email address on the login page |
| `--sso` | Force SSO login flow |

#### `claude auth status`

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON (default) |
| `--text` | Output as human-readable text |

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [../storage/003_root_files.md](../storage/003_root_files.md) | `.credentials.json` storage |
| doc | [../formats/002_credentials.md](../formats/002_credentials.md) | Credentials file format |
| doc | [../endpoint/004_oauth_token.md](../endpoint/004_oauth_token.md) | OAuth token refresh endpoint |
