//! Terse `.projects` overview rendering — flat recency table and directory tree.
//!
//! Renders the `detail::projects` view in two layouts over one shared row set:
//! a flat table sorted by recency (default) and a directory tree
//! (`show_tree::1`) that makes the cwd-bucket nature of a "project" visible.
//! The full `detail::sessions` view is rendered by `super::projects` itself and
//! never passes through this module.
//!
//! ## Known Pitfalls
//!
//! ### Absent decoded paths must be marked, not silently rendered
//!
//! **Issue**: A project's decoded display path may name a directory that no
//! longer exists — deleted `-commit` scratch directories are the common case.
//! The `_`-versus-`/` decode ambiguity is only resolvable by consulting the real
//! filesystem, so once the directory is gone the rendered path is a guess
//! (`docs/cli/command/07_projects.md`, issue-029/031/035).
//!
//! **Solution**: Rows whose expanded path is not a directory carry a `⚠ gone`
//! marker. The path is still shown — it is the only identifier the storage key
//! provides — but it is never presented as verified.
//!
//! **Prevention**: Any future column derived from a decoded path must consult
//! [`path_is_present`] rather than assuming the path resolves.

use core::fmt::Write as FmtWrite;
use std::path::PathBuf;
use std::time::SystemTime;

use super::projects::format_relative_time;

// ─── constants ─────────────────────────────────────────────────────────────

/// Placeholder for a numeric column whose value is zero.
const ZERO_CELL : &str = "·";

/// Marker for a project whose decoded path is absent from disk.
const GONE_MARKER : &str = "⚠ gone";

/// Gutter marker on the project matching the current working directory.
const CWD_MARKER : &str = "▸";

/// Column gap, in spaces.
const GAP : &str = "  ";

// ─── row set ───────────────────────────────────────────────────────────────

/// One project's row in a terse overview.
///
/// Built by `super::projects` from the same aggregation that feeds the
/// `detail::sessions` view — no additional filesystem traversal.
#[ derive( Debug ) ]
pub( super ) struct OverviewRow
{
  /// Decoded project path as displayed (`~/…` form, or absolute when the
  /// decode did not land under `$HOME`).
  pub display_path  : String,
  /// Root conversations in this project.
  pub conversations : usize,
  /// Agent sessions across those conversations.
  pub agents        : usize,
  /// Most recent non-zero-byte session mtime — the sort key.
  pub last_mtime    : SystemTime,
}

// ─── path helpers ──────────────────────────────────────────────────────────

/// Expand a `~/…` display path back to an absolute path.
///
/// Paths that are already absolute, or that cannot be expanded because `HOME`
/// is unset, are returned unchanged.
fn expand_display_path( display : &str ) -> PathBuf
{
  if let Some( rest ) = display.strip_prefix( "~/" )
  {
    if let Some( home ) = std::env::var_os( "HOME" )
    {
      return PathBuf::from( home ).join( rest );
    }
  }
  PathBuf::from( display )
}

/// Whether a decoded display path resolves to a real directory.
fn path_is_present( display : &str ) -> bool
{
  expand_display_path( display ).is_dir()
}

/// Split a display path into components, preserving a leading `/` as its own
/// component so absolute and `~`-rooted paths never share a tree root.
fn split_components( display : &str ) -> Vec< &str >
{
  let rest = display.split( '/' ).filter( | s | !s.is_empty() );
  if display.starts_with( '/' )
  {
    core::iter::once( "/" ).chain( rest ).collect()
  }
  else
  {
    rest.collect()
  }
}

// ─── shared cell construction ──────────────────────────────────────────────

/// Pad `text` on the right to `width` display characters.
fn pad_right( text : &str, width : usize ) -> String
{
  let len = text.chars().count();
  let mut padded = text.to_string();
  for _ in len .. width
  {
    padded.push( ' ' );
  }
  padded
}

/// Pad `text` on the left to `width` display characters.
fn pad_left( text : &str, width : usize ) -> String
{
  let len = text.chars().count();
  let mut padded = String::new();
  for _ in len .. width
  {
    padded.push( ' ' );
  }
  padded.push_str( text );
  padded
}

/// Widest entry in a column, in display characters.
fn column_width< 'a >( cells : impl Iterator< Item = &'a str > ) -> usize
{
  cells.map( | c | c.chars().count() ).max().unwrap_or( 0 )
}

/// `N conv` / `N ag` cell, collapsing zero to [`ZERO_CELL`].
fn count_cell( value : usize, unit : &str ) -> String
{
  if value == 0
  {
    ZERO_CELL.to_string()
  }
  else
  {
    format!( "{value} {unit}" )
  }
}

/// Totals line: `10 projects · 76 conversations · 103 agents`.
fn summary_line( rows : &[ OverviewRow ] ) -> String
{
  let projects : usize = rows.len();
  let conversations : usize = rows.iter().map( | r | r.conversations ).sum();
  let agents : usize = rows.iter().map( | r | r.agents ).sum();

  let p_noun = if projects == 1 { "project" } else { "projects" };
  let c_noun = if conversations == 1 { "conversation" } else { "conversations" };
  let mut line = format!( "{projects} {p_noun} · {conversations} {c_noun}" );
  if agents > 0
  {
    let a_noun = if agents == 1 { "agent" } else { "agents" };
    write!( line, " · {agents} {a_noun}" ).expect( "writing to String cannot fail" );
  }
  line
}

