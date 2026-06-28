# Type :: 3. `ExportFormat`

### Scope

- **Purpose**: Specify the `ExportFormat` semantic type.
- **Responsibility**: Validation rules, fundamental type, and parameter mapping for `ExportFormat`.
- **In Scope**: Parsing rules, valid/invalid values, error messages.
- **Out of Scope**: Parameter usage (→ `param/`), command context (→ `command/`).

**Purpose:** Output serialization format for session export. Determines the structure and encoding of the exported file.

**Fundamental Type:** Wrapper around string enum

**Constants:**
- MARKDOWN = `"markdown"` (default — human-readable)
- JSON = `"json"` (raw entries)
- TEXT = `"text"` (plain transcript)
- DEFAULT = MARKDOWN

**Constraints:**
- Valid values: `markdown`, `json`, `text`
- Case-insensitive on parse
- Error on invalid: `"format must be markdown|json|text, got {value}"`

**Parsing:**
```
Parse string to enum variant (case-insensitive):
  Input: "markdown" → ExportFormat::Markdown
  Input: "json" → ExportFormat::Json
  Input: "text" → ExportFormat::Text
  Error: "format must be markdown|json|text, got {value}"
```

**Methods:**
- `get() -> string` — Returns canonical lowercase variant name
- `is_default() -> boolean` — True when format is Markdown
- `file_extension() -> string` — Returns `"md"`, `"json"`, or `"txt"`

**Commands:** `.export`

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 6 | [`.export`](../command/06_export.md) | `format::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 5 | [`format::`](../param/05_format.md) | 1 |
