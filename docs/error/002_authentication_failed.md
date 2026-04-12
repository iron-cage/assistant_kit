# Error: Authentication Failed

### Scope

- **Purpose**: Document the authentication error family that Claude Code emits when the API key or OAuth token is invalid, expired, or missing.
- **Responsibility**: Describe the exact error variants, what each trigger condition looks like, and how to restore a working session.
- **In Scope**: HTTP 401 authentication_error, OAuth error variants (invalid code, timeout, expired sign-in), ANTHROPIC_API_KEY override conflicts.
- **Out of Scope**: Authorization permission errors (HTTP 403 → future `error/` doc instance); account rotation mechanics (→ `../../module/claude_profile/src/commands.rs`).

### Abstract

Claude Code emits authentication errors when it cannot establish a valid identity with the Anthropic API. There are two distinct error families: **API authentication errors** (returned as HTTP 401 during a request) and **OAuth errors** (returned during the login flow or session validation). Both are fatal for the current operation; Claude Code exits non-zero.

**API authentication error** (terminal output):
```
API Error: 401 {"type":"error","error":{"type":"authentication_error","message":"Invalid authentication credentials"},"request_id":"req_011..."}
```

**OAuth error variants** (terminal output during login or on startup):
```
OAuth error: Invalid code. Please make sure the full code was copied
OAuth error: Request failed with status code 500
OAuth error: timeout of 15000ms exceeded
OAuth account information not found in config
Unable to start session: account information is unavailable because your sign-in has expired
Unable to start session: account information is unavailable
```

### Trigger Conditions

- **Expired OAuth token**: The session cookie or OAuth access token stored in `~/.claude/.credentials.json` has expired; Claude Code cannot refresh it automatically.
- **`ANTHROPIC_API_KEY` env var override**: When this variable is set, Claude Code uses it instead of the stored OAuth credentials. A revoked, deleted, or wrong-org key produces a 401 on every request, including logout.
- **Truncated or expired login code**: During `claude login`, the one-time code was pasted incorrectly or the browser session timed out before completion.
- **DNS failure during OAuth**: `auth.anthropic.com` is unreachable (corporate proxy, VPN split-tunnel, or strict firewall) — produces `OAuth error: timeout of 15000ms exceeded`.
- **Disabled organization**: The API key belongs to a disabled organization — the 401 message body contains `"This organization has been disabled"`.

### Recovery

1. **Unset `ANTHROPIC_API_KEY`**: If this env var is set in the shell profile, unset or rename it. It overrides subscription OAuth credentials and a stale key silently breaks every operation.
   ```bash
   unset ANTHROPIC_API_KEY
   ```
2. **Re-authenticate**: Run `claude logout` then `claude login` and complete the browser flow. If `claude logout` itself fails with a 401, delete the credential file manually:
   ```bash
   rm ~/.claude/.credentials.json && claude login
   ```
3. **Check network for OAuth DNS**: Ensure `auth.anthropic.com` and `api.anthropic.com` resolve. On corporate networks, add them to proxy exceptions or use `clp account auto-rotate` to switch to an account that still has a valid token.
4. **Re-copy the login code**: If `OAuth error: Invalid code`, the full alphanumeric code must be copied precisely from the browser; retry `claude login`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| error | [error/001_rate_limit_reached.md](001_rate_limit_reached.md) | Rate-limit error — different HTTP status (429) and recovery path |
| source | `../../module/claude_profile/src/commands.rs` | `account auto-rotate` and credential status commands |
| source | `../../module/claude_profile/src/token.rs` | Credential file parsing and expiry detection |
