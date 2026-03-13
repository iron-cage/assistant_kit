# Types

| # | Type | Fundamental | Parameters | Commands |
|---|------|-------------|------------|----------|
| 1 | `AccountName` | `String` (newtype) | [`name::`](params.md#parameter--1-name) | 3 |
| 2 | `VerbosityLevel` | `u8` (newtype) | [`verbosity::`](params.md#parameter--2-verbosity--v) | 3 |
| 3 | `OutputFormat` | `enum` | [`format::`](params.md#parameter--3-format) | 3 |
| 4 | `WarningThreshold` | `u64` (newtype) | [`threshold::`](params.md#parameter--4-threshold) | 1 |

**Total:** 4 types

---

### Type :: 1. `AccountName`

**Purpose:** Identifies a named credential profile in the account store. Enforces filesystem-safe naming to guarantee safe file creation under `~/.claude/accounts/`.

**Fundamental Type:** Newtype wrapping `String`

```rust
pub struct AccountName( String );
```

**Constants:**
- No predefined constants — all valid names are user-defined

**Constraints:**
- Non-empty (reject `""`)
- No filesystem-forbidden characters: `/`, `\`, `:`, `*`, `?`, `"`, `<`, `>`, `|`, null byte
- Maps to file `~/.claude/accounts/{name}.credentials.json`

**Parsing:**

```rust
impl AccountName
{
  pub fn new( s : &str ) -> Result< Self, String >
  {
    if s.is_empty() { return Err( "account name must not be empty".into() ); }
    if s.chars().any( | c | matches!( c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' ) )
    {
      return Err( format!( "account name '{}' contains invalid characters", s ) );
    }
    Ok( Self( s.to_string() ) )
  }
}
```

**Methods:**
- `get() -> &str` — raw string accessor
- `to_credential_path( accounts_dir : &Path ) -> PathBuf` — resolves `{accounts_dir}/{name}.credentials.json`

**Commands:** [`.account.save`](commands.md#command--4-accountsave), [`.account.switch`](commands.md#command--5-accountswitch), [`.account.delete`](commands.md#command--6-accountdelete)

---

### Type :: 2. `VerbosityLevel`

**Purpose:** Controls output detail density. Enables scripts to suppress labels (`v::0`) and debuggers to surface full metadata (`v::2`).

**Fundamental Type:** Newtype wrapping `u8`

```rust
pub struct VerbosityLevel( u8 );
```

**Constants:**
- `QUIET = 0` — bare values only (machine-friendly)
- `NORMAL = 1` — labeled output with human context (default)
- `VERBOSE = 2` — full metadata including subscription type, tier, raw timestamps
- `DEFAULT = 1`
- `MIN = 0`
- `MAX = 2`

**Constraints:**
- Range: 0-2 inclusive
- Values outside range rejected with exit 1

**Parsing:**

```rust
impl VerbosityLevel
{
  pub fn new( n : u8 ) -> Result< Self, String >
  {
    if n > 2 { return Err( format!( "verbosity must be 0-2, got {}", n ) ); }
    Ok( Self( n ) )
  }
}
```

**Methods:**
- `get() -> u8` — raw numeric value
- `is_quiet() -> bool` — true when level is 0
- `is_verbose() -> bool` — true when level is 2
- `includes_labels() -> bool` — true when level >= 1

**Commands:** [`.account.list`](commands.md#command--3-accountlist), [`.token.status`](commands.md#command--7-tokenstatus), [`.paths`](commands.md#command--8-paths)

---

### Type :: 3. `OutputFormat`

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

**Commands:** [`.account.list`](commands.md#command--3-accountlist), [`.token.status`](commands.md#command--7-tokenstatus), [`.paths`](commands.md#command--8-paths)

---

### Type :: 4. `WarningThreshold`

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
