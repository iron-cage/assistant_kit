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

// ── Public API ─────────────────────────────────────────────────────────────────

/// Render `summary` format output from a successful `--output-format json` CLR response.
///
/// Returns `Some(rendered)` on success, `None` when the JSON cannot be parsed as a
/// CLR result envelope (caller should fall back to printing raw `json`).
pub( super ) fn render_summary( json : &str ) -> Option< String >
{
  // Fix(BUG-309): gate on "session_id" — CLR result envelope field; old code gated on "id" (Messages API)
  // Root cause: claude --output-format json emits CLR result envelope (session_id), not Messages API (id)
  // Pitfall: wrong-schema test fixtures masked 100% production failure; all 13 EC tests passed on bad fixtures
  let session_id    = extract_str( json, "session_id" )?;
  let msg_type      = extract_str( json, "type" ).unwrap_or_default();
  let subtype       = extract_str( json, "subtype" ).unwrap_or_default();
  let is_error      = json.contains( "\"is_error\":true" );
  let result        = extract_str( json, "result" ).unwrap_or_default();

  let usage_marker  = "\"usage\":{";
  let usage_str     = json.find( usage_marker ).map( |p| &json[ p + usage_marker.len() .. ] );
  let input_tokens  = usage_str.and_then( |s| extract_u64( s, "input_tokens" ) ).unwrap_or( 0 );
  let output_tokens = usage_str.and_then( |s| extract_u64( s, "output_tokens" ) ).unwrap_or( 0 );
  let cost          = extract_f64( json, "total_cost_usd" ).unwrap_or( 0.0 );

  let is_err_s = if is_error { "true" } else { "false" };

  let mut out = String::new();
  let _ = writeln!( out, "{CYAN}type:{RESET} {GREEN}{msg_type}{RESET}" );
  let _ = writeln!( out, "{CYAN}subtype:{RESET} {GREEN}{subtype}{RESET}" );
  let _ = writeln!( out, "{CYAN}session_id:{RESET} {GREEN}{session_id}{RESET}" );
  let _ = writeln!( out, "{CYAN}is_error:{RESET} {YELLOW}{is_err_s}{RESET}" );
  let _ = writeln!( out, "{CYAN}input_tokens:{RESET} {YELLOW}{input_tokens}{RESET}" );
  let _ = writeln!( out, "{CYAN}output_tokens:{RESET} {YELLOW}{output_tokens}{RESET}" );
  let _ = writeln!( out, "{CYAN}total_cost_usd:{RESET} {YELLOW}{cost:.4}{RESET}" );
  let _ = writeln!( out, "{DIM}---{RESET}" );

  out.push_str( &result );
  if !result.is_empty() && !result.ends_with( '\n' ) { out.push( '\n' ); }

  Some( out )
}

#[ cfg( test ) ]
mod tests
{
  use super::render_summary;

  /// EC-14: `render_summary()` returns `Some` for a valid CLR result envelope.
  ///
  /// Root Cause (BUG-309): old parser hard-gated on `"id"` field (Messages API); CLR envelopes
  /// have `"session_id"` instead — causing 100% production failure masked by wrong-schema fixtures.
  #[ test ]
  fn ec14_render_summary_clr_envelope_accepted()
  {
    let json     = r#"{"type":"result","subtype":"success","session_id":"00000000-0000-0000-0000-000000000001","is_error":false,"result":"hello world","usage":{"input_tokens":3,"output_tokens":4},"total_cost_usd":0.001}"#;
    let rendered = render_summary( json );
    assert!( rendered.is_some(), "render_summary must return Some for valid CLR envelope; got None" );
    let s = rendered.unwrap();
    assert!( s.contains( "---" ),          "rendered output must contain separator '---'. Got:\n{s}" );
    assert!( s.contains( "hello world" ),  "rendered output must contain the result text. Got:\n{s}" );
    assert!( s.contains( "session_id:" ),  "output must contain 'session_id:'. Got:\n{s}" );
  }

  /// Unescape test: JSON `\n` in `result` field becomes actual newline in output.
  #[ test ]
  fn extract_str_unescapes_json_newlines()
  {
    let json = r#"{"type":"result","subtype":"success","session_id":"x","is_error":false,"result":"line1\nline2","usage":{"input_tokens":0,"output_tokens":0},"total_cost_usd":0.0}"#;
    let rendered = render_summary( json ).expect( "must parse" );
    assert!( rendered.contains( "line1\nline2" ), "\\n must be unescaped to actual newline. Got:\n{rendered}" );
  }
}
