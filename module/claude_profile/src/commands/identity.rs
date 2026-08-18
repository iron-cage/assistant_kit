//! `.tags`, `.identities`, `.identity.filter` — Identity-centric listings and
//! the per-identity include/exclude tag filter (Features 075/076).
//!
//! Store-file surfaces: `_active_{slug}` markers, `_filter_{slug}` filter files,
//! and the `owner`/`tags` fields of `{name}.json`. The current-identity filter
//! path delegates to core `read_filter`/`write_filter` (env-derived slug);
//! `identity::USER@HOST` routing builds the explicit slug locally because core
//! keeps `host_user_slug()` private and task 528 leaves core untouched (C16).
//! Explicit-slug files are read via core's exported `parse_string_array_field`
//! and written as hand-built JSON (`"key": [...]`, no space before the colon —
//! the shape that parser requires) per this crate's zero-third-party-deps rule.

use core::fmt::Write as _;
use std::collections::{ BTreeMap, BTreeSet };
use std::path::{ Path, PathBuf };
use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use super::cmd_args::io_err_to_error_data;
use super::cmd_context::require_credential_store;
use crate::output::json_escape;

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Output format for the Identity commands: `text` (default) or `json` only.
#[ derive( Clone, Copy, PartialEq, Eq ) ]
enum IdentityFormat
{
  Text,
  Json,
}

/// Parse `format::` — `text`/`json` only; `table` (and anything else) is rejected.
fn parse_format( cmd : &VerifiedCommand ) -> Result< IdentityFormat, ErrorData >
{
  match cmd.arguments.get( "format" )
  {
    None => Ok( IdentityFormat::Text ),
    Some( Value::String( s ) ) if s.is_empty() || s == "text" => Ok( IdentityFormat::Text ),
    Some( Value::String( s ) ) if s == "json" => Ok( IdentityFormat::Json ),
    Some( Value::String( s ) ) => Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "format:: must be `text` (default) or `json` — got '{s}'" ),
    ) ),
    _ => Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "format:: must be `text` (default) or `json`".to_string(),
    ) ),
  }
}

/// Sanitize one identity component for slug use — keep `[A-Za-z0-9.-]`, else `_`
/// (mirrors core `host_user_slug()`'s cleaning, kept CLI-local per C16).
fn clean_component( raw : &str ) -> String
{
  raw.chars()
    .map( | c | if c.is_ascii_alphanumeric() || c == '.' || c == '-' { c } else { '_' } )
    .collect()
}

/// Parse `identity::USER@HOST` — exactly one `@`, both components non-empty.
fn parse_identity( raw : &str ) -> Result< ( String, String ), ErrorData >
{
  let parts : Vec< &str > = raw.split( '@' ).collect();
  if parts.len() != 2 || parts[ 0 ].is_empty() || parts[ 1 ].is_empty()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "identity:: must be USER@HOST with non-empty components — got '{raw}'" ),
    ) );
  }
  Ok( ( parts[ 0 ].to_string(), parts[ 1 ].to_string() ) )
}

/// Filter-file path for an explicit identity: `_filter_{clean(host)}_{clean(user)}`.
fn explicit_filter_path( store : &Path, user : &str, host : &str ) -> PathBuf
{
  store.join( format!( "_filter_{}_{}", clean_component( host ), clean_component( user ) ) )
}

/// Display form of a marker/filter slug: last-`_` split → `user@host`.
///
/// Lossy for hosts whose slug itself contains `_` (sanitization is one-way);
/// slugs with no `_` at all render verbatim.
fn display_from_slug( slug : &str ) -> String
{
  match slug.rsplit_once( '_' )
  {
    Some( ( host, user ) ) => format!( "{user}@{host}" ),
    None                   => slug.to_string(),
  }
}

/// Read an explicit-slug filter file: absent → permit-all default.
// core::io::ErrorKind requires the unstable `core_io` feature (rust-lang/rust#154046) — not usable on stable.
#[ allow( clippy::std_instead_of_core ) ]
fn read_filter_at( path : &Path ) -> Result< ( Vec< String >, Vec< String > ), ErrorData >
{
  match std::fs::read_to_string( path )
  {
    Err( e ) if e.kind() == std::io::ErrorKind::NotFound => Ok( ( Vec::new(), Vec::new() ) ),
    Err( e ) => Err( io_err_to_error_data( &e, "identity filter read" ) ),
    Ok( text ) => Ok( (
      crate::account::parse_string_array_field( &text, "include" ),
      crate::account::parse_string_array_field( &text, "exclude" ),
    ) ),
  }
}

