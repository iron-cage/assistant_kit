//! Shared comma-separated column-key validation for `ps` and `tools` — both parse a
//! `--columns`-style flag against their own fixed `COLUMN_KEYS` table using identical
//! split/trim/lookup logic; this module holds that logic once instead of twice.

// Validate a comma-separated column key string against `table`.
//
// Returns ordered `&'static str` keys (from `table` — not slices of `csv`) so callers
// have a stable `'static` lifetime regardless of where the input string lives.
pub( super ) fn validate_columns(
  csv   : &str,
  table : &[ ( &'static str, &str ) ],
) -> Result< Vec< &'static str >, String >
{
  let mut out = Vec::new();
  for raw in csv.split( ',' )
  {
    let key = raw.trim();
    if let Some( ( k, _ ) ) = table.iter().find( | ( k, _ ) | *k == key )
    {
      out.push( *k );
    }
    else
    {
      let valid : Vec< &str > = table.iter().map( | ( k, _ ) | *k ).collect();
      return Err( format!(
        "unknown column key `{key}`; valid keys: {}",
        valid.join( ", " )
      ) );
    }
  }
  if out.is_empty()
  {
    let valid : Vec< &str > = table.iter().map( | ( k, _ ) | *k ).collect();
    return Err( format!( "no column keys given; valid keys: {}", valid.join( ", " ) ) );
  }
  Ok( out )
}
