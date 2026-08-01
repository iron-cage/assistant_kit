# Commands: Token

> **DEPRECATED** — `.token.status` (the only command in this file) has been removed. Token expiry classification is now exposed via `.credentials.status`'s `token`/`expires` fields (see [command/002_credentials.md](002_credentials.md#command-10-credentialsstatus)). The content below is preserved as historical record of the removed command.

Token status commands. `.credentials.status` (see [command/002_credentials.md](002_credentials.md#command-10-credentialsstatus)) offers the same classification alongside broader account metadata — use `.token.status` when only the bare token classification is needed.

---

### Command: 7. `.token.status`

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
| `trace::` | `bool` | `0` | Print timestamped diagnostic lines to stderr for the credential file read |

**Algorithm (3 steps):**
1. Read `expiresAt` from `~/.claude/.credentials.json`; absent (active account is `backend: redirect`) → classify `Static` immediately, skip step 2
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

clp .token.status
# static   (redirect-backend account — no expiry)
```

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [format::](../param/002_format.md) | Output format |
| 2 | [threshold::](../param/003_threshold.md) | ExpiringSoon threshold in seconds |
| 3 | [trace::](../param/023_trace.md) | Diagnostic trace output |

**Notes:**
- **Redirect backend:** a `backend: redirect` active account always classifies `Static`, checked before any `threshold::` comparison — `expiresAt` is absent, never merely far away. See [feature/071](../../feature/071_redirect_backend_accounts.md).

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Token Status](../../feature/006_token_status.md) | Token expiry classification algorithm |
| 2 | [Auto Rotate](../../feature/008_auto_rotate.md) | Token status drives auto-rotation trigger |
| 3 | [Redirect Backend Accounts](../../feature/071_redirect_backend_accounts.md) | `Static` classification for `backend: redirect` accounts |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Token expiry check before rotation decision |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Live token status for diagnostic inspection |

### Referenced Parameter Groups

| # | Group | Parameters Used |
|---|-------|-----------------|
| 1 | [Output Control](../param_group/001_output_control.md) | `format::` |
| 2 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | `trace::` |

### Referenced Formats

| # | Format | Trigger |
|---|--------|---------|
| 1 | [text](../format/001_text.md) | `format::text` (default) |
| 2 | [json](../format/002_json.md) | `format::json` |
