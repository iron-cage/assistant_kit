//! `summary` output-format rendering.
//!
//! Parses the CLR result envelope emitted by `claude --output-format json`
//! and renders a key:val header followed by a `---` separator and the `result`
//! field value as the text body.
//!
//! Called from `execution::run_print_mode()` when `output_style == "summary"` and
//! claude exits 0.  On parse failure the caller falls back to raw output.

use core::fmt::Write as _;

const CYAN   : &str = "\x1b[36m";
const GREEN  : &str = "\x1b[32m";
const YELLOW : &str = "\x1b[33m";
const DIM    : &str = "\x1b[2m";
const RESET  : &str = "\x1b[0m";

// ── Minimal JSON extraction ────────────────────────────────────────────────────

/// Extract a JSON string value for `key`.  Returns `None` for `null` or absent keys.
/// JSON escape sequences (`\n`, `\t`, `\\`, `\"`, `\/`, `\r`) are unescaped.
fn extract_str( s : &str, key : &str ) -> Option< String >
{
  let needle = format!( "\"{key}\":" );
  let pos    = s.find( &needle )?;
  let rest   = s[ pos + needle.len() .. ].trim_start_matches( ' ' );
  if rest.starts_with( "null" ) { return None; }
  if !rest.starts_with( '"' )   { return None; }
  let inner  = &rest[ 1 .. ];
  let mut out    = String::new();
  let mut escape = false;
  for c in inner.chars()
  {
    if escape
    {
      match c
      {
        'n'  => out.push( '\n' ),
        't'  => out.push( '\t' ),
        'r'  => out.push( '\r' ),
        '"'  => out.push( '"' ),
        '\\' => out.push( '\\' ),
        '/'  => out.push( '/' ),
        _    => { out.push( '\\' ); out.push( c ); }
      }
      escape = false;
      continue;
    }
    if c == '\\' { escape = true; continue; }
    if c == '"'  { return Some( out ); }
    out.push( c );
  }
  Some( out )
}

/// Extract a `u64` JSON number for `key`.
fn extract_u64( s : &str, key : &str ) -> Option< u64 >
{
  let needle = format!( "\"{key}\":" );
  let pos    = s.find( &needle )?;
  let rest   = s[ pos + needle.len() .. ].trim_start_matches( ' ' );
  let end    = rest.find( |c : char| !c.is_ascii_digit() ).unwrap_or( rest.len() );
  rest[ ..end ].parse().ok()
}

/// Extract an `f64` JSON number for `key`.
fn extract_f64( s : &str, key : &str ) -> Option< f64 >
{
  let needle = format!( "\"{key}\":" );
  let pos    = s.find( &needle )?;
  let rest   = s[ pos + needle.len() .. ].trim_start_matches( ' ' );
  let end    = rest
    .find( |c : char| !matches!( c, '0'..='9' | '.' | '-' | 'e' | 'E' | '+' ) )
    .unwrap_or( rest.len() );
  rest[ ..end ].parse().ok()
}

/// Extract a JSON boolean value for `key`.
fn extract_bool( s : &str, key : &str ) -> Option< bool >
{
  let needle = format!( "\"{key}\":" );
  let pos    = s.find( &needle )?;
  let rest   = s[ pos + needle.len() .. ].trim_start_matches( ' ' );
  if rest.starts_with( "true" )  { return Some( true ); }
  if rest.starts_with( "false" ) { return Some( false ); }
  None
}

/// Count elements in the `permission_denials` JSON array.
fn count_permission_denials( json : &str ) -> u64
{
  let needle = "\"permission_denials\":[";
  let Some( pos ) = json.find( needle ) else { return 0 };
  let rest  = &json[ pos + needle.len() .. ];
  let Some( end ) = rest.find( ']' ) else { return 0 };
  let inner = rest[ ..end ].trim();
  if inner.is_empty() { return 0; }
  ( inner.matches( "},{" ).count() + 1 ) as u64
}

/// Extract the `"result"` field from a CLR JSON envelope.
///
/// Returns `Some(text)` when the envelope contains a non-null `"result"` string,
/// `None` otherwise.  Used by `first_message()` to avoid showing raw JSON blobs
/// in retry diagnostic messages.
pub( super ) fn extract_result_text( json : &str ) -> Option< String >
{
  extract_str( json, "result" )
}

// ── Field constants ──────────────────────────────────────────────────────────────