/// Serialize a tag list as a JSON string array.
fn tags_json( tags : &[ String ] ) -> String
{
  let inner : Vec< String > = tags.iter()
    .map( | t | format!( "\"{}\"", json_escape( t ) ) )
    .collect();
  format!( "[{}]", inner.join( "," ) )
}

/// Write an explicit-slug filter file with core-equivalent semantics: one side
/// given → side-replace (other side preserved via read-merge); both given →
/// full overwrite; include ∩ exclude overlap rejected before any write.
fn write_filter_at(
  path    : &Path,
  include : Option< &[ String ] >,
  exclude : Option< &[ String ] >,
) -> Result< ( Vec< String >, Vec< String > ), ErrorData >
{
  let ( cur_inc, cur_exc ) = if include.is_some() && exclude.is_some()
  { ( Vec::new(), Vec::new() ) }
  else
  { read_filter_at( path )? };
  let inc = crate::account::normalize_tag_set( include.unwrap_or( &cur_inc ) )
    .map_err( | e | io_err_to_error_data( &e, "identity filter" ) )?;
  let exc = crate::account::normalize_tag_set( exclude.unwrap_or( &cur_exc ) )
    .map_err( | e | io_err_to_error_data( &e, "identity filter" ) )?;
  let overlap : Vec< String > = inc.iter().filter( | t | exc.contains( t ) ).cloned().collect();
  if !overlap.is_empty()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "include ∩ exclude must be empty — overlapping: {}", overlap.join( ", " ) ),
    ) );
  }
  let json = format!( "{{\n  \"include\": {},\n  \"exclude\": {}\n}}\n", tags_json( &inc ), tags_json( &exc ) );
  claude_core::file_io::atomic_write( path, &json )
    .map_err( | e | io_err_to_error_data( &e, "identity filter write" ) )?;
  Ok( ( inc, exc ) )
}

/// Enumerate `_filter_*` files in the store as (slug, path) pairs.
fn filter_files( store : &Path ) -> Result< Vec< ( String, PathBuf ) >, ErrorData >
{
  let mut found = Vec::new();
  let entries = std::fs::read_dir( store )
    .map_err( | e | io_err_to_error_data( &e, "identity scan" ) )?;
  for entry in entries
  {
    let entry = entry.map_err( | e | io_err_to_error_data( &e, "identity scan" ) )?;
    let fname = entry.file_name().to_string_lossy().to_string();
    if let Some( slug ) = fname.strip_prefix( "_filter_" )
    {
      if !slug.is_empty() { found.push( ( slug.to_string(), entry.path() ) ); }
    }
  }
  Ok( found )
}

// ── .tags ─────────────────────────────────────────────────────────────────────

/// Empty-state output for `.tags`.
fn empty_tags_output( fmt : IdentityFormat ) -> OutputData
{
  match fmt
  {
    IdentityFormat::Json => OutputData::new( "[]\n".to_string(), "text" ),
    IdentityFormat::Text => OutputData::new( "(no tags)\n".to_string(), "text" ),
  }
}

