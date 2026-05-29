//! Code fence stripping utility for the `--strip-fences` output mode.

/// Strip the outermost markdown code fence pair from `stdout`.
///
/// Finds the first and last lines starting with ` ``` ` (after optional leading whitespace).
/// If both exist and are distinct lines, returns the content between them (preserving
/// the original trailing-newline state). If fewer than two fences exist, returns `stdout`
/// unchanged.
#[ must_use ]
#[ inline ]
pub fn strip_fences( stdout : &str ) -> String
{
  let lines : Vec< &str > = stdout.lines().collect();
  let first_fence = lines.iter().position( | l | l.trim_start().starts_with( "```" ) );
  let last_fence  = lines.iter().rposition( | l | l.trim_start().starts_with( "```" ) );
  match ( first_fence, last_fence )
  {
    ( Some( f ), Some( l ) ) if f < l =>
    {
      let body = lines[ f + 1 .. l ].join( "\n" );
      if stdout.ends_with( '\n' ) { format!( "{body}\n" ) } else { body }
    }
    _ => stdout.to_string(),
  }
}
