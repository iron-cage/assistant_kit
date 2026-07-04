# Format: Credentials

### Scope

- **Purpose**: Specify the `~/.claude/.credentials.json` format — the active API authentication token storage.
- **Responsibility**: Authoritative instance for the credentials JSON format — structure, security requirements, write semantics.
- **In Scope**: File location, `claudeAiOauth` structure, security permissions, write semantics.
- **Out of Scope**: Per-account credential store (→ [`../filesystem/003_credential_store.md`](../filesystem/003_credential_store.md)); OAuth token refresh endpoint (→ `../endpoint/004_oauth_token.md`).

### Location

`~/.claude/.credentials.json`

**Format**: Single JSON object.
**Mutability**: Overwritten atomically by `.account.switch`.

### Schema

```json
{
  "claudeAiOauth": {
    "... (authentication data)"
  }
}
```

The `claudeAiOauth` object contains the OAuth authentication payload. The exact internal structure is not publicly documented; it includes access token, refresh token, and expiry metadata.

### Security

**Sensitivity**: High — contains API access tokens.
**Recommended permissions**: `chmod 600 ~/.claude/.credentials.json` (owner read/write only).

### Write Semantics

Written by `.account.switch` when switching active accounts. Also written by the OAuth refresh cycle. Written atomically (read-modify-write with file replacement).

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Formats master index |
| filesystem | [`../filesystem/001_claude_home.md`](../filesystem/001_claude_home.md) | Path location and access pattern |
| filesystem | [`../filesystem/003_credential_store.md`](../filesystem/003_credential_store.md) | Per-account credential snapshots (different from this active token) |
| endpoint | `../endpoint/004_oauth_token.md` | OAuth token refresh endpoint |
| subcommand | [`../subcommand/002_auth.md`](../subcommand/002_auth.md) | Auth subcommand that manages this file |
| subcommand | [`../subcommand/008_setup_token.md`](../subcommand/008_setup_token.md) | Setup-token subcommand that writes this file |
