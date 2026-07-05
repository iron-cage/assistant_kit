# EventType

**Status**: Planned | **Since**: 1.3.0

### Scope

- **Purpose**: Define the core data model shared by every journal event record.
- **Responsibility**: Documents the `EventType` enum, `EventRecord` struct, and `EventFields` field bag, plus their serialization behavior.
- **In Scope**: The 8 event-type discriminators, the flat optional-field bag, and JSON (de)serialization rules.
- **Out of Scope**: Writing events to disk (→ `docs/api/001_journal_writer.md`), reading/filtering events (→ `docs/api/002_journal_reader.md`).

## Description

Core data model for the journal system: the `EventType` enum, `EventRecord` struct, and `EventFields` field bag. `EventType` discriminates the 8 event categories. `EventRecord` is the top-level serializable unit containing version, timestamp, type, and fields. `EventFields` is a flat struct with all possible fields as `Option` — each event type populates its relevant subset.

## Interface

```rust
/// Discriminator for event categories.
#[derive( Debug, Clone, Copy, PartialEq, Eq )]
pub enum EventType
{
  Execution,
  Credential,
  GateWait,
  Retry,
  Timeout,
  RunnerRetry,
  ValidationRetry,
  Interactive,
}

impl EventType
{
  /// Serialize to the JSON `"type"` field value.
  pub fn as_str( self ) -> &'static str;

  /// Deserialize from a JSON `"type"` field value. Returns `None` for unknown types.
  pub fn from_str( s : &str ) -> Option< Self >;
}

/// Top-level journal event record — one per JSONL line.
#[derive( Debug, Clone )]
pub struct EventRecord
{
  pub v : u8,
  pub ts : String,
  pub event_type : EventType,
  pub fields : EventFields,
}

/// Flat field bag — each event type populates a relevant subset.
/// All fields are `Option` — absent fields are omitted from JSON output (not serialized as null).
#[derive( Debug, Clone, Default )]
pub struct EventFields
{
  pub command : Option< String >,
  pub message : Option< String >,
  pub dir : Option< String >,
  pub model : Option< String >,
  pub effort : Option< String >,
  pub timeout_secs : Option< u32 >,
  pub exit_code : Option< i32 >,
  pub duration_ms : Option< u64 >,
  pub error_class : Option< String >,
  pub error_kind : Option< String >,
  pub retries : Option< u32 >,
  pub retry_class_counts : Option< [ u32; 6 ] >,
  pub cost_usd : Option< f64 >,
  pub input_tokens : Option< u64 >,
  pub output_tokens : Option< u64 >,
  pub session_id : Option< String >,
  pub creds : Option< String >,
  pub stdout : Option< String >,
  pub stderr : Option< String >,
  pub max_sessions : Option< u32 >,
  pub wait_ms : Option< u64 >,
  pub gate_attempts : Option< u32 >,
  pub gate_outcome : Option< String >,
  pub attempt : Option< u32 >,
  pub limit : Option< u32 >,
  pub delay_secs : Option< u32 >,
  pub pattern : Option< String >,
  pub got : Option< String >,
  pub strategy : Option< String >,
  pub pid : Option< u32 >,
  pub output_style : Option< String >,
  pub output_format : Option< String >,
  pub runner_version : Option< String >,
  pub error_message : Option< String >,
}
```

## Behavioral Contract

- `EventType::as_str()` returns lowercase snake_case strings matching the JSON `"type"` values: `"execution"`, `"credential"`, `"gate_wait"`, `"retry"`, `"timeout"`, `"runner_retry"`, `"validation_retry"`, `"interactive"`
- `EventType::from_str()` is case-sensitive and returns `None` for unrecognized strings
- `EventFields::default()` returns all fields as `None`
- `EventRecord` serializes to a single JSON line (no embedded newlines in string fields — newlines in stdout/stderr are JSON-escaped as `\n`)
- The `retry_class_counts` array indices map to `[Transient, Account, Auth, Service, Process, Unknown]` matching `ErrorClass` variant order in `claude_runner`

## Sources

- `src/event.rs` — implementation
- `docs/feature/002_event_schema.md` — schema specification
- `claude_runner/src/cli/execution.rs` — ErrorClass enum (index mapping)
