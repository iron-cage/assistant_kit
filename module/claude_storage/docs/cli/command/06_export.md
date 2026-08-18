# Command :: 6. `.export`

### Scope

- **Purpose**: Specify the `.export` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.export`.
- **In Scope**: Invocation syntax, accepted parameters, output structure, error conditions.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`).

Export a session to a file in the specified format. Use this to save a conversation for sharing, archiving, or further processing.

**Parameters:** `session_id::`, `output::`, `format::`, `project::`, `scope::`, `path::`

`scope::` (default `local`) and `path::` (default cwd) narrow the source-session lookup used when `project::` is absent (see [Scope Configuration](../param_group/05_scope_configuration.md)) — `local` reproduces the original cwd-project-only lookup. An explicit `project::` makes both parameters a no-op (already fully scoped).

**Exit:** `0` success | `1` argument error (missing required params) | `2` storage read error or write error

**Syntax:**
```bash
claude_storage .export session_id::ID output::PATH
claude_storage .export session_id::ID output::PATH format::FORMAT
claude_storage .export session_id::ID output::PATH project::PROJECT
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `session_id::` | [`SessionId`](../type/09_session_id.md) | **required** | — | Session to export |
| `output::` | [`StoragePath`](../type/10_storage_path.md) | **required** | — | Output file path |
| `format::` | [`ExportFormat`](../type/03_export_format.md) | optional | `markdown` | Export format |
| `project::` | [`ProjectId`](../type/05_project_id.md) | optional | current dir | Source project |
| `scope::` | [`ScopeValue`](../type/07_scope_value.md) | optional | `local` | Project search boundary for source session lookup when `project::` is absent |
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | current dir | Anchor path for `scope::` resolution |

`session_id::` belongs to [Session Identification](../param_group/03_session_identification.md). `project::` belongs to [Project Scope](../param_group/02_project_scope.md). `scope::` and `path::` belong to the [Scope Configuration group](../param_group/05_scope_configuration.md) and narrow the source-session lookup as described above when `project::` is absent.

**Algorithm (4 steps):**
1. Validate required parameters — `session_id::` and `output::`; parse `format::` (default: `markdown`)
2. Load project — explicit `project::` (scope::/path:: ignored), or `scope::`-resolved projects (default `local`, reproducing the original cwd-only lookup) searched for `session_id::` when `project::` is absent
3. Find session by ID — prefix matching for partial UUIDs
4. Render session to format and write to output file — overwrites without warning

**Examples:**
```bash
# Export as markdown (default)
claude_storage .export session_id::-default_topic output::conversation.md

# Export as JSON for programmatic use
claude_storage .export session_id::abc123 format::json output::session.json

# Export as plain text
claude_storage .export session_id::-default_topic format::text output::transcript.txt

# Export a session from anywhere under an ancestor directory (scope::under)
claude_storage .export session_id::abc123 output::conversation.md scope::under path::/some/ancestor
```

**Notes:**
- Both `session_id::` and `output::` are required; command exits with `1` if either is missing
- Output file is overwritten without warning if it already exists
- `scope::`/`path::` only apply when `project::` is absent; `local` (the default) reproduces the original cwd-project-only lookup exactly
- A session file containing a malformed JSONL line no longer breaks `.export` (BUG-489): the corrupted line is skipped and the export proceeds using the rest of the session's valid entries and stats
- `session_id::` matches a leading prefix of the session ID, never a substring found elsewhere in the ID — a matching predicate shared with `.show`/`.search`/`.tail` that briefly matched substrings anywhere in the ID is fixed (BUG-490)

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [markdown](../format/01_markdown.md) | Default human-readable transcript output |
| 2 | [json](../format/02_json.md) | Machine-parseable structured export |
| 3 | [text](../format/03_text.md) | Plain text transcript output |

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 2 | [Project Scope](../param_group/02_project_scope.md) | Full | — |
| 3 | [Session Identification](../param_group/03_session_identification.md) | Full | — |
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Full | — |

### Referenced Parameters

| # | Parameter | Type | Required |
|---|-----------|------|----------|
| 5 | [`format::`](../param/05_format.md) | [`ExportFormat`](../type/03_export_format.md) | optional |
| 8 | [`output::`](../param/08_output.md) | [`StoragePath`](../type/10_storage_path.md) | **required** |
| 9 | [`path::`](../param/09_path.md) | [`StoragePath`](../type/10_storage_path.md) | optional |
| 10 | [`project::`](../param/10_project.md) | [`ProjectId`](../type/05_project_id.md) | optional |
| 12 | [`scope::`](../param/12_scope.md) | [`ScopeValue`](../type/07_scope_value.md) | optional |
| 14 | [`session_id::`](../param/14_session_id.md) | [`SessionId`](../type/09_session_id.md) | **required** |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 3 | [Export Session for Review](../user_story/003_export_session_for_review.md) | developer |
