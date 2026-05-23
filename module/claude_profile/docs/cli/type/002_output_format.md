# Type :: 2. `OutputFormat`

**Purpose:** Selects between human-readable text, compact table, and machine-parseable JSON output. Enables pipeline composition via `format::json | jq`; enables at-a-glance multi-account comparison via `format::table`.

**Fundamental Type:** Enum

```rust
pub enum OutputFormat
{
  Text,
  Json,
  Table,
}
```

**Constants:**
- `TEXT` — human-readable labeled output (default)
- `JSON` — structured JSON output
- `TABLE` — compact aligned table (`.accounts` only)
- `DEFAULT = Text`

**Constraints:**
- One of: `text`, `json`, `table` (case-insensitive)
- `table` is accepted only by `.accounts`; other commands reject it with exit 1
- Unknown values rejected with exit 1

**Parsing:**

```rust
impl OutputFormat
{
  pub fn new( s : &str ) -> Result< Self, String >
  {
    match s.to_lowercase().as_str()
    {
      "text"  => Ok( Self::Text ),
      "json"  => Ok( Self::Json ),
      "table" => Ok( Self::Table ),
      other   => Err( format!( "unknown format '{}': expected text, json, or table", other ) ),
    }
  }
}
```

**Methods:**
- `get() -> &str` — string representation (`"text"`, `"json"`, or `"table"`)
- `is_json() -> bool` — true for JSON format
- `is_text() -> bool` — true for text format
- `is_table() -> bool` — true for table format

**Parameters:** [`format::`](../param/002_format.md)

**Commands:** [`.accounts`](../command/001_account.md#command--3-accounts), [`.token.status`](../command/005_token.md#command--7-tokenstatus), [`.paths`](../command/004_paths.md#command--8-paths), [`.usage`](../command/006_usage.md#command--9-usage), [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus), [`.account.limits`](../command/001_account.md#command--11-accountlimits)
