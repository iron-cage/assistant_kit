//! Mechanical enforcement of the unsafe-containment invariant.
//!
//! ## Specification References
//!
//! - `docs/invariant/001_unsafe_containment.md` — all `unsafe` confined to `src/ffi.rs`
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | unsafe01 | Scan the code of every `src/*.rs` for the `unsafe` token | Only `ffi.rs` contains it |
//! | unsafe02 | `ffi.rs` still contains `unsafe` | Guards against a vacuous unsafe01 |
//! | unsafe03 | Every `unsafe` block in `ffi.rs` has a `SAFETY:` comment | Count of `SAFETY:` ≥ count of `unsafe {` |

use std::fs;
use std::path::{ Path, PathBuf };

/// The one module permitted to contain `unsafe`.
const FFI_MODULE : &str = "ffi.rs";

fn src_dir() -> PathBuf
{
  Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src" )
}

/// Every `.rs` file directly under `src/`, sorted for deterministic failure output.
fn source_files() -> Vec< PathBuf >
{
  let dir = src_dir();
  let mut files : Vec< PathBuf > = fs::read_dir( &dir )
    .unwrap_or_else( | e | panic!( "cannot read {}: {e}", dir.display() ) )
    .map( | entry | entry.expect( "cannot enumerate src/" ).path() )
    .filter( | path | path.extension().is_some_and( | ext | ext == "rs" ) )
    .collect();
  files.sort();
  assert!( !files.is_empty(), "no .rs files found under src/ — the scan would be vacuous" );
  files
}

/// Everything on `line` up to a `//`, which is where a comment starts and code
/// stops.
///
/// The invariant is about `unsafe` *code*; prose that names the invariant is not
/// a violation of it, and a crate whose whole reason for existing is hand-rolled
/// FFI cannot document itself under a rule that forbids writing the word. The
/// caller asserts the two assumptions that make this exact rather than
/// approximate for these sources — see [`assert_strippable`].
fn code_of( line : &str ) -> &str
{
  line.split_once( "//" ).map_or( line, | ( code, _ ) | code )
}

/// Fail if `content` uses a construct [`code_of`] would mis-handle.
///
/// Two would: a block comment (`/*`), whose body [`code_of`] does not remove, and
/// a `//` inside a string literal, which it would treat as the start of a comment
/// and strip the rest of the line — the one direction that could hide a real
/// `unsafe`. Neither appears in this crate today. Asserting that keeps it an
/// enforced precondition rather than a silent assumption that decays.
fn assert_strippable( name : &str, content : &str )
{
  assert!(
    !content.contains( "/*" ),
    "{name} uses a block comment, which the line-comment scan in this test does not \
     strip — extend `code_of` before introducing one",
  );
  for ( number, line ) in content.lines().enumerate()
  {
    let quotes_before_comment = code_of( line ).matches( '"' ).count();
    assert!(
      quotes_before_comment % 2 == 0,
      "{name}:{} has `//` inside a string literal, which `code_of` would strip as a \
       comment — extend it before introducing one",
      number + 1,
    );
  }
}

/// unsafe01: no module outside `ffi.rs` contains `unsafe` in its code.
///
/// The check is on the token, not on the compiler's `unsafe_code` lint: the
/// workspace denies that lint, so a per-block `#[ allow ]` anywhere in the crate
/// would satisfy the compiler while defeating the containment this test exists
/// to hold. That attribute carries the token itself, so scanning code catches the
/// escape hatch and the block it would open, while leaving prose about the
/// invariant — in `lib.rs`, and in every module doc that explains why the FFI is
/// where it is — free to name what it describes.
#[ test ]
fn unsafe01_unsafe_only_in_ffi()
{
  let mut offenders = Vec::new();

  for path in source_files()
  {
    let name = path.file_name().and_then( | n | n.to_str() ).expect( "non-UTF-8 source filename" );
    if name == FFI_MODULE
    {
      continue;
    }
    let content = fs::read_to_string( &path )
      .unwrap_or_else( | e | panic!( "cannot read {}: {e}", path.display() ) );
    assert_strippable( name, &content );
    if content.lines().any( | line | code_of( line ).contains( "unsafe" ) )
    {
      offenders.push( name.to_string() );
    }
  }

  assert!(
    offenders.is_empty(),
    "unsafe containment broken: `unsafe` appears in the code of {} module(s) outside \
     {FFI_MODULE}: {:?}\n\
     See docs/invariant/001_unsafe_containment.md — move the FFI call into {FFI_MODULE} \
     and expose a safe wrapper.",
    offenders.len(),
    offenders,
  );
}

/// unsafe02: `ffi.rs` still contains `unsafe` in its code.
///
/// Without this, unsafe01 would keep passing if `ffi.rs` were renamed or the FFI
/// removed — a green suite that no longer checks anything. Code rather than raw
/// content for the same reason: `ffi.rs` opens with a module doc explaining the
/// containment, so a prose-level check would stay green through the removal of
/// every block it describes.
#[ test ]
fn unsafe02_ffi_module_is_the_unsafe_one()
{
  let path = src_dir().join( FFI_MODULE );
  let content = fs::read_to_string( &path )
    .unwrap_or_else( | e | panic!( "cannot read {}: {e}", path.display() ) );
  assert!(
    content.lines().any( | line | code_of( line ).contains( "unsafe" ) ),
    "{FFI_MODULE} contains no `unsafe` — either the FFI moved (update this test and \
     docs/invariant/001_unsafe_containment.md) or unsafe01 is now vacuous",
  );
}

/// unsafe03: every `unsafe` block in `ffi.rs` carries a `SAFETY:` comment.
///
/// The workspace sets `undocumented_unsafe_blocks = "deny"`, so this duplicates a
/// compiler check — deliberately. The lint is a workspace setting that a future
/// crate-level `allow` could switch off; this assertion is local to the crate
/// that actually holds the unsafe code.
#[ test ]
fn unsafe03_every_unsafe_block_documented()
{
  let path = src_dir().join( FFI_MODULE );
  let content = fs::read_to_string( &path )
    .unwrap_or_else( | e | panic!( "cannot read {}: {e}", path.display() ) );

  let blocks = content.matches( "unsafe {" ).count();
  let documented = content.matches( "SAFETY:" ).count();

  assert!(
    documented >= blocks,
    "{FFI_MODULE} has {blocks} `unsafe {{` block(s) but only {documented} `SAFETY:` \
     comment(s) — every unsafe block must state why it is sound",
  );
}
