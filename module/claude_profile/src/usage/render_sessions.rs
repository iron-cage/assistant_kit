//! Sessions marker table for quota output footer.

use data_fmt::{ RowBuilder, TableFormatter, TableConfig, Format };

/// Append sessions table to `body` when `store_path` is provided and visibility allows.
///
/// `who = None` → auto (show when >1 `_active_*` marker); `Some(true)` → force on;
/// `Some(false)` → suppress.
pub( crate ) fn append_sessions_table(
  body       : String,
  store_path : Option< &std::path::Path >,
  who        : Option< bool >,
) -> String
{
  let Some( store ) = store_path else { return body; };
  let ( marker_count, sessions_text ) = build_sessions_table( store );
  let show = match who
  {
    Some( true  ) => true,
    Some( false ) => false,
    None          => marker_count > 1,
  };
  if show && !sessions_text.is_empty()
  {
    format!( "{body}\n{sessions_text}" )
  }
  else
  {
    body
  }
}

/// Read all `_active_*` markers from `store_path`, render as `{user}@{host} → account` table.
///
/// Returns `(marker_count, table_string)`. `marker_count` includes the own marker.
/// Own session receives `✓` appended to the Account column.
fn build_sessions_table( store_path : &std::path::Path ) -> ( usize, String )
{
  use claude_profile_core::account::active_marker_filename;

  let own_marker = active_marker_filename();

  // Collect all `_active_*` entries from the credential store.
  let entries : Vec< ( String, String, bool ) > =
    std::fs::read_dir( store_path )
      .ok()
      .into_iter()
      .flatten()
      .filter_map( Result::ok )
      .filter_map( | entry |
      {
        let fname = entry.file_name().to_string_lossy().into_owned();
        if !fname.starts_with( "_active_" ) { return None; }
        // Parse `{user}@{host}` from `_active_{host}_{user}`: strip prefix, split on last `_`.
        let rest = fname.strip_prefix( "_active_" )?;
        let ( host, user ) = rest.rsplit_once( '_' )?;
        let identity    = format!( "{user}@{host}" );
        let account_raw = std::fs::read_to_string( entry.path() ).unwrap_or_default();
        let account     = account_raw.trim().to_string();
        let is_own      = fname == own_marker;
        Some( ( identity, account, is_own ) )
      } )
      .collect();

  let marker_count = entries.len();
  if marker_count == 0 { return ( 0, String::new() ); }

  // Build table with `data_fmt`.
  let headers = vec![ "Session".to_string(), "Account".to_string() ];
  let mut builder = RowBuilder::new( headers );
  for ( identity, account, is_own ) in &entries
  {
    let account_cell = if *is_own
    {
      format!( "{account} ✓" )
    }
    else
    {
      account.clone()
    };
    builder = builder.add_row( vec![ identity.clone().into(), account_cell.into() ] );
  }
  let view  = builder.build_view();
  let table = Format::format(
    &TableFormatter::with_config( TableConfig::default().with_auto_wrap( false ) ),
    &view,
  ).unwrap_or_default();

  ( marker_count, format!( "Sessions\n\n{table}\n" ) )
}