/// `.tags` — union every tag across account metadata and identity filter files,
/// with per-tag usage counts (accounts carrying it; filter files referencing it).
///
/// Read-only; store unavailable or empty union → `(no tags)` / `[]` with exit 0.
///
/// # Errors
///
/// Returns `ErrorData` if `format::` is not `text`/`json` (exit 1) or the store
/// directory is unreadable (exit 2).
#[ inline ]
pub fn tags_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let fmt = parse_format( &cmd )?;
  let Ok( credential_store ) = require_credential_store() else { return Ok( empty_tags_output( fmt ) ); };
  if !credential_store.exists() { return Ok( empty_tags_output( fmt ) ); }

  // Account-side counts: how many accounts carry each tag.
  let mut acct_counts : BTreeMap< String, usize > = BTreeMap::new();
  let accounts = crate::account::list( &credential_store )
    .map_err( | e | io_err_to_error_data( &e, "tags" ) )?;
  for account in &accounts
  {
    for tag in &account.tags
    {
      *acct_counts.entry( tag.clone() ).or_insert( 0 ) += 1;
    }
  }

  // Filter-side counts: one per filter FILE referencing the tag (include ∪ exclude).
  let mut filter_counts : BTreeMap< String, usize > = BTreeMap::new();
  for ( _slug, path ) in filter_files( &credential_store )?
  {
    let text = std::fs::read_to_string( &path ).unwrap_or_default();
    let mut seen : BTreeSet< String > = BTreeSet::new();
    seen.extend( crate::account::parse_string_array_field( &text, "include" ) );
    seen.extend( crate::account::parse_string_array_field( &text, "exclude" ) );
    for tag in seen
    {
      *filter_counts.entry( tag ).or_insert( 0 ) += 1;
    }
  }

  let mut all_tags : BTreeSet< String > = BTreeSet::new();
  all_tags.extend( acct_counts.keys().cloned() );
  all_tags.extend( filter_counts.keys().cloned() );
  if all_tags.is_empty() { return Ok( empty_tags_output( fmt ) ); }

  let rows : Vec< ( String, usize, usize ) > = all_tags.into_iter()
    .map( | tag |
    {
      let accounts_n = acct_counts.get( &tag ).copied().unwrap_or( 0 );
      let filters_n  = filter_counts.get( &tag ).copied().unwrap_or( 0 );
      ( tag, accounts_n, filters_n )
    } )
    .collect();

  let body = match fmt
  {
    IdentityFormat::Json =>
    {
      let entries : Vec< String > = rows.iter()
        .map( | ( tag, accounts_n, filters_n ) |
          format!( "{{\"tag\":\"{}\",\"accounts\":{accounts_n},\"filters\":{filters_n}}}", json_escape( tag ) ) )
        .collect();
      format!( "[{}]\n", entries.join( "," ) )
    }
    IdentityFormat::Text =>
    {
      let w_tag = rows.iter().map( | r | r.0.chars().count() )
        .chain( core::iter::once( "Tag".len() ) )
        .max()
        .unwrap_or( 3 );
      let mut out = String::new();
      let _ = writeln!( out, "{:<w_tag$}  {:<8}  Filters", "Tag", "Accounts" );
      for ( tag, accounts_n, filters_n ) in &rows
      {
        let _ = writeln!( out, "{tag:<w_tag$}  {accounts_n:<8}  {filters_n}" );
      }
      out
    }
  };
  Ok( OutputData::new( body, "text" ) )
}

// ── .identities ───────────────────────────────────────────────────────────────

/// One `.identities` row, keyed by display identity (`user@host`).
#[ derive( Default ) ]
struct IdentityRow
{
  active  : String,
  include : Vec< String >,
  exclude : Vec< String >,
  owned   : usize,
}

/// Empty-state output for `.identities`.
fn empty_identities_output( fmt : IdentityFormat ) -> OutputData
{
  match fmt
  {
    IdentityFormat::Json => OutputData::new( "[]\n".to_string(), "text" ),
    IdentityFormat::Text => OutputData::new( "(no identities)\n".to_string(), "text" ),
  }
}

