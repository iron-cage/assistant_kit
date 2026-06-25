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
| `trace::` | `bool` | `0` | Print timestamped diagnostic lines to stderr for home resolution source and each resolved path |

**Algorithm (3 steps):**
1. Resolve `HOME`; derive all canonical `~/.claude/` paths via `ClaudePaths`
2. `(when field:: provided)` Extract the named field value; exit 1 on unknown field name
3. Render all paths (or single field) in requested `format::` (`field::` takes priority over `format::`)

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

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [File Topology](../../feature/007_file_topology.md) | Canonical path set resolved by this command |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | Script-accessible path resolution for tooling integration |
| 2 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Path topology check during credential diagnostics |
