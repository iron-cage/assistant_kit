# Commands :: Paths

File path resolution commands.

---

### Command :: 8. `.paths`

Displays all canonical `~/.claude/` file and directory paths resolved from `HOME`. Use this for diagnostics and tooling integration.

-- **Parameters:** [`format::`](../param/002_format.md), [`field::`](../param/024_field.md), [`trace::`](../param/023_trace.md)
-- **Exit:** 0 (success) | 1 (usage: unknown `field::` value) | 2 (runtime: HOME not set)

**Syntax:**

```bash
clp .paths
clp .paths format::json
clp .paths field::credential_store
clp .paths field::credentials
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format |
| `field::` | `String` | `""` (show all) | Output a single named path value; valid: `base`, `credentials`, `credential_store`, `projects`, `stats`, `settings`, `session_env`, `sessions` |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr for home resolution source and each resolved path |

**Examples:**

```bash
clp .paths
# credentials:      /home/user/.claude/.credentials.json
# credential_store: /home/user/.persistent/claude/credential/
# projects:         /home/user/.claude/projects/
# stats:       /home/user/.claude/stats-cache.json
# settings:    /home/user/.claude/settings.json
# session-env: /home/user/.claude/session-env/
# sessions:    /home/user/.claude/sessions/

clp .paths format::json
# {"base":"/home/user/.claude","credentials":"/home/user/.claude/.credentials.json",...}

clp .paths field::credential_store
# /home/user/.persistent/claude/credential/

clp .paths field::unknown
# exit 1: "unknown field 'unknown'; valid: base, credentials, credential_store, ..."
```

**Notes:**
- `field::` takes priority over `format::` — when both are provided, `format::` is ignored.
- Field names use underscores (`session_env`), matching the JSON key names from `format::json`.