/// `.identities` — union every known Identity from `_active_*` markers,
/// `_filter_*` files, and account `owner` fields, with per-identity state.
///
/// A saved account alone contributes NO identity — only markers, filters, and
/// non-empty owners do. Read-only; empty union → `(no identities)` / `[]`.
///
/// # Errors
///
/// Returns `ErrorData` if `format::` is not `text`/`json` (exit 1) or the store
/// directory is unreadable (exit 2).
#[ inline ]
#[ allow( clippy::too_many_lines ) ]
pub fn identities_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let fmt = parse_format( &cmd )?;
  let Ok( credential_store ) = require_credential_store() else { return Ok( empty_identities_output( fmt ) ); };
  if !credential_store.exists() { return Ok( empty_identities_output( fmt ) ); }

  let mut rows : BTreeMap< String, IdentityRow > = BTreeMap::new();

  // One directory pass: `_active_{slug}` markers and `_filter_{slug}` files.
  let entries = std::fs::read_dir( &credential_store )
    .map_err( | e | io_err_to_error_data( &e, "identity scan" ) )?;
  for entry in entries
  {
    let entry = entry.map_err( | e | io_err_to_error_data( &e, "identity scan" ) )?;
    let fname = entry.file_name().to_string_lossy().to_string();
    if let Some( slug ) = fname.strip_prefix( "_active_" )
    {
      if slug.is_empty() { continue; }
      let content = std::fs::read_to_string( entry.path() ).unwrap_or_default().trim().to_string();
      rows.entry( display_from_slug( slug ) ).or_default().active = content;
    }
    else if let Some( slug ) = fname.strip_prefix( "_filter_" )
    {
      if slug.is_empty() { continue; }
      let text = std::fs::read_to_string( entry.path() ).unwrap_or_default();
      let row  = rows.entry( display_from_slug( slug ) ).or_default();
      row.include = crate::account::parse_string_array_field( &text, "include" );
      row.exclude = crate::account::parse_string_array_field( &text, "exclude" );
    }
  }

  // Owner fields: each non-empty owner is an identity; count accounts it owns.
  let accounts = crate::account::list( &credential_store )
    .map_err( | e | io_err_to_error_data( &e, "identities" ) )?;
  for account in &accounts
  {
    if !account.owner.is_empty()
    {
      rows.entry( account.owner.clone() ).or_default().owned += 1;
    }
  }

  if rows.is_empty() { return Ok( empty_identities_output( fmt ) ); }

  let body = match fmt
  {
    IdentityFormat::Json =>
    {
      let entries : Vec< String > = rows.iter()
        .map( | ( display, row ) |
          format!(
            "{{\"identity\":\"{}\",\"active\":\"{}\",\"owned\":{},\"include\":{},\"exclude\":{}}}",
            json_escape( display ),
            json_escape( &row.active ),
            row.owned,
            tags_json( &row.include ),
            tags_json( &row.exclude ),
          ) )
        .collect();
      format!( "[{}]\n", entries.join( "," ) )
    }
    IdentityFormat::Text =>
    {
      const HEADERS : [ &str ; 5 ] = [ "Identity", "Active", "Owned", "Include", "Exclude" ];
      let dash = "\u{2014}";
      let cell = | list : &[ String ] | if list.is_empty() { dash.to_string() } else { list.join( ", " ) };
      let table : Vec< [ String ; 5 ] > = rows.iter()
        .map( | ( display, row ) |
        [
          display.clone(),
          if row.active.is_empty() { dash.to_string() } else { row.active.clone() },
          row.owned.to_string(),
          cell( &row.include ),
          cell( &row.exclude ),
        ] )
        .collect();
      let mut widths : [ usize ; 5 ] = [ 0 ; 5 ];
      for ( i, header ) in HEADERS.iter().enumerate() { widths[ i ] = header.len(); }
      for line in &table
      {
        for ( i, text ) in line.iter().enumerate()
        {
          widths[ i ] = widths[ i ].max( text.chars().count() );
        }
      }
      let mut out = String::new();
      for ( i, header ) in HEADERS.iter().enumerate()
      {
        if i > 0 { out.push_str( "  " ); }
        if i == HEADERS.len() - 1 { out.push_str( header ); }
        else { let w = widths[ i ]; let _ = write!( out, "{header:<w$}" ); }
      }
      out.push( '\n' );
      for line in &table
      {
        for ( i, text ) in line.iter().enumerate()
        {
          if i > 0 { out.push_str( "  " ); }
          if i == line.len() - 1 { out.push_str( text ); }
          else { let w = widths[ i ]; let _ = write!( out, "{text:<w$}" ); }
        }
        out.push( '\n' );
      }
      out
    }
  };
  Ok( OutputData::new( body, "text" ) )
}

// ── .identity.filter ──────────────────────────────────────────────────────────

/// Render a filter get/set result in the selected format.
fn render_filter( fmt : IdentityFormat, identity : &str, include : &[ String ], exclude : &[ String ] ) -> String
{
  match fmt
  {
    IdentityFormat::Json => format!(
      "{{\"identity\":\"{}\",\"include\":{},\"exclude\":{}}}\n",
      json_escape( identity ),
      tags_json( include ),
      tags_json( exclude ),
    ),
    IdentityFormat::Text =>
    {
      let mut line = format!( "include=[{}] exclude=[{}]", include.join( ", " ), exclude.join( ", " ) );
      if include.is_empty() && exclude.is_empty() { line.push_str( " (permit-all)" ); }
      line.push( '\n' );
      line
    }
  }
}

/// Typo guard (Feature 076): after a successful write, warn on stderr about
/// written tags carried by no saved account — non-blocking, exit stays 0.
fn warn_unknown_tags( store : &Path, include : &[ String ], exclude : &[ String ] )
{
  let Ok( accounts ) = crate::account::list( store ) else { return };
  let mut known : BTreeSet< &str > = BTreeSet::new();
  for account in &accounts
  {
    for tag in &account.tags { known.insert( tag.as_str() ); }
  }
  for tag in include.iter().chain( exclude.iter() )
  {
    if !known.contains( tag.as_str() )
    {
      eprintln!( "warning: tag '{tag}' matches no saved account" );
    }
  }
}

