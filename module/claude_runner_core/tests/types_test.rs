//! Type definitions tests
//!
//! ## Purpose
//!
//! Verify all type enums have correct string conversions and defaults.
//!
//! ## Evidence
//!
//! - `ActionMode::as_str()` returns correct lowercase strings
//! - `LogLevel::as_str()` returns correct lowercase strings
//! - `ActionMode::default()` returns `Ask` (security)
//! - `LogLevel::default()` returns `Info`
//! - `OutputFormat::StreamJson.as_str()` returns `"stream-json"` with HYPHEN (not underscore)
//! - `InputFormat::StreamJson.as_str()` returns `"stream-json"` with HYPHEN (not underscore)
//! - `PermissionMode::AcceptEdits.as_str()` returns `"acceptEdits"` (camelCase)
//! - `PermissionMode::BypassPermissions.as_str()` returns `"bypassPermissions"` (camelCase)
//! - `EffortLevel::Max.as_str()` returns `"max"` (NOT `"maximum"`)
//!
//! ## Test Coverage Matrix
//!
//! | Enum | as_str | default | ordering |
//! |------|--------|---------|----------|
//! | ActionMode | ✅ | ✅ | — |
//! | LogLevel | ✅ | ✅ | ✅ |
//! | OutputFormat | ✅ | ✅ | — |
//! | InputFormat | ✅ | ✅ | — |
//! | PermissionMode | ✅ | ✅ | — |
//! | EffortLevel | ✅ | ✅ | — |

use claude_runner_core::{ ActionMode, EffortLevel, InputFormat, LogLevel, OutputFormat, PermissionMode };

#[test]
fn action_mode_as_str_conversions() {
  assert_eq!( ActionMode::Ask.as_str(), "ask" );
  assert_eq!( ActionMode::Allow.as_str(), "allow" );
  assert_eq!( ActionMode::Deny.as_str(), "deny" );
}

#[test]
fn action_mode_default_is_ask() {
  // Fix(issue-action-mode-default): Default must be Ask for security
  assert_eq!( ActionMode::default(), ActionMode::Ask );
}

#[test]
fn log_level_as_str_conversions() {
  assert_eq!( LogLevel::Error.as_str(), "error" );
  assert_eq!( LogLevel::Warn.as_str(), "warn" );
  assert_eq!( LogLevel::Info.as_str(), "info" );
  assert_eq!( LogLevel::Debug.as_str(), "debug" );
  assert_eq!( LogLevel::Trace.as_str(), "trace" );
}

#[test]
fn log_level_default_is_info() {
  assert_eq!( LogLevel::default(), LogLevel::Info );
}

#[test]
fn log_level_ordering() {
  // Verify LogLevel has correct ordering (Error < Warn < Info < Debug < Trace)
  assert!( LogLevel::Error < LogLevel::Warn );
  assert!( LogLevel::Warn < LogLevel::Info );
  assert!( LogLevel::Info < LogLevel::Debug );
  assert!( LogLevel::Debug < LogLevel::Trace );
}

// OutputFormat tests

#[test]
fn output_format_as_str_conversions() {
  assert_eq!( OutputFormat::Text.as_str(), "text" );
  assert_eq!( OutputFormat::Json.as_str(), "json" );
  assert_eq!( OutputFormat::StreamJson.as_str(), "stream-json" );
}

#[test]
fn output_format_stream_json_has_hyphen() {
  // Fix(issue-output-format-stream-json-hyphen): StreamJson maps to "stream-json" with a hyphen
  // Root cause: Claude CLI uses "stream-json" (hyphen) not "stream_json" (underscore)
  // Pitfall: Do not use underscore — "stream_json" is not a valid claude CLI value
  let s = OutputFormat::StreamJson.as_str();
  assert!( s.contains( '-' ), "stream-json must use hyphen, got: {s}" );
  assert!( !s.contains( '_' ), "stream-json must NOT use underscore, got: {s}" );
}

#[test]
fn output_format_default_is_text() {
  assert_eq!( OutputFormat::default(), OutputFormat::Text );
}

// InputFormat tests

#[test]
fn input_format_as_str_conversions() {
  assert_eq!( InputFormat::Text.as_str(), "text" );
  assert_eq!( InputFormat::StreamJson.as_str(), "stream-json" );
}

#[test]
fn input_format_stream_json_has_hyphen() {
  // Fix(issue-input-format-stream-json-hyphen): StreamJson maps to "stream-json" with a hyphen
  // Root cause: Claude CLI uses "stream-json" (hyphen) not "stream_json" (underscore)
  // Pitfall: Do not use underscore — "stream_json" is not a valid claude CLI value
  let s = InputFormat::StreamJson.as_str();
  assert!( s.contains( '-' ), "stream-json must use hyphen, got: {s}" );
  assert!( !s.contains( '_' ), "stream-json must NOT use underscore, got: {s}" );
}

#[test]
fn input_format_default_is_text() {
  assert_eq!( InputFormat::default(), InputFormat::Text );
}

// PermissionMode tests

#[test]
fn permission_mode_as_str_conversions() {
  assert_eq!( PermissionMode::Default.as_str(), "default" );
  assert_eq!( PermissionMode::AcceptEdits.as_str(), "acceptEdits" );
  assert_eq!( PermissionMode::BypassPermissions.as_str(), "bypassPermissions" );
}

#[test]
fn permission_mode_accept_edits_is_camel_case() {
  // Fix(issue-permission-mode-camelcase): AcceptEdits uses camelCase "acceptEdits"
  // Root cause: Claude CLI uses camelCase for multi-word permission mode strings
  // Pitfall: Do not use lowercase — "acceptedits" is not a valid claude CLI value
  assert_eq!( PermissionMode::AcceptEdits.as_str(), "acceptEdits" );
}

#[test]
fn permission_mode_bypass_permissions_is_camel_case() {
  // Fix(issue-permission-mode-camelcase): BypassPermissions uses camelCase "bypassPermissions"
  // Root cause: Claude CLI uses camelCase for multi-word permission mode strings
  // Pitfall: Do not use lowercase — "bypasspermissions" is not a valid claude CLI value
  assert_eq!( PermissionMode::BypassPermissions.as_str(), "bypassPermissions" );
}

#[test]
fn permission_mode_default_is_default() {
  assert_eq!( PermissionMode::default(), PermissionMode::Default );
}

// EffortLevel tests

#[test]
fn effort_level_as_str_conversions() {
  assert_eq!( EffortLevel::Low.as_str(), "low" );
  assert_eq!( EffortLevel::Medium.as_str(), "medium" );
  assert_eq!( EffortLevel::High.as_str(), "high" );
  assert_eq!( EffortLevel::Max.as_str(), "max" );
}

#[test]
fn effort_level_max_is_max_not_maximum() {
  // Max maps to "max", not "maximum" — matches Claude CLI accepted values
  assert_eq!( EffortLevel::Max.as_str(), "max" );
  assert_ne!( EffortLevel::Max.as_str(), "maximum" );
}

#[test]
fn effort_level_default_is_medium() {
  assert_eq!( EffortLevel::default(), EffortLevel::Medium );
}
