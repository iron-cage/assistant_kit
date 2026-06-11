# Commands :: Token

Token status commands.

---

### Command :: 7. `.token.status`

Reads `expiresAt` from `~/.claude/.credentials.json` and classifies the active OAuth token as Valid, ExpiringSoon, or Expired. Use this to detect when account rotation is needed.

-- **Parameters:** [`format::`](../param/002_format.md), [`threshold::`](../param/003_threshold.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 2 (runtime: credentials unreadable, expiresAt unparseable)

**Syntax:**

```bash
clp .token.status
clp .token.status threshold::1800
clp .token.status format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format |
| `threshold::` | [`WarningThreshold`](../type/003_warning_threshold.md) | `3600` | ExpiringSoon threshold in seconds |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for the credential file read |

**Algorithm (3 steps):**
1. Read `expiresAt` from `~/.claude/.credentials.json`
2. Classify: `Valid` (`expiresAt > now + threshold::`), `ExpiringSoon` (`now < expiresAt ≤ now + threshold::`), or `Expired` (`expiresAt ≤ now`)
3. Render in requested `format::`

**Examples:**

```bash
clp .token.status
# valid — 47m remaining

clp .token.status threshold::1800
# expiring soon — 25m remaining

clp .token.status format::json
# {"status":"valid","expires_in_secs":2820}
```

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Token Status](../../feature/006_token_status.md) | Token expiry classification algorithm |
| 2 | [Auto Rotate](../../feature/008_auto_rotate.md) | Token status drives auto-rotation trigger |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Token expiry check before rotation decision |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Live token status for diagnostic inspection |
