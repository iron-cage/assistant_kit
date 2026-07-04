# Parameter: 24. `field::`

When set, outputs the raw resolved path value for a single named field instead of the full path listing. Useful for shell scripts that need one specific path without parsing multi-line output or piping through `jq`.

- **Default:** `""` (omit to show all paths)
- **Constraints:** Must be one of: `base`, `credentials`, `credential_store`, `projects`, `stats`, `settings`, `session_env`, `sessions`; unknown value exits 1 with an error message listing all valid field names
- **Purpose:** Script integration — eliminates parsing or `jq` when only one path value is needed (e.g., locating the credential store in a refresh script).

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

### Referenced Type

- **Fundamental Type:** `String`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Output Control](../param_group/001_output_control.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.paths`](../command/004_paths.md#command-8-paths) | Single path value extraction |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | Direct path value for scripts without parsing |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Locate credential store for diagnostic tooling |