/// `.identity.filter` — get, set, or clear an Identity's include/exclude tag filter.
///
/// No set params → get (absent file ≡ permit-all). `include::`/`exclude::` →
/// side-replace write (core semantics). `clear::1` → idempotent file delete,
/// mutually exclusive with the set params. `identity::USER@HOST` targets an
/// explicit Identity's filter file instead of the current env-derived one.
///
/// # Errors
///
/// Returns `ErrorData` if `format::`/`identity::` is malformed, a tag is
/// invalid, include ∩ exclude overlaps, or `clear::1` is combined with a set
/// param (all exit 1); store IO failures and a malformed current-identity
/// filter file exit 2.
#[ inline ]
// core::io::ErrorKind requires the unstable `core_io` feature (rust-lang/rust#154046) — not usable on stable.
#[ allow( clippy::std_instead_of_core ) ]
pub fn identity_filter_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let fmt              = parse_format( &cmd )?;
  let credential_store = require_credential_store()?;

  let target = match cmd.arguments.get( "identity" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => Some( parse_identity( s )? ),
    _ => None,
  };
  let identity_display = match &target
  {
    Some( ( user, host ) ) => format!( "{user}@{host}" ),
    None                   => crate::account::current_identity(),
  };

  // An explicitly-empty include::/exclude:: still counts as given — it fails tag
  // validation loudly below instead of silently no-oping.
  let split_arg = | key : &str | -> Option< Vec< String > >
  {
    match cmd.arguments.get( key )
    {
      Some( Value::String( s ) ) => Some( s.split( ',' ).map( str::to_string ).collect() ),
      _ => None,
    }
  };
  let include_arg = split_arg( "include" );
  let exclude_arg = split_arg( "exclude" );
  let clear       = crate::output::parse_int_flag( &cmd, "clear", 0 )? != 0;

  if clear && ( include_arg.is_some() || exclude_arg.is_some() )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "clear::1 cannot be combined with include:: or exclude::".to_string(),
    ) );
  }

  if clear
  {
    let path = match &target
    {
      Some( ( user, host ) ) => explicit_filter_path( &credential_store, user, host ),
      None                   => credential_store.join( crate::account::filter_filename() ),
    };
    match std::fs::remove_file( &path )
    {
      Ok( () ) => {}
      Err( e ) if e.kind() == std::io::ErrorKind::NotFound => {} // idempotent
      Err( e ) => return Err( io_err_to_error_data( &e, "identity filter clear" ) ),
    }
    return Ok( OutputData::new( format!( "{identity_display}: filter cleared — permit-all\n" ), "text" ) );
  }

  if include_arg.is_some() || exclude_arg.is_some()
  {
    // A filter may legitimately be the store's first file.
    std::fs::create_dir_all( &credential_store )
      .map_err( | e | io_err_to_error_data( &e, "identity filter write" ) )?;
    let ( inc, exc ) = if let Some( ( user, host ) ) = &target
    {
      write_filter_at(
        &explicit_filter_path( &credential_store, user, host ),
        include_arg.as_deref(),
        exclude_arg.as_deref(),
      )?
    }
    else
    {
      let stored = crate::account::write_filter( &credential_store, include_arg.as_deref(), exclude_arg.as_deref() )
        .map_err( | e | io_err_to_error_data( &e, "identity filter write" ) )?;
      ( stored.include, stored.exclude )
    };
    warn_unknown_tags( &credential_store, &inc, &exc );
    return Ok( OutputData::new( render_filter( fmt, &identity_display, &inc, &exc ), "text" ) );
  }

  // Get.
  let ( inc, exc ) = if let Some( ( user, host ) ) = &target
  {
    read_filter_at( &explicit_filter_path( &credential_store, user, host ) )?
  }
  else
  {
    let stored = crate::account::read_filter( &credential_store )
      .map_err( | e | io_err_to_error_data( &e, "identity filter read" ) )?;
    ( stored.include, stored.exclude )
  };
  Ok( OutputData::new( render_filter( fmt, &identity_display, &inc, &exc ), "text" ) )
}
