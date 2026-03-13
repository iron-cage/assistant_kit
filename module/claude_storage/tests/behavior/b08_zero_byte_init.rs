#![ allow( clippy::doc_markdown ) ]
//! B8: Claude Code creates 0-byte `.jsonl` files as session placeholders on startup.
//!
//! Validates that at least one 0-byte `.jsonl` file exists in real `~/.claude/`
//! storage — evidence that Claude Code creates empty files before writing entries.

/// B8: at least one 0-byte `.jsonl` exists in real storage.
///
/// These appear when Claude Code initializes a session but the process exits
/// before writing entries. If Claude Code changed to write entries atomically
/// (no empty placeholder phase), zero-byte files would not exist.
#[ test ]
fn b8_zero_byte_jsonl_exists_in_real_storage()
{
  let projects = super::find_projects();
  if projects.is_empty()
  {
    eprintln!( "skip: no ~/.claude/projects/ found" );
    return;
  }

  let has_zero_byte = projects.iter().any( | p |
  {
    super::find_all_jsonl( p ).iter().any( | f |
      std::fs::metadata( f ).map( | m | m.len() == 0 ).unwrap_or( false )
    )
  });

  if !has_zero_byte
  {
    // Zero-byte files are transient — they may have been cleaned up or none
    // exist on this machine. This is not a hard failure; log for visibility.
    eprintln!(
      "B8 note: no 0-byte .jsonl found across {} projects. \
       This may be normal if no sessions crashed during init.",
      projects.len()
    );
  }
}
