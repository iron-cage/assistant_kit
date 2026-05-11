# Types

### All Types (3 total)

| # | Type | Fundamental | Parameters | Commands |
|---|------|-------------|------------|----------|
| 1 | `AccountName` | `String` (newtype) | [`name::`](params.md#parameter--1-name) | 5 cmds |
| 2 | `OutputFormat` | `enum` | [`format::`](params.md#parameter--2-format) | 6 cmds |
| 3 | `WarningThreshold` | `u64` (newtype) | [`threshold::`](params.md#parameter--3-threshold) | 1 cmd |

**Total:** 3 types

---

### Type :: 1. `AccountName`

**Purpose:** Identifies a credential profile in the account store using the account's email address as the unique key. Enforces email format to guarantee correct identification and safe file creation under the credential store.

**Fundamental Type:** Newtype wrapping `String`

```rust
pub struct AccountName( String );
```

**Constants:**
- No predefined constants — all valid names are user-defined

**Constraints:**
- Non-empty (reject `""`)
- Must contain `@` with non-empty local part and domain (valid email format)
- Local part (before `@`) must not contain `/`, `\`, or `*` (path-unsafe chars rejected before any filesystem operation)
- Maps to file `{credential_store}/{email}.credentials.json`

**Parsing:**

```rust
impl AccountName
{
  pub fn new( s : &str ) -> Result< Self, String >
  {
    if s.is_empty() { return Err( "account name must not be empty".into() ); }
    let at = s.find( '@' ).ok_or_else( || format!( "account name '{}' must be an email address", s ) )?;
    if at == 0 || at == s.len() - 1
    {
      return Err( format!( "account name '{}' must be an email address", s ) );
    }
    Ok( Self( s.to_string() ) )
  }
}
```

**Methods:**
- `get() -> &str` — raw string accessor
- `to_credential_path( credential_store : &Path ) -> PathBuf` — resolves `{credential_store}/{name}.credentials.json`

**Commands:** [`.accounts`](commands.md#command--3-accounts) *(optional)*, [`.account.save`](commands.md#command--4-accountsave), [`.account.use`](commands.md#command--5-accountuse), [`.account.delete`](commands.md#command--6-accountdelete), [`.account.limits`](commands.md#command--11-accountlimits) *(optional)*

---

### Type :: 2. `OutputFormat`

**Purpose:** Selects between human-readable text and machine-parseable JSON output. Enables pipeline composition via `format::json | jq`.

**Fundamental Type:** Enum

```rust
pub enum OutputFormat
{
  Text,
  Json,
}
```

**Constants:**
- `TEXT` — human-readable labeled output (default)
- `JSON` — structured JSON output
- `DEFAULT = Text`

**Constraints:**
- Exactly one of: `text`, `json` (case-insensitive)
- Unknown values rejected with exit 1

**Parsing:**

```rust
impl OutputFormat
{
  pub fn new( s : &str ) -> Result< Self, String >
  {
    match s.to_lowercase().as_str()
    {
      "text" => Ok( Self::Text ),
      "json" => Ok( Self::Json ),
      other => Err( format!( "invalid format '{}' — expected 'text' or 'json'", other ) ),
    }
  }
}
```

**Methods:**
- `get() -> &str` — string representation (`"text"` or `"json"`)
- `is_json() -> bool` — true for JSON format
- `is_text() -> bool` — true for text format

**Commands:** [`.accounts`](commands.md#command--3-accounts), [`.token.status`](commands.md#command--7-tokenstatus), [`.paths`](commands.md#command--8-paths), [`.usage`](commands.md#command--9-usage), [`.credentials.status`](commands.md#command--10-credentialsstatus), [`.account.limits`](commands.md#command--11-accountlimits)

---

### Type :: 3. `WarningThreshold`

**Purpose:** Configures the boundary between `Valid` and `ExpiringSoon` token classification. Allows callers to tune early-warning sensitivity for automation or interactive use.

**Fundamental Type:** Newtype wrapping `u64` (seconds)

```rust
pub struct WarningThreshold( u64 );
```

**Constants:**
- `DEFAULT = 3600` — 60 minutes (matches `token::WARNING_THRESHOLD_SECS`)
- `MIN = 0` — never classify as ExpiringSoon

**Constraints:**
- Non-negative integer (unsigned, so always non-negative)
- No upper bound (any u64 value accepted)

**Parsing:**

```rust
impl WarningThreshold
{
  pub fn new( s : &str ) -> Result< Self, String >
  {
    let secs : u64 = s.parse()
      .map_err( | _ | format!( "invalid threshold '{}' — expected seconds as integer", s ) )?;
    Ok( Self( secs ) )
  }
}
```

**Methods:**
- `get() -> u64` — raw seconds value
- `as_duration() -> Duration` — converts to `std::time::Duration`
- `is_disabled() -> bool` — true when threshold is 0

**Commands:** [`.token.status`](commands.md#command--7-tokenstatus)
