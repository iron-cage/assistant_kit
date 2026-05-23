# Parameter :: 24. `field::`

When set, outputs the raw resolved path value for a single named field instead of the full path listing. Useful for shell scripts that need one specific path without parsing multi-line output or piping through `jq`.

- **Type:** `String`
- **Default:** `""` (omit to show all paths)
- **Constraints:** Must be one of: `base`, `credentials`, `credential_store`, `projects`, `stats`, `settings`, `session_env`, `sessions`; unknown value exits 1 with an error message listing all valid field names
- **Commands:** [`.paths`](../command/paths.md#command--8-paths)
- **Purpose:** Script integration — eliminates parsing or `jq` when only one path value is needed (e.g., locating the credential store in a refresh script).
- **Group:** Output Selection

**Examples:**

```text
field::credential_store   → /home/user/.persistent/claude/credential/
field::credentials        → /home/user/.claude/.credentials.json
field::base               → /home/user/.claude
field::session_env        → /home/user/.claude/session-env/
field::unknown            → exit 1: "unknown field 'unknown'; valid: base, credentials, credential_store, projects, stats, settings, session_env, sessions"
```

**Notes:**
- When `field::` is set, `format::` is ignored — output is always the raw string value followed by a newline.
- Field names match the JSON keys from `clp .paths format::json` output (underscores, not hyphens).
