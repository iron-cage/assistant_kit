//! Test-only stand-in for the real `claude` binary's bidirectional control protocol.
//!
//! Built as a permanent `[[bin]]` Cargo target (not a shebang script) because
//! `find_claude_processes()` (`claude_core::process`) discovers live sessions by reading
//! `/proc/{pid}/cmdline` and matching `args[0]`'s basename against `"claude"` — the kernel's
//! binfmt_script handler rewrites a `/bin/sh`-shebang script's argv[0] to the interpreter path
//! at exec time, so a shell script named "claude" is invisible to that scan. A real compiled
//! ELF binary, symlinked to a temp-dir path named `claude`, preserves the basename and is
//! discoverable exactly like the real thing.
//!
//! Speaks the exact wire protocol confirmed against
//! `claude_runner_core/tests/fixtures/sdk_control_capture/` (task 415 Phase 0 capture) and
//! `claude_runner_core/src/control.rs`'s per-method request/response handling: reads
//! newline-delimited `control_request` envelopes from stdin, writes matching
//! `control_response` envelopes to stdout. Ignores all argv (the real `claude` binary's CLI
//! flags are irrelevant here — only the stdin/stdout protocol matters).

use std::io::{ BufRead, Write };

fn main()
{
  let stdin = std::io::stdin();
  let mut stdout = std::io::stdout();

  for line in stdin.lock().lines()
  {
    let Ok( line ) = line else { break };
    if line.trim().is_empty() { continue; }

    let Ok( value ) = serde_json::from_str::< serde_json::Value >( &line ) else { continue };

    if value.get( "type" ).and_then( serde_json::Value::as_str ) != Some( "control_request" )
    {
      // Non-control lines (e.g. streamInput's plain `{"type":"user",...}` turn) expect no
      // response at all — confirmed in control.rs's `stream_input()` doc comment.
      continue;
    }

    let Some( request_id ) = value.get( "request_id" ).and_then( serde_json::Value::as_str ) else { continue };
    let subtype = value.get( "request" )
      .and_then( |r| r.get( "subtype" ) )
      .and_then( serde_json::Value::as_str )
      .unwrap_or( "" );

    let payload = response_payload( subtype );

    let envelope = serde_json::json!(
    {
      "type" : "control_response",
      "response" :
      {
        "subtype" : "success",
        "request_id" : request_id,
        "response" : payload,
      },
    } );

    if writeln!( stdout, "{envelope}" ).is_err() { break; }
    if stdout.flush().is_err() { break; }
  }
}

/// Build the `response.response` payload for a given wire `subtype`, matching the exact
/// struct shapes `claude_runner_core::types` deserializes (field names, required-vs-optional,
/// camelCase-vs-snake_case per field — confirmed field-by-field against `types.rs` and the
/// real capture, not invented independently of them).
fn response_payload( subtype : &str ) -> serde_json::Value
{
  match subtype
  {
    // InitializeResult: top-level keys are snake_case (confirmed against the real capture's
    // wire_stdout.ndjson line 1); nested `account` uses camelCase (subscriptionType/apiProvider).
    "initialize" => serde_json::json!(
    {
      "commands" : [],
      "agents" : [],
      "output_style" : serde_json::Value::Null,
      "available_output_styles" : [],
      "models" : [],
      "account" : { "subscriptionType" : "test", "apiProvider" : "test" },
      "pid" : 0,
      "feedback_survey_config" : serde_json::Value::Null,
    } ),
    "rewind_files" => serde_json::json!( { "canRewind" : false, "error" : serde_json::Value::Null } ),
    "mcp_status" => serde_json::json!( { "mcpServers" : [] } ),
    "get_context_usage" => serde_json::json!( { "categories" : [] } ),
    "read_file" => serde_json::json!( { "contents" : "", "absPath" : "" } ),
    "reload_plugins" => serde_json::json!(
    {
      "commands" : [],
      "agents" : [],
      "plugins" : [],
      "mcpServers" : [],
      "error_count" : 0,
    } ),
    "reload_skills" => serde_json::json!( { "skills" : [] } ),
    "mcp_set_servers" => serde_json::json!( { "added" : [], "removed" : [], "errors" : {} } ),
    "background_tasks" => serde_json::Value::Bool( true ),
    // interrupt / set_permission_mode / set_mcp_permission_mode_override / set_model /
    // set_max_thinking_tokens / apply_flag_settings / seed_read_state / mcp_reconnect /
    // mcp_toggle / stop_task: response value is ignored by the caller — `null` is valid.
    _ => serde_json::Value::Null,
  }
}
