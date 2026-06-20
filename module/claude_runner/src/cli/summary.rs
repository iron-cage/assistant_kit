//! `summary` output-format rendering.
//!
//! Parses the JSON response emitted by `claude --output-format json` and renders
//! a YAML metadata header (boxed, ANSI-colored) followed by a `---` separator
//! and the extracted text body.
//!
//! Called from `execution::run_print_mode()` when `cli.output_format == "summary"`
//! and claude exits 0.  On parse failure the caller falls back to raw output.

use core::fmt::Write as _;

const CYAN         : &str = "\x1b[36m";
const GREEN        : &str = "\x1b[32m";
const YELLOW       : &str = "\x1b[33m";
const DIM          : &str = "\x1b[2m";
const MAGENTA      : &str = "\x1b[35m";
const BRIGHT_BLACK : &str = "\x1b[90m";
const BOLD_GREEN   : &str = "\x1b[1;32m";
const RESET        : &str = "\x1b[0m";

// ── Minimal JSON extraction ────────────────────────────────────────────────────

/// Extract a JSON string value for `key`.  Returns `None` for `null` or absent keys.
fn extract_str( s : &str, key : &str ) -> Option< String >
{
  let needle = format!( "\"{key}\":" );
  let pos    = s.find( &needle )?;
  let rest   = s[ pos + needle.len() .. ].trim_start_matches( ' ' );
  if rest.starts_with( "null" ) { return None; }
  if !rest.starts_with( '"' )   { return None; }
  let inner  = &rest[ 1 .. ];
  let mut end    = inner.len();
  let mut escape = false;
  for ( i, c ) in inner.char_indices()
  {
    if escape    { escape = false; continue; }
    if c == '\\' { escape = true;  continue; }
    if c == '"'  { end = i; break; }
  }
  Some( inner[ ..end ].to_string() )
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

/// Find the end of a balanced bracket expression.
///
/// `s` starts AFTER the opening bracket.  Handles string literals so brackets
/// and braces inside `"..."` do not affect depth.
fn find_close( s : &str, open : char, close : char ) -> usize
{
  let mut depth  = 1i32;
  let mut in_str = false;
  let mut escape = false;
  for ( i, c ) in s.char_indices()
  {
    if escape  { escape = false; continue; }
    if in_str
    {
      if c == '\\' { escape = true; }
      else if c == '"' { in_str = false; }
      continue;
    }
    if      c == '"'   { in_str = true; }
    else if c == open  { depth += 1; }
    else if c == close { depth -= 1; if depth == 0 { return i; } }
  }
  s.len()
}

// ── Content block parsing ──────────────────────────────────────────────────────

struct ContentBlock
{
  block_type : String,
  text_body  : Option< String >,   // extracted text (text blocks only)
  tool_name  : Option< String >,   // tool name (tool_use blocks)
  input_keys : Vec< String >,       // input object keys (tool_use blocks)
  fields     : Vec< String >,       // field name list (thinking blocks)
  is_error   : bool,               // error flag (tool_result blocks)
}

fn parse_input_keys( obj : &str ) -> Vec< String >
{
  let marker     = "\"input\":{";
  let Some( start ) = obj.find( marker ) else { return vec![]; };
  let rest       = &obj[ start + marker.len() .. ];
  let body       = &rest[ ..find_close( rest, '{', '}' ) ];
  let mut keys   = Vec::new();
  let mut pos    = 0usize;
  while pos < body.len()
  {
    let Some( q ) = body[ pos .. ].find( '"' ) else { break; };
    let key_start = pos + q + 1;
    let key_rest  = &body[ key_start .. ];
    let Some( q2 ) = key_rest.find( '"' ) else { break; };
    let after     = key_rest[ q2 + 1 .. ].trim_start_matches( ' ' );
    if after.starts_with( ':' ) { keys.push( key_rest[ ..q2 ].to_string() ); }
    pos = key_start + q2 + 1;
  }
  keys
}

fn parse_content_blocks( json : &str ) -> Vec< ContentBlock >
{
  let marker = "\"content\":[";
  let Some( start ) = json.find( marker ) else { return vec![]; };
  let rest      = &json[ start + marker.len() .. ];
  let array_str = &rest[ ..find_close( rest, '[', ']' ) ];
  let mut blocks = Vec::new();
  let mut pos    = 0usize;
  while pos < array_str.len()
  {
    let Some( brace ) = array_str[ pos .. ].find( '{' ) else { break; };
    let obj_start = pos + brace + 1;
    let obj_rest  = &array_str[ obj_start .. ];
    let obj_end   = find_close( obj_rest, '{', '}' );
    let obj       = &obj_rest[ ..obj_end ];
    pos           = obj_start + obj_end + 1;

    let Some( block_type ) = extract_str( obj, "type" ) else { continue; };
    let block = match block_type.as_str()
    {
      "text" =>
      {
        let text = extract_str( obj, "text" ).unwrap_or_default();
        ContentBlock
        {
          block_type : "text".to_string(),
          text_body  : Some( text ),
          tool_name  : None,
          input_keys : vec![],
          fields     : vec![],
          is_error   : false,
        }
      }
      "thinking" =>
      {
        ContentBlock
        {
          block_type : "thinking".to_string(),
          text_body  : None,
          tool_name  : None,
          input_keys : vec![],
          fields     : vec![ "thinking".to_string(), "signature".to_string() ],
          is_error   : false,
        }
      }
      "tool_use" =>
      {
        ContentBlock
        {
          block_type : "tool_use".to_string(),
          text_body  : None,
          tool_name  : extract_str( obj, "name" ),
          input_keys : parse_input_keys( obj ),
          fields     : vec![],
          is_error   : false,
        }
      }
      "tool_result" =>
      {
        ContentBlock
        {
          block_type : "tool_result".to_string(),
          text_body  : None,
          tool_name  : None,
          input_keys : vec![],
          fields     : vec![],
          is_error   : obj.contains( "\"is_error\":true" ),
        }
      }
      other =>
      {
        ContentBlock
        {
          block_type : other.to_string(),
          text_body  : None,
          tool_name  : None,
          input_keys : vec![],
          fields     : vec![],
          is_error   : false,
        }
      }
    };
    blocks.push( block );
  }
  blocks
}

// ── Box rendering ──────────────────────────────────────────────────────────────

/// A display line with plain text (for width) and ANSI-colored text (for output).
struct Line { plain : String, colored : String }

impl Line
{
  fn new( plain : impl Into< String >, colored : impl Into< String > ) -> Self
  {
    Self { plain : plain.into(), colored : colored.into() }
  }
}

fn render_box( lines : &[ Line ] ) -> String
{
  let width = lines.iter().map( |l| l.plain.len() ).max().unwrap_or( 0 );
  let bar   = "─".repeat( width + 2 );
  let mut out = format!( "╭{bar}╮\n" );
  for l in lines
  {
    let pad = " ".repeat( width - l.plain.len() );
    let _ = writeln!( out, "│ {}{pad} │", l.colored );
  }
  let _ = writeln!( out, "╰{bar}╯" );
  out
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// Render `summary` format output from a successful `--output-format json` response.
///
/// Returns `Some(rendered)` on success, `None` when the JSON cannot be parsed
/// (caller should fall back to printing raw `json`).
#[ allow( clippy::too_many_lines ) ]
pub( super ) fn render_summary( json : &str ) -> Option< String >
{
  let id            = extract_str( json, "id" )?;
  let msg_type      = extract_str( json, "type" ).unwrap_or_default();
  let role          = extract_str( json, "role" ).unwrap_or_default();
  let model         = extract_str( json, "model" ).unwrap_or_default();
  let stop_reason   = extract_str( json, "stop_reason" );
  let stop_sequence = extract_str( json, "stop_sequence" );

  let usage_marker  = "\"usage\":{";
  let usage_str     = json.find( usage_marker ).map( |p| &json[ p + usage_marker.len() .. ] );
  let input_tokens  = usage_str.and_then( |s| extract_u64( s, "input_tokens" ) ).unwrap_or( 0 );
  let output_tokens = usage_str.and_then( |s| extract_u64( s, "output_tokens" ) ).unwrap_or( 0 );

  let blocks = parse_content_blocks( json );

  // ── Build header lines ──────────────────────────────────────────────────────

  let mut lines = Vec::< Line >::new();

  // Helper: key:value line for Option<&str> (None → "null")
  let push_kv = |lines : &mut Vec< Line >, k : &str, v : Option< &str >|
  {
    let (  kp,  kc ) = ( format!( "{k}:" ), format!( "{CYAN}{k}:{RESET}" ) );
    let ( vp, vc ) = match v
    {
      Some( s ) =>
      (
        format!( "\"{s}\"" ),
        format!( "{GREEN}\"{s}\"{RESET}" ),
      ),
      None =>
      (
        "null".to_string(),
        format!( "{DIM}null{RESET}" ),
      ),
    };
    lines.push( Line::new( format!( " {kp}  {vp}" ), format!( " {kc}  {vc}" ) ) );
  };

  push_kv( &mut lines, "id",            Some( id.as_str() ) );
  push_kv( &mut lines, "type",          Some( msg_type.as_str() ) );
  push_kv( &mut lines, "role",          Some( role.as_str() ) );
  push_kv( &mut lines, "model",         Some( model.as_str() ) );
  push_kv( &mut lines, "stop_reason",   stop_reason.as_deref() );
  push_kv( &mut lines, "stop_sequence", stop_sequence.as_deref() );

  // content: block count + per-block topology
  {
    let count  = blocks.len();
    let plural = if count == 1 { "block" } else { "blocks" };
    lines.push( Line::new(
      format!( " content:  {count} {plural}" ),
      format!( " {CYAN}content:{RESET}  {YELLOW}{count}{RESET} {plural}" ),
    ) );
  }
  for ( idx, block ) in blocks.iter().enumerate()
  {
    let ip = format!( "[{idx}]" );
    let ic = format!( "{BRIGHT_BLACK}[{idx}]{RESET}" );
    let tp = block.block_type.as_str();
    let tc = format!( "{MAGENTA}{tp}{RESET}" );

    let ( topo_p, topo_c ) = match tp
    {
      "text" =>
      {
        let chars = block.text_body.as_deref().unwrap_or( "" ).len();
        (
          format!( "   {ip} {tp}  {chars} chars" ),
          format!( "   {ic} {tc}  {YELLOW}{chars}{RESET} chars" ),
        )
      }
      "thinking" =>
      {
        let f = block.fields.join( ", " );
        (
          format!( "   {ip} {tp}  {{{f}}}" ),
          format!( "   {ic} {tc}  {{{f}}}" ),
        )
      }
      "tool_use" =>
      {
        let name = block.tool_name.as_deref().unwrap_or( "?" );
        let keys = if block.input_keys.is_empty()
        {
          String::new()
        }
        else
        {
          format!( " {{{}}}", block.input_keys.join( ", " ) )
        };
        (
          format!( "   {ip} {tp}  \"{name}\"{keys}" ),
          format!( "   {ic} {tc}  {BOLD_GREEN}\"{name}\"{RESET}{keys}" ),
        )
      }
      "tool_result" =>
      {
        let status = if block.is_error { "ERROR" } else { "ok" };
        (
          format!( "   {ip} {tp}  {status}" ),
          format!( "   {ic} {tc}  {status}" ),
        )
      }
      _ =>
      (
        format!( "   {ip} {tp}" ),
        format!( "   {ic} {tc}" ),
      ),
    };
    lines.push( Line::new( topo_p, topo_c ) );
  }

  // usage + token counts
  lines.push( Line::new(
    " usage:".to_string(),
    format!( " {CYAN}usage:{RESET}" ),
  ) );
  {
    let ( vp, vc ) = ( input_tokens.to_string(), format!( "{YELLOW}{input_tokens}{RESET}" ) );
    lines.push( Line::new(
      format!( "   input_tokens:  {vp}" ),
      format!( "   {CYAN}input_tokens:{RESET}  {vc}" ),
    ) );
  }
  {
    let ( vp, vc ) = ( output_tokens.to_string(), format!( "{YELLOW}{output_tokens}{RESET}" ) );
    lines.push( Line::new(
      format!( "   output_tokens: {vp}" ),
      format!( "   {CYAN}output_tokens:{RESET} {vc}" ),
    ) );
  }

  // ── Render box + separator + text body ─────────────────────────────────────

  let mut out = render_box( &lines );
  let _ = writeln!( out, "{DIM}---{RESET}" );

  for block in &blocks
  {
    if let Some( ref text ) = block.text_body
    {
      out.push_str( text );
      if !text.ends_with( '\n' ) { out.push( '\n' ); }
    }
  }

  Some( out )
}
