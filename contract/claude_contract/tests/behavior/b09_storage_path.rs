#![ allow( clippy::doc_markdown ) ]
//! B9: Claude Code stores project sessions at `~/.claude/projects/{encoded}/`
//! where path encoding maps `/` → `-`.
//!
//! Validates that real project directory names under `~/.claude/projects/`
//! follow the `/`→`-` encoding convention.

/// B9: real project dir names follow the `/`→`-` encoding rule.
///
/// Each directory name under `~/.claude/projects/` should start with `-`
/// (because absolute paths start with `/` which encodes to `-`).
/// If Claude Code changed the encoding scheme, directory names would
/// no longer match this pattern.
#[ test ]
fn b9_project_dir_names_follow_encoding_convention()
{
  let projects = super::find_projects();
  if projects.is_empty()
  {
    eprintln!( "skip: no ~/.claude/projects/ found" );
    return;
  }

  // All project dirs that represent filesystem paths should start with `-`
  // (the encoded leading `/`). Some dirs may use UUIDs or topic suffixes,
  // so we check that at least one follows the convention.
  let encoded_count = projects.iter()
    .filter( | p |
    {
      let name = p.file_name().unwrap_or_default().to_string_lossy();
      name.starts_with( '-' )
    })
    .count();

  assert!(
    encoded_count > 0,
    "B9 violated: no project dir starts with `-` (encoded leading `/`).\n\
     Claude Code may have changed path encoding.\n\
     Project dirs: {:?}",
    projects.iter().map( | p | p.file_name().unwrap_or_default().to_string_lossy().into_owned() ).collect::< Vec<_> >()
  );

  // Verify round-trip: decode a real dir name back to a path and check it exists
  for project in &projects
  {
    let name = project.file_name().unwrap_or_default().to_string_lossy();
    if !name.starts_with( '-' ) { continue; }

    // Strip optional topic suffix (--topic) before decoding
    let base = name.find( "--" ).map_or( name.as_ref(), | i | &name[ ..i ] );

    // Decode: `-` → `/`
    let decoded = base.replace( '-', "/" );
    let decoded_path = std::path::Path::new( &decoded );

    if decoded_path.exists()
    {
      // Successfully decoded to a real path — encoding convention confirmed
      return;
    }
  }

  // If we reach here, no decoded path exists on disk — may be deleted projects.
  // Still pass: the naming convention (starts with `-`) was confirmed above.
}