/// All 32 renderable CLR envelope fields in canonical order.
const FIELD_ORDER : &[ &str ] = &[
  "type", "subtype", "session_id", "uuid", "is_error", "stop_reason",
  "num_turns", "fast_mode_state", "duration_ms", "duration_api_ms",
  "input_tokens", "output_tokens", "cache_creation_input_tokens",
  "cache_read_input_tokens", "total_cost_usd", "service_tier", "speed",
  "inference_geo", "web_search_requests", "web_fetch_requests",
  "cache_ephemeral_1h_input_tokens", "cache_ephemeral_5m_input_tokens",
  "model", "model_input_tokens", "model_output_tokens",
  "model_cache_read_input_tokens", "model_cache_creation_input_tokens",
  "model_web_search_requests", "model_cost_usd", "model_context_window",
  "model_max_output_tokens", "permission_denials",
];

/// `minimal` profile: 7 fields (v1.2.0 backward-compatible rendering).
const PROFILE_MINIMAL : &[ &str ] = &[
  "type", "subtype", "session_id", "is_error",
  "input_tokens", "output_tokens", "total_cost_usd",
];

/// `standard` profile: 14 key operational fields.
const PROFILE_STANDARD : &[ &str ] = &[
  "type", "subtype", "session_id", "is_error", "stop_reason", "num_turns",
  "duration_ms", "input_tokens", "output_tokens",
  "cache_creation_input_tokens", "cache_read_input_tokens",
  "total_cost_usd", "service_tier", "model",
];

