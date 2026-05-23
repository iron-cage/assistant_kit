# Commands :: Token

Token status commands.

---

### Command :: 7. `.token.status`

Reads `expiresAt` from `~/.claude/.credentials.json` and classifies the active OAuth token as Valid, ExpiringSoon, or Expired. Use this to detect when account rotation is needed.

-- **Parameters:** [`format::`](../param/02_format.md), [`threshold::`](../param/03_threshold.md)
-- **Exit:** 0 (success) | 2 (runtime: credentials unreadable, expiresAt unparseable)

**Syntax:**

```bash
clp .token.status
clp .token.status threshold::1800
clp .token.status format::json
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `format::` | [`OutputFormat`](../type/02_output_format.md) | `text` | Output format |
| `threshold::` | [`WarningThreshold`](../type/03_warning_threshold.md) | `3600` | ExpiringSoon threshold in seconds |

**Examples:**

```bash
clp .token.status
# valid — 47m remaining

clp .token.status threshold::1800
# expiring soon — 25m remaining

clp .token.status format::json
# {"status":"valid","expires_in_secs":2820}
```
