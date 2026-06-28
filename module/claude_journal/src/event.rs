//! Event types and serializable record structs for the journal system.

use serde::{ Deserialize, Serialize };

/// Discriminator for event categories emitted by `clr`.
///
/// Each variant corresponds to a specific execution lifecycle moment.
/// Serializes as lowercase `snake_case` strings in JSON (e.g. `"gate_wait"`).
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize ) ]
#[ serde( rename_all = "snake_case" ) ]
pub enum EventType
{
  /// Print-mode `run`/`ask` subprocess completed.
  Execution,
  /// `isolated` or `refresh` credential subprocess completed.
  Credential,
  /// Concurrency gate activated — runner blocked waiting for a session slot.
  GateWait,
  /// An error-class retry attempt fired inside the retry loop.
  Retry,
  /// Watchdog timer killed the subprocess.
  Timeout,
  /// A spawn-failure runner retry attempt fired.
  RunnerRetry,
  /// An expect-validation retry attempt fired.
  ValidationRetry,
  /// An interactive session ended.
  Interactive,
}

impl EventType
{
  /// Return the JSON `"type"` field string for this variant.
  #[ inline ]
  #[ must_use ]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::Execution       => "execution",
      Self::Credential      => "credential",
      Self::GateWait        => "gate_wait",
      Self::Retry           => "retry",
      Self::Timeout         => "timeout",
      Self::RunnerRetry     => "runner_retry",
      Self::ValidationRetry => "validation_retry",
      Self::Interactive     => "interactive",
    }
  }

  /// Parse a `"type"` JSON field value into an `EventType`.
  ///
  /// Case-sensitive. Returns `None` for unrecognized strings (forward compat).
  #[ inline ]
  #[ must_use ]
  pub fn parse( s : &str ) -> Option< Self >
  {
    match s
    {
      "execution"        => Some( Self::Execution ),
      "credential"       => Some( Self::Credential ),
      "gate_wait"        => Some( Self::GateWait ),
      "retry"            => Some( Self::Retry ),
      "timeout"          => Some( Self::Timeout ),
      "runner_retry"     => Some( Self::RunnerRetry ),
      "validation_retry" => Some( Self::ValidationRetry ),
      "interactive"      => Some( Self::Interactive ),
      _                  => None,
    }
  }
}

/// Top-level journal event record — one per JSONL line.
///
/// Serializes as a single flat JSON object: `v`, `ts`, `type`, then all
/// non-None fields from `fields` inlined at the top level.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct EventRecord
{
  /// Schema version. Always `1` for events emitted by this crate version.
  pub v          : u8,
  /// UTC timestamp, ISO 8601 with millisecond precision (`2026-06-27T14:30:00.123Z`).
  pub ts         : String,
  /// Event category discriminator.
  #[ serde( rename = "type" ) ]
  pub event_type : EventType,
  /// Flat field bag — each event type populates a relevant subset.
  #[ serde( flatten ) ]
  pub fields     : EventFields,
}

impl EventRecord
{
  /// Construct a minimal event record with the current UTC timestamp and an empty field bag.
  #[ inline ]
  #[ must_use ]
  pub fn new( event_type : EventType ) -> Self
  {
    Self
    {
      v          : 1,
      ts         : chrono::Utc::now().format( "%Y-%m-%dT%H:%M:%S%.3fZ" ).to_string(),
      event_type,
      fields     : EventFields::default(),
    }
  }
}

/// Flat field bag for `EventRecord` — each event type populates a relevant subset.
///
/// All fields are `Option`; absent fields are omitted from JSON serialization.
/// Use [`EventFields::default()`] to start with all fields absent.
#[ allow( clippy::struct_excessive_bools ) ]
#[ derive( Debug, Clone, Default, Serialize, Deserialize ) ]
pub struct EventFields
{
  /// Subcommand that produced this event (`"run"`, `"ask"`, `"isolated"`, `"refresh"`).
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub command           : Option< String >,
  /// User prompt message (may be truncated for long prompts).
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub message           : Option< String >,
  /// Working directory (`--dir` effective value).
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub dir               : Option< String >,
  /// Claude model used (`--model` effective value).
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub model             : Option< String >,
  /// Reasoning effort level (`--effort` effective value).
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub effort            : Option< String >,
  /// Watchdog timeout in seconds (`--timeout` effective value; `0` = unlimited).
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub timeout_secs      : Option< u32 >,
  /// Subprocess exit code.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub exit_code         : Option< i32 >,
  /// Wall-clock duration from subprocess start to exit, in milliseconds.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub duration_ms       : Option< u64 >,
  /// Error class name for retry events (`"Transient"`, `"Account"`, etc.).
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub error_class       : Option< String >,
  /// Error kind detail string from `ErrorKind::as_str()`.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub error_kind        : Option< String >,
  /// Number of retries attempted before this result.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub retries           : Option< u32 >,
  /// Per-class retry counts `[Transient, Account, Auth, Service, Process, Unknown]`.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub retry_class_counts : Option< [ u32; 6 ] >,
  /// Cost in USD from the CLR envelope `cost_usd` field.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub cost_usd          : Option< f64 >,
  /// Input token count from the CLR envelope.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub input_tokens      : Option< u64 >,
  /// Output token count from the CLR envelope.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub output_tokens     : Option< u64 >,
  /// Session ID from the CLR envelope `session_id` field.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub session_id        : Option< String >,
  /// Credentials file path (`--creds` effective value).
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub creds             : Option< String >,
  /// Captured subprocess stdout (truncated to 1 MiB in `full` journal level; absent in `meta`).
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub stdout            : Option< String >,
  /// Captured subprocess stderr (truncated to 1 MiB in `full` journal level; absent in `meta`).
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub stderr            : Option< String >,
  /// Max concurrent sessions (`--max-sessions` effective value).
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub max_sessions      : Option< u32 >,
  /// Gate wait duration in milliseconds.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub wait_ms           : Option< u64 >,
  /// Number of gate polling attempts before a slot was acquired.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub gate_attempts     : Option< u32 >,
  /// Gate outcome: `"acquired"` or `"abandoned"`.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub gate_outcome      : Option< String >,
  /// Retry attempt number (1-based).
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub attempt           : Option< u32 >,
  /// Retry limit for this error class.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub limit             : Option< u32 >,
  /// Retry delay in seconds.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub delay_secs        : Option< u32 >,
  /// Expect pattern string (`--expect` value).
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub pattern           : Option< String >,
  /// Actual stdout value that did not match the expect pattern.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub got               : Option< String >,
  /// Expect mismatch strategy (`"fail"`, `"retry"`, `"default:…"`).
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub strategy          : Option< String >,
  /// PID of the subprocess killed by the watchdog.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub pid               : Option< u32 >,
  /// Output style (`"summary"` or `"raw"`).
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub output_style      : Option< String >,
  /// Output format (`"text"`, `"json"`, `"stream-json"`).
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub output_format     : Option< String >,
  /// `claude_runner` crate version that emitted this event.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub runner_version    : Option< String >,
  /// Human-readable error message for runner-retry events.
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub error_message     : Option< String >,
}