/// Gutter marker for the row matching the current working directory.
fn gutter( display : &str, cwd : Option< &std::path::Path > ) -> &'static str
{
  let is_cwd = cwd.is_some_and( | c | expand_display_path( display ) == c );
  if is_cwd { CWD_MARKER } else { " " }
}

// ─── flat layout ───────────────────────────────────────────────────────────

/// Render the flat recency table — the `detail::projects` default.
///
/// Rows arrive already sorted by `last_mtime` descending; that order is
/// preserved so the leading `LAST` column reads as a sorted axis.
///
/// Paths are printed in full rather than factored against a shared prefix:
/// a project path is the command's primary output and must stay directly
/// copyable and greppable. Prefix factoring is what `show_tree::1` is for —
/// there the nesting carries the shared prefix without truncating any row.
pub( super ) fn render_flat( rows : &[ OverviewRow ] ) -> String
{
  let mut out = String::new();
  writeln!( out, "{}", summary_line( rows ) ).expect( "writing to String cannot fail" );

  if rows.is_empty()
  {
    return out;
  }
  writeln!( out ).expect( "writing to String cannot fail" );

  let cwd = std::env::current_dir().ok();
  let any_gone = rows.iter().any( | r | !path_is_present( &r.display_path ) );

  let cells : Vec< ( &'static str, String, String, String, String, String ) > = rows
    .iter()
    .map( | r |
    {
      let gone = if path_is_present( &r.display_path ) { String::new() }
                 else { GONE_MARKER.to_string() };
      (
        gutter( &r.display_path, cwd.as_deref() ),
        format_relative_time( r.last_mtime ),
        count_cell( r.conversations, "conv" ),
        count_cell( r.agents, "ag" ),
        gone,
        r.display_path.clone(),
      )
    } )
    .collect();

  let last_w = core::cmp::max( 4, column_width( cells.iter().map( | c | c.1.as_str() ) ) );
  let conv_w = core::cmp::max( 4, column_width( cells.iter().map( | c | c.2.as_str() ) ) );
  let ag_w   = core::cmp::max( 6, column_width( cells.iter().map( | c | c.3.as_str() ) ) );
  let gone_w = if any_gone { GONE_MARKER.chars().count() } else { 0 };

  // Header — column names occupy the same widths as their cells.
  let mut header = format!(
    "  {}{GAP}{}{GAP}{}",
    pad_right( "LAST", last_w ),
    pad_left( "CONV", conv_w ),
    pad_left( "AGENTS", ag_w ),
  );
  if any_gone
  {
    header.push_str( GAP );
    header.push_str( &pad_right( "", gone_w ) );
  }
  header.push_str( GAP );
  header.push_str( "PROJECT" );
  writeln!( out, "{}", header.trim_end() ).expect( "writing to String cannot fail" );

  for ( marker, last, conv, ag, gone, label ) in &cells
  {
    let mut line = format!(
      "{marker} {}{GAP}{}{GAP}{}",
      pad_right( last, last_w ),
      pad_left( conv, conv_w ),
      pad_left( ag, ag_w ),
    );
    if any_gone
    {
      line.push_str( GAP );
      line.push_str( &pad_right( gone, gone_w ) );
    }
    line.push_str( GAP );
    line.push_str( label );
    writeln!( out, "{}", line.trim_end() ).expect( "writing to String cannot fail" );
  }

  out
}

// ─── tree layout ───────────────────────────────────────────────────────────

/// One node in the project directory tree.
///
/// `row` is `None` for a purely structural node — a directory that is an
/// ancestor of two or more projects but is not itself a project.
#[ derive( Debug ) ]
struct TreeNode
{
  label    : String,
  row      : Option< usize >,
  children : Vec< TreeNode >,
}

impl TreeNode
{
  fn new( label : String ) -> Self
  {
    Self { label, row : None, children : Vec::new() }
  }

  /// Fetch (or create) the child carrying `label`.
  fn child_mut( &mut self, label : &str ) -> &mut Self
  {
    if let Some( idx ) = self.children.iter().position( | c | c.label == label )
    {
      return &mut self.children[ idx ];
    }
    self.children.push( Self::new( label.to_string() ) );
    self.children.last_mut().expect( "child pushed immediately above" )
  }
}

/// Fold single-child structural nodes into their child.
///
/// Without this, `~/pro/lib/yrd_core/assistant_kit` would render as four nested
/// levels of which three carry no project. Collapsing yields one node labelled
/// with the whole run.
fn collapse( node : &mut TreeNode )
{
  while node.row.is_none() && node.children.len() == 1
  {
    let child = node.children.remove( 0 );
    node.label = if node.label == "/" { format!( "/{}", child.label ) }
                 else { format!( "{}/{}", node.label, child.label ) };
    node.row = child.row;
    node.children = child.children;
  }
  for child in &mut node.children
  {
    collapse( child );
  }
}

/// Most recent mtime anywhere in a subtree — the sort key at every level.
fn subtree_mtime( node : &TreeNode, rows : &[ OverviewRow ] ) -> SystemTime
{
  node
    .children
    .iter()
    .map( | c | subtree_mtime( c, rows ) )
    .chain( node.row.map( | i | rows[ i ].last_mtime ) )
    .max()
    .unwrap_or( SystemTime::UNIX_EPOCH )
}

/// Sort every level by subtree recency descending, matching the flat layout.
fn sort_tree( nodes : &mut [ TreeNode ], rows : &[ OverviewRow ] )
{
  nodes.sort_by_key( | n | core::cmp::Reverse( subtree_mtime( n, rows ) ) );
  for node in nodes.iter_mut()
  {
    sort_tree( &mut node.children, rows );
  }
}

/// Build the collapsed, recency-sorted forest for `rows`.
fn build_tree( rows : &[ OverviewRow ] ) -> Vec< TreeNode >
{
  let mut root = TreeNode::new( String::new() );
  for ( index, row ) in rows.iter().enumerate()
  {
    let mut node = &mut root;
    for part in split_components( &row.display_path )
    {
      node = node.child_mut( part );
    }
    node.row = Some( index );
  }
  let mut tops = root.children;
  for top in &mut tops
  {
    collapse( top );
  }
  sort_tree( &mut tops, rows );
  tops
}

/// Depth-first walk emitting `(label_with_connectors, row_index)` pairs.
fn flatten_tree( node : &TreeNode, prefix : &str, last : Option< bool >,
                 out : &mut Vec< ( String, Option< usize > ) > )
{
  let ( label, child_prefix ) = match last
  {
    // Top-level node: no connector, children start at column zero.
    None => ( node.label.clone(), String::new() ),
    Some( true )  => ( format!( "{prefix}└─ {}", node.label ), format!( "{prefix}   " ) ),
    Some( false ) => ( format!( "{prefix}├─ {}", node.label ), format!( "{prefix}│  " ) ),
  };
  out.push( ( label, node.row ) );

  let count = node.children.len();
  for ( i, child ) in node.children.iter().enumerate()
  {
    flatten_tree( child, &child_prefix, Some( i + 1 == count ), out );
  }
}

/// Render the directory-tree layout — `detail::projects show_tree::1`.
///
/// Structural nodes (directories that are ancestors of several projects but
/// hold no sessions themselves) render as a label with empty columns.
pub( super ) fn render_tree( rows : &[ OverviewRow ] ) -> String
{
  let mut out = String::new();
  writeln!( out, "{}", summary_line( rows ) ).expect( "writing to String cannot fail" );

  if rows.is_empty()
  {
    return out;
  }
  writeln!( out ).expect( "writing to String cannot fail" );

  let mut lines : Vec< ( String, Option< usize > ) > = Vec::new();
  for top in &build_tree( rows )
  {
    flatten_tree( top, "", None, &mut lines );
  }

  let cwd = std::env::current_dir().ok();
  let any_gone = rows.iter().any( | r | !path_is_present( &r.display_path ) );

  let label_w = column_width( lines.iter().map( | ( l, _ ) | l.as_str() ) );
  let conv_cells : Vec< String > = lines
    .iter()
    .map( | ( _, row ) | row.map_or_else( String::new, | i | count_cell( rows[ i ].conversations, "conv" ) ) )
    .collect();
  let ag_cells : Vec< String > = lines
    .iter()
    .map( | ( _, row ) | row.map_or_else( String::new, | i | count_cell( rows[ i ].agents, "ag" ) ) )
    .collect();
  let conv_w = column_width( conv_cells.iter().map( String::as_str ) );
  let ag_w   = column_width( ag_cells.iter().map( String::as_str ) );
  let gone_w = if any_gone { GONE_MARKER.chars().count() } else { 0 };

  for ( index, ( label, row ) ) in lines.iter().enumerate()
  {
    let marker = row.map_or( " ", | i | gutter( &rows[ i ].display_path, cwd.as_deref() ) );
    let gone = match row
    {
      Some( i ) if !path_is_present( &rows[ *i ].display_path ) => GONE_MARKER,
      _ => "",
    };
    let time = row.map_or_else( String::new, | i | format_relative_time( rows[ i ].last_mtime ) );

    let mut line = format!( "{marker} {}", pad_right( label, label_w ) );
    if any_gone
    {
      line.push_str( GAP );
      line.push_str( &pad_right( gone, gone_w ) );
    }
    line.push_str( GAP );
    line.push_str( &pad_left( &conv_cells[ index ], conv_w ) );
    line.push_str( GAP );
    line.push_str( &pad_left( &ag_cells[ index ], ag_w ) );
    line.push_str( GAP );
    line.push_str( &time );
    writeln!( out, "{}", line.trim_end() ).expect( "writing to String cannot fail" );
  }

  out
}
