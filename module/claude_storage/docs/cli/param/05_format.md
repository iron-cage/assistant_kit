# Parameter :: 5. `format::`

Export output format for `.export` operations.

**Type:** [`ExportFormat`](../type/03_export_format.md)

**Fundamental Type:** String enum wrapper

**Constraints:**
- Valid values: `markdown`, `json`, `text`
- Case-insensitive on input
- Error on invalid: `"format must be markdown|json|text, got {value}"`

**Default:** `markdown`

**Commands:** `.export`

**Purpose:** Selects the output serialization format. `markdown` produces a human-readable conversation document; `json` produces the raw JSONL entries as a JSON array suitable for programmatic processing; `text` produces a plain text transcript without markup.

**Examples:**
```bash
# Valid values
format::markdown   # Human-readable document (default)
format::json       # Raw entries as JSON array
format::text       # Plain text transcript

# Invalid values
format::html       # "format must be markdown|json|text, got html"
format::pdf        # "format must be markdown|json|text, got pdf"
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`ExportFormat`](../type/03_export_format.md) | String enum wrapper | String | `markdown`, `json`, or `text` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 6 | [`.export`](../command/06_export.md) | `markdown` | Selects export rendering format |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 3 | [Export Session for Review](../user_story/003_export_session_for_review.md) | developer |