/// Resolve a `--summary-fields` value to a validated field list.
///
/// Returns `Ok(fields)` for valid profiles (`full`, `standard`, `minimal`) or
/// valid comma-separated custom whitelists.  Returns `Err(bad_token)` on invalid input.
///
/// # Errors
///
/// Returns `Err(token)` when `value` is not a recognized profile name and
/// contains at least one field name absent from `FIELD_ORDER`.
#[ inline ]
pub fn resolve_fields( value : &str ) -> Result< Vec< &'static str >, String >
{
  match value
  {
    "full"     => return Ok( FIELD_ORDER.to_vec() ),
    "minimal"  => return Ok( PROFILE_MINIMAL.to_vec() ),
    "standard" => return Ok( PROFILE_STANDARD.to_vec() ),
    _          => {}
  }
  let mut fields = Vec::new();
  for token in value.split( ',' )
  {
    let token = token.trim();
    if let Some( &field ) = FIELD_ORDER.iter().find( |&&f| f == token )
    {
      if !fields.contains( &field ) { fields.push( field ); }
    }
    else
    {
      return Err( token.to_string() );
    }
  }
  if fields.is_empty()
  {
    return Err( value.to_string() );
  }
  Ok( fields )
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// Render `summary` format output from a successful `--output-format json` CLR response.
///
/// Returns `Some(rendered)` on success, `None` when the JSON cannot be parsed as a
/// CLR result envelope (caller should fall back to printing raw `json`).
///
/// `fields` is the raw `--summary-fields` value (`None` defaults to `"full"`).
#[ inline ]
#[ must_use ]
#[ allow( clippy::too_many_lines, clippy::similar_names ) ]
pub fn render_summary( json : &str, fields : Option< &str > ) -> Option< String >
{
  let selected = resolve_fields( fields.unwrap_or( "full" ) ).ok()?;
  let has      = |f : &str| selected.contains( &f );

  // Fix(BUG-310): gate on invariant field, not optional session_id.
  // Root cause: extract_str(json,"session_id")? returned None for 7-field envelopes
  //   where session_id is absent — restoring BUG-309 raw-JSON symptom for those versions.
  // Pitfall: any ? on an optional CLR field silently breaks all envelopes missing that field.
  let msg_type     = extract_str( json, "type" )?;
  if msg_type != "result" { return None; }
  let session_id   = extract_str( json, "session_id" ).unwrap_or_default();
  let subtype      = extract_str( json, "subtype" ).unwrap_or_default();
  let is_error     = extract_bool( json, "is_error" ).unwrap_or( false );
  let result       = extract_str( json, "result" ).unwrap_or_default();

  // Top-level scalars
  let uuid         = extract_str( json, "uuid" ).unwrap_or_default();
  let stop_reason  = extract_str( json, "stop_reason" ).unwrap_or_default();
  let num_turns    = extract_u64( json, "num_turns" ).unwrap_or( 0 );
  let fast_mode    = extract_str( json, "fast_mode_state" ).unwrap_or_default();
  let duration_ms  = extract_u64( json, "duration_ms" ).unwrap_or( 0 );
  let duration_api = extract_u64( json, "duration_api_ms" ).unwrap_or( 0 );
  let cost         = extract_f64( json, "total_cost_usd" ).unwrap_or( 0.0 );

  // usage nested object
  let usage_marker = "\"usage\":{";
  let usage_str    = json.find( usage_marker ).map( |p| &json[ p + usage_marker.len() .. ] );
  let in_tok       = usage_str.and_then( |s| extract_u64( s, "input_tokens" ) ).unwrap_or( 0 );
  let out_tok      = usage_str.and_then( |s| extract_u64( s, "output_tokens" ) ).unwrap_or( 0 );
  let cache_create = usage_str.and_then( |s| extract_u64( s, "cache_creation_input_tokens" ) ).unwrap_or( 0 );
  let cache_read   = usage_str.and_then( |s| extract_u64( s, "cache_read_input_tokens" ) ).unwrap_or( 0 );
  let svc_tier     = usage_str.and_then( |s| extract_str( s, "service_tier" ) ).unwrap_or_default();
  let spd          = usage_str.and_then( |s| extract_str( s, "speed" ) ).unwrap_or_default();
  let inf_geo      = usage_str.and_then( |s| extract_str( s, "inference_geo" ) ).unwrap_or_default();

  // usage.server_tool_use
  let stu_marker = "\"server_tool_use\":{";
  let stu_str    = usage_str.and_then( |s| s.find( stu_marker ).map( |p| &s[ p + stu_marker.len() .. ] ) );
  let web_search = stu_str.and_then( |s| extract_u64( s, "web_search_requests" ) ).unwrap_or( 0 );
  let web_fetch  = stu_str.and_then( |s| extract_u64( s, "web_fetch_requests" ) ).unwrap_or( 0 );

  // usage.cache_creation
  let cc_marker = "\"cache_creation\":{";
  let cc_str    = usage_str.and_then( |s| s.find( cc_marker ).map( |p| &s[ p + cc_marker.len() .. ] ) );
  let eph_1h    = cc_str.and_then( |s| extract_u64( s, "ephemeral_1h_input_tokens" ) ).unwrap_or( 0 );
  let eph_5m    = cc_str.and_then( |s| extract_u64( s, "ephemeral_5m_input_tokens" ) ).unwrap_or( 0 );

  // modelUsage — first model's stats
  let mu_marker  = "\"modelUsage\":{";
  let mu_str     = json.find( mu_marker ).map( |p| &json[ p + mu_marker.len() .. ] );
  let model_name = mu_str.and_then( |s| {
    let q1    = s.find( '"' )?;
    let inner = &s[ q1 + 1 .. ];
    let q2    = inner.find( '"' )?;
    Some( inner[ ..q2 ].to_string() )
  } ).unwrap_or_default();
  let mu_inner   = mu_str.and_then( |s| s.find( '{' ).map( |p| &s[ p + 1 .. ] ) );
  let m_in_tok   = mu_inner.and_then( |s| extract_u64( s, "inputTokens" ) ).unwrap_or( 0 );
  let m_out_tok  = mu_inner.and_then( |s| extract_u64( s, "outputTokens" ) ).unwrap_or( 0 );
  let m_cr_tok   = mu_inner.and_then( |s| extract_u64( s, "cacheReadInputTokens" ) ).unwrap_or( 0 );
  let m_cc_tok   = mu_inner.and_then( |s| extract_u64( s, "cacheCreationInputTokens" ) ).unwrap_or( 0 );
  let m_ws       = mu_inner.and_then( |s| extract_u64( s, "webSearchRequests" ) ).unwrap_or( 0 );
  let m_cost     = mu_inner.and_then( |s| extract_f64( s, "costUSD" ) ).unwrap_or( 0.0 );
  let m_ctx      = mu_inner.and_then( |s| extract_u64( s, "contextWindow" ) ).unwrap_or( 0 );
  let m_max_out  = mu_inner.and_then( |s| extract_u64( s, "maxOutputTokens" ) ).unwrap_or( 0 );

  let denials  = count_permission_denials( json );
  let is_err_s = if is_error { "true" } else { "false" };

  let mut out = String::new();
  if has( "type" )            { let _ = writeln!( out, "{CYAN}type:{RESET} {GREEN}{msg_type}{RESET}" ); }
  if has( "subtype" )         { let _ = writeln!( out, "{CYAN}subtype:{RESET} {GREEN}{subtype}{RESET}" ); }
  if has( "session_id" )      { let _ = writeln!( out, "{CYAN}session_id:{RESET} {GREEN}{session_id}{RESET}" ); }
  if has( "uuid" )            { let _ = writeln!( out, "{CYAN}uuid:{RESET} {GREEN}{uuid}{RESET}" ); }
  if has( "is_error" )        { let _ = writeln!( out, "{CYAN}is_error:{RESET} {YELLOW}{is_err_s}{RESET}" ); }
  if has( "stop_reason" )     { let _ = writeln!( out, "{CYAN}stop_reason:{RESET} {GREEN}{stop_reason}{RESET}" ); }
  if has( "num_turns" )       { let _ = writeln!( out, "{CYAN}num_turns:{RESET} {YELLOW}{num_turns}{RESET}" ); }
  if has( "fast_mode_state" ) { let _ = writeln!( out, "{CYAN}fast_mode_state:{RESET} {GREEN}{fast_mode}{RESET}" ); }
  if has( "duration_ms" )     { let _ = writeln!( out, "{CYAN}duration_ms:{RESET} {YELLOW}{duration_ms}{RESET}" ); }
  if has( "duration_api_ms" ) { let _ = writeln!( out, "{CYAN}duration_api_ms:{RESET} {YELLOW}{duration_api}{RESET}" ); }
  if has( "input_tokens" )    { let _ = writeln!( out, "{CYAN}input_tokens:{RESET} {YELLOW}{in_tok}{RESET}" ); }
  if has( "output_tokens" )   { let _ = writeln!( out, "{CYAN}output_tokens:{RESET} {YELLOW}{out_tok}{RESET}" ); }
  if has( "cache_creation_input_tokens" )
  {
    let _ = writeln!( out, "{CYAN}cache_creation_input_tokens:{RESET} {YELLOW}{cache_create}{RESET}" );
  }
  if has( "cache_read_input_tokens" )
  {
    let _ = writeln!( out, "{CYAN}cache_read_input_tokens:{RESET} {YELLOW}{cache_read}{RESET}" );
  }
  if has( "total_cost_usd" )  { let _ = writeln!( out, "{CYAN}total_cost_usd:{RESET} {YELLOW}{cost:.4}{RESET}" ); }
  if has( "service_tier" )    { let _ = writeln!( out, "{CYAN}service_tier:{RESET} {GREEN}{svc_tier}{RESET}" ); }
  if has( "speed" )           { let _ = writeln!( out, "{CYAN}speed:{RESET} {GREEN}{spd}{RESET}" ); }
  if has( "inference_geo" )
  {
    let color = if inf_geo.is_empty() { DIM } else { GREEN };
    let _ = writeln!( out, "{CYAN}inference_geo:{RESET} {color}{inf_geo}{RESET}" );
  }
  if has( "web_search_requests" )
  {
    let _ = writeln!( out, "{CYAN}web_search_requests:{RESET} {YELLOW}{web_search}{RESET}" );
  }
  if has( "web_fetch_requests" )
  {
    let _ = writeln!( out, "{CYAN}web_fetch_requests:{RESET} {YELLOW}{web_fetch}{RESET}" );
  }
  if has( "cache_ephemeral_1h_input_tokens" )
  {
    let _ = writeln!( out, "{CYAN}cache_ephemeral_1h_input_tokens:{RESET} {YELLOW}{eph_1h}{RESET}" );
  }
  if has( "cache_ephemeral_5m_input_tokens" )
  {
    let _ = writeln!( out, "{CYAN}cache_ephemeral_5m_input_tokens:{RESET} {YELLOW}{eph_5m}{RESET}" );
  }
  if has( "model" )           { let _ = writeln!( out, "{CYAN}model:{RESET} {GREEN}{model_name}{RESET}" ); }
  if has( "model_input_tokens" )
  {
    let _ = writeln!( out, "{CYAN}model_input_tokens:{RESET} {YELLOW}{m_in_tok}{RESET}" );
  }
  if has( "model_output_tokens" )
  {
    let _ = writeln!( out, "{CYAN}model_output_tokens:{RESET} {YELLOW}{m_out_tok}{RESET}" );
  }
  if has( "model_cache_read_input_tokens" )
  {
    let _ = writeln!( out, "{CYAN}model_cache_read_input_tokens:{RESET} {YELLOW}{m_cr_tok}{RESET}" );
  }
  if has( "model_cache_creation_input_tokens" )
  {
    let _ = writeln!( out, "{CYAN}model_cache_creation_input_tokens:{RESET} {YELLOW}{m_cc_tok}{RESET}" );
  }
  if has( "model_web_search_requests" )
  {
    let _ = writeln!( out, "{CYAN}model_web_search_requests:{RESET} {YELLOW}{m_ws}{RESET}" );
  }
  if has( "model_cost_usd" )  { let _ = writeln!( out, "{CYAN}model_cost_usd:{RESET} {YELLOW}{m_cost:.4}{RESET}" ); }
  if has( "model_context_window" )
  {
    let _ = writeln!( out, "{CYAN}model_context_window:{RESET} {YELLOW}{m_ctx}{RESET}" );
  }
  if has( "model_max_output_tokens" )
  {
    let _ = writeln!( out, "{CYAN}model_max_output_tokens:{RESET} {YELLOW}{m_max_out}{RESET}" );
  }
  if has( "permission_denials" )
  {
    let _ = writeln!( out, "{CYAN}permission_denials:{RESET} {YELLOW}{denials}{RESET}" );
  }

  let _ = writeln!( out, "{DIM}---{RESET}" );
  out.push_str( &result );
  if !result.is_empty() && !result.ends_with( '\n' ) { out.push( '\n' ); }

  Some( out )
}
