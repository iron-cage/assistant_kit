//! Tests for `.projects` path encoding/decoding — `decode_project_display` behavior.
//!
//! ## Coverage
//!
//! Bug reproducers for path display correctness when project paths contain
//! underscores or hyphen-prefixed directory components:
//!
//! | ID    | Issue | What it covers                                                |
//! |-------|-------|---------------------------------------------------------------|
//! | IT-24 | 030   | Hyphen-prefixed topic dirs preserved in display path          |
//! | IT-23 | 029   | Underscore-named dirs decoded correctly (not split on `/`)    |
//! | IT-25 | 031   | `scope::under` excludes underscore-suffix sibling modules     |
//! | IT-26 | 032   | `scope::relevant` excludes underscore-prefix sibling modules  |
//! | IT-60 | 035   | Topic path shown even when topic dir absent from disk (T01)   |
//! | IT-61 | 035   | Topic path shown when topic dir present on disk (T02)         |
//! | IT-62 | 035   | Default-topic path shown when absent from disk (T03)          |
//! | IT-63 | 035   | Base path shown correctly with no topic suffix (T04)          |
//! | IT-64 | 035   | Double-topic storage key shows both topic components (T05)    |
//! | IT-65 | BUG-003 | Display resolves a dot-prefixed mid-path component           |
//! | IT-67 | BUG-003 | `scope::under` excludes a dot-prefixed similar-named sibling  |
//! | IT-68 | BUG-003 | `scope::relevant` excludes a dot-prefixed similar-named sibling |
//! | IT-69 | BUG-509 | `scope::local` excludes a real nested dot-prefixed project    |
//! | IT-70 | BUG-510 | `scope::local` excludes a real nested project named a single special character |
//! | IT-71 | BUG-510 | `scope::local` excludes a real nested project when the anchor's own name ends in a special character |
//! | IT-72 | BUG-511 | `scope::local` excludes a nested project whose name starts with an arbitrary special character |
//! | IT-73 | BUG-511 | `scope::under` excludes a sibling whose name embeds a double-hyphen topic-boundary shape |
//! | IT-74 | BUG-511 | `scope::local` excludes a nested project whose name starts with two consecutive special characters |
//! | IT-75 | BUG-512 | `scope::under` excludes a sibling carrying a synthetic (non-real-dir) topic suffix |
//! | IT-76 | BUG-512 | `scope::relevant` excludes a sibling carrying a synthetic (non-real-dir) topic suffix |
//! | IT-77 | BUG-512 | `.count target::projects scope::under` excludes the same topic-suffixed sibling |
//! | IT-78 | BUG-513 | `scope::local` excludes a real nested project past `encode_path`'s 200-char truncation boundary |
//! | IT-79 | BUG-514 | `scope::under` conservatively includes an ambiguous session when a multi-byte-Unicode sibling wins `best_partial`'s byte-length tie-break over the real anchor |
//! | IT-80 | BUG-516 | `scope::under` excludes a real sibling whose name textually extends the anchor's name, past `encode_path`'s 200-char truncation boundary |
//! | IT-81 | BUG-516 | `scope::under` excludes the same extension-named sibling when it also carries a synthetic topic suffix |
//! | IT-82 | BUG-516 | `scope::local` excludes the same extension-named sibling when it also carries a synthetic topic suffix |
//! | IT-83 | BUG-516 | `scope::around` excludes the same extension-named sibling when it also carries a synthetic topic suffix |
//! | IT-84 | BUG-517 | `scope::relevant` includes a real ancestor whose own encoding independently exceeds the 200-char truncation boundary |
//! | IT-85 | BUG-517 | `scope::under` includes a real descendant whose own encoding independently exceeds the 200-char truncation boundary |
//! | IT-86 | BUG-518 | `scope::under` includes a genuinely ambiguous full-match tie, excludes a tie where no candidate relates to the anchor |
//!
//! Note: IT-60..IT-64 follow IT-59 (`scope::around` tests in `projects_scope_around_test.rs`).
//! IT-27..IT-30 were already allocated in `tests/docs/cli/command/007_projects.md`
//! for unrelated tests, so the next available block was used here. IT-65 follows
//! IT-64 (next free ID in the shared `.projects` IT-N sequence). IT-66 is
//! allocated to `scope_under_finds_project_with_dot_prefixed_path` in
//! `projects_scope_test.rs`; IT-67/IT-68 continue the shared sequence from
//! there — combinatorial gap between IT-25/IT-26 (sibling exclusion, no
//! dot-prefix) and IT-65/IT-66 (dot-prefix, no sibling collision). IT-69
//! renames what was originally added as `it_27_...` (a numbering collision
//! with the IT-27..IT-30 reservation above — see git history); IT-70/IT-71
//! continue the sequence for two further `scope::local` bypass shapes found
//! independently during BUG-509's own MAAV re-verification. IT-72/IT-73/IT-74
//! continue the sequence for three further bypass shapes (arbitrary special
//! character, mid-component sibling collision, consecutive leading specials)
//! found during a Tier 5 MAAV Cycle's Round 6 re-verification of BUG-509/510's
//! own fix, all sharing one root cause and fixed together as BUG-511. IT-75
//! /IT-76/IT-77 continue the sequence for BUG-512 (a real sibling wrongly
//! admitted whenever its storage key carries an unresolvable, purely-metadata
//! `--topic` suffix, across `matches_under`/`matches_relevant`/the shared
//! resolver's `.count` call site) and IT-78 for BUG-513 (a real nested
//! project past `encode_path`'s own 200-character truncation boundary,
//! wrongly admitted under `scope::local`) — both found during the same MAAV
//! Cycle's Round 7 re-verification of BUG-511's own just-landed fix. IT-79
//! continues the sequence for BUG-514 (`walk_fs`'s ambiguity tie-break
//! compared candidate `PathBuf`s by raw UTF-8 byte length — a multi-byte
//! Unicode sibling colliding on `encode_component_piece` with a real anchor
//! could out-weigh it purely by byte footprint, not genuine specificity) and
//! IT-80..IT-83 for BUG-516 (a real sibling whose name is a literal prefix-
//! extension of a shorter real match's name — e.g. `ancX8` vs
//! `ancX8-extra-<tail>` — never became a forward-matching candidate at all
//! once its own full piece could not fit within `remaining` past
//! `encode_path`'s 200-char truncation boundary, so BUG-515's single-anchor
//! `search_encoded_subtree` fallback stayed anchored at the shorter match's
//! own dead-end subtree instead), both found empirically, via direct test
//! execution rather than hand-tracing alone, while implementing the fix for
//! BUG-514/515 during the same MAAV Cycle's Round 8 re-verification. IT-84
//! /IT-85 continue the sequence for BUG-517 (single-sided 200-char
//! truncation is provably safe — appending path components only ever
//! lengthens an encoding — but once BOTH an ancestor and descendant
//! independently cross the boundary, each gets a hash suffix derived from
//! its OWN distinct full path string, breaking the ancestor/descendant
//! fast-rejects' literal-prefix assumption even for a genuine relationship)
//! and IT-86 for BUG-518 (`walk_fs`'s Full-match arm had no tie detection at
//! all, unlike the Partial-vs-Partial case; the first fix attempt collapsed
//! a detected tie to the tied candidates' shared parent, which is itself too
//! coarse — a caller's ancestor check against that shared parent can be
//! satisfied even when NEITHER individual tied candidate relates to the
//! query), both found during the same MAAV Cycle's Round 9 re-verification
//! of BUG-514/515/516's own fixes.
#![ cfg( unix ) ]

mod common;

use tempfile::TempDir;

fn stdout( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

fn stderr( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stderr ).into_owned()
}

fn assert_exit( out : &std::process::Output, code : i32 )
{
  assert_eq!(
    out.status.code().unwrap_or( -1 ),
    code,
    "expected exit {code}, got {:?}; stderr: {}",
    out.status.code(),
    stderr( out )
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Decode Display — Hyphen-Prefixed Topic Directory (issue-030)
//
// Root Cause: decode_project_display stripped the `--topic` suffix before
// decoding, so `-...-src--default-topic` displayed as `src` even when
// `-default_topic` is a real filesystem directory (the actual working directory).
//
// Why Not Caught: All prior tests used simple session paths with no
// hyphen-prefixed directory components. No test path ended in `/-default_topic`
// or any other `-name` component that the topic strip discarded.
//
// Fix Applied: decode_project_display now tries to extend the decoded base path
// by each `--topic` component as a real filesystem directory. The display uses
// the longest existing path prefix. So `-...-src--default-topic` displays as
// `src/-default_topic` when that directory exists on disk.
//
// Prevention: Test that sessions created from a hyphen-prefixed working
// directory (e.g. `src/-default_topic`) display the full path in the header.
//
// Pitfall: After Fix(issue-035) the existence check in the topic-extension loop
// was removed — topics are now unconditionally joined. The IT-60..IT-62 tests
// verify the absent-dir case. The only remaining existence check is on the base
// path decode (used for underscore/slash ambiguity resolution), which is correct.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(issue-030)
fn it_24_decode_display_includes_hyphen_prefixed_topic_dir()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  // Project path ending in a hyphen-prefixed directory (the real CWD pattern)
  let project = root.path().join( "src" ).join( "-default_topic" );
  // Create the actual directories so the existence check passes
  std::fs::create_dir_all( &project ).expect( "create src/-default_topic dir" );
  common::write_path_project_session( &storage_root, &project, "session-topic-dir-test", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )

    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "-default_topic" ),
    "display path must include hyphen-prefixed topic dir '-default_topic'; got:\n{s}"
  );
  assert!(
    !s.lines().any( | l | l.trim_end().ends_with( "src:" ) ),
    "display path must NOT be truncated to 'src' when '-default_topic' exists; got:\n{s}"
  );
  assert!( s.contains( "session-topic-dir-test" ), "session ID must appear; got:\n{s}" );
}

// ─────────────────────────────────────────────────────────────────────────────
// Decode Display — Underscore Directory Names (issue-029)
//
// Root Cause: encode_path converts `_` → `-` (lossy). The heuristic decoder
// defaults to path separator (`/`) for all unrecognized `-` boundaries, so
// underscore-named directories like `my_project` decode to `wip/core` in the
// display path.
//
// Why Not Caught: All prior tests used simple single-word project dir names
// (e.g., "proj", "agent_filter_proj"). No test path had underscore-named
// intermediate components like `my_project/project`.
//
// Fix Applied: decode_project_display now checks whether the heuristic-decoded
// path exists on the filesystem. If not, it falls back to decode_path_via_fs
// which walks the real directory tree, choosing `/` vs `_` at each `-` boundary
// by calling is_dir() on the candidate path prefix.
//
// Prevention: Test project paths that contain underscore-named intermediate
// directories. The test must also create those directories on disk so the
// filesystem walk can verify existence.
//
// Pitfall: decode_path_via_fs requires the project directory to exist at display
// time. Deleted or remote projects fall back to the raw encoded storage dir name.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(issue-029)
fn it_23_decode_display_preserves_underscore_named_dirs()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  // Project path with underscore-named directory component
  let project = root.path().join( "my_project" ).join( "myproject" );
  // Create the actual directories so filesystem-guided decode can verify existence
  std::fs::create_dir_all( &project ).expect( "create project dir with underscore component" );
  common::write_path_project_session( &storage_root, &project, "session-underscore-test", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )

    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "my_project" ),
    "display path must preserve underscore: 'my_project' not 'wip/core'; got:\n{s}"
  );
  assert!(
    !s.lines().any( | l | l.contains( "wip/core" ) ),
    "display path must NOT split my_project into wip/core; got:\n{s}"
  );
  assert!( s.contains( "session-underscore-test" ), "session ID must appear; got:\n{s}" );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::under — Sibling Module Exclusion (issue-031)
//
// Root Cause: encode_path maps both `_` and `/` to `-`. The `under` predicate
// used string starts_with on encoded forms, so a sibling `base_extra/` passed
// the same prefix check as a child `base/sub/`: both encoded forms start with
// the `base-` prefix. String comparison cannot distinguish path-separator `/`
// from underscore `_` in encoded form.
//
// Why Not Caught: All prior scope::under tests used simple single-word base dirs
// (e.g., "workspace"). No test had a sibling whose name was the base name plus
// an underscore suffix, simulating real module naming like `claude_storage_core`
// next to `claude_storage`.
//
// Fix Applied: Two-stage predicate. String prefix is fast-reject only. Candidates
// passing string check (not exact) are verified via decode_path_via_fs +
// Path::starts_with. Path::starts_with is component-wise: Path("/x/base_extra")
// does NOT start_with Path("/x/base") even though string "/x/base_extra"
// starts_with "/x/base".
//
// Prevention: Always test scope::under with a sibling whose encoded form shares the
// base encoded prefix (underscore-suffix sibling). Create all directories on disk
// so decode_path_via_fs can resolve them correctly.
//
// Pitfall: decode_path_via_fs returns None for deleted/remote paths. The fixed
// predicate uses unwrap_or(true) (conservative include) to avoid silently dropping
// sessions from projects that existed when the session was created.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(issue-031)
fn it_25_scope_under_excludes_underscore_named_sibling()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // Simulate: base = module/claude_storage
  //           child = module/claude_storage/sub  (under base → must appear)
  //           sibling = module/base_extra         (NOT under base → must not appear)
  let base    = root.path().join( "base" );
  let child   = base.join( "sub" );
  let sibling = root.path().join( "base_extra" );

  // Directories must exist on disk: decode_path_via_fs uses is_dir() to walk.
  // Without real dirs the walker returns None → unwrap_or(true) includes all.
  std::fs::create_dir_all( &child ).expect( "create child dir" );
  std::fs::create_dir_all( &sibling ).expect( "create sibling dir" );

  common::write_path_project_session( &storage_root, &child,   "session-it25-child",   2 );
  common::write_path_project_session( &storage_root, &sibling, "session-it25-sibling", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", base.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-it25-child" ),
    "must contain session-it25-child (child base/sub is under base); got:\n{s}"
  );
  assert!(
    !s.contains( "session-it25-sibling" ),
    "must NOT contain session-it25-sibling (sibling base_extra is NOT under base); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::relevant — Sibling Module Exclusion (issue-032)
//
// Root Cause: encode_path maps both `_` and `/` to `-`. The `relevant` scope
// predicate (is_relevant_encoded) uses string starts_with: encoded_base
// starts_with(dir_name + "-"). A sibling `base/` passed the same prefix check
// as a real ancestor: if base_path is `/tmp/base_extra`, the project at `/tmp/base`
// (encoded `-tmp-base`) matched because `-tmp-base-extra` starts with `-tmp-base-`.
// String comparison cannot distinguish `/` from `_` in encoded form.
//
// Why Not Caught: All prior scope::relevant tests used simple ancestor chains
// (e.g., /a, /a/b, /a/b/c). No test had a sibling whose encoded name was a
// prefix of the current path's encoded form — the `base` vs `base_extra` pattern.
//
// Fix Applied: Two-stage predicate in the `"relevant"` arm of project_matches.
// is_relevant_encoded is fast-reject only. Exact encoded match returns true.
// Prefix-match candidates are verified via decode_path_via_fs +
// base_path.starts_with(decoded_path). Path::starts_with is component-wise:
// Path("/x/base_extra").starts_with(Path("/x/base")) → false.
//
// Prevention: Always test scope::relevant with a project whose name is a
// string prefix of the current path's name (underscore-suffix sibling).
// Create all directories on disk so decode_path_via_fs can resolve them.
//
// Pitfall: Same as issue-031 fix for scope::under — decode_path_via_fs returns
// None for deleted/remote paths; is_none_or provides conservative include.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(issue-032)
fn it_26_scope_relevant_excludes_underscore_named_sibling()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // Simulate: sibling = base      (NOT an ancestor of base_extra despite prefix match)
  //           target  = base_extra (current path; encoded -...-base-extra)
  // /base encoded to `-...-base`; `/base_extra` encoded to `-...-base-extra`.
  // Without fix: is_relevant_encoded returns true because encoded_base starts
  // with (dir_name + "-"), making scope::relevant include /base as a false ancestor.
  let sibling = root.path().join( "base" );
  let target  = root.path().join( "base_extra" );

  // Directories must exist on disk: decode_path_via_fs uses is_dir() to walk.
  // Without real dirs the walker returns None → is_none_or(true) includes all.
  std::fs::create_dir_all( &sibling ).expect( "create sibling dir" );
  std::fs::create_dir_all( &target ).expect( "create target dir" );

  common::write_path_project_session( &storage_root, &sibling, "session-it26-sibling", 2 );
  common::write_path_project_session( &storage_root, &target,  "session-it26-target",  2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::relevant" )
    .arg( format!( "path::{}", target.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-it26-target" ),
    "must contain session-it26-target (current project at base_extra); got:\n{s}"
  );
  assert!(
    !s.contains( "session-it26-sibling" ),
    "must NOT contain session-it26-sibling (/base is NOT an ancestor of /base_extra); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::local — Nested Dot-Prefixed Project False-Match (BUG-509)
//
// Root Cause: project_matches's "local" arm used a naive string check —
// dir_name == encoded_base || dir_name.starts_with(format!("{encoded_base}--"))
// — with no filesystem verification, unlike scope::under/relevant
// (matches_under/matches_relevant, both fixed by BUG-003). A REAL nested
// project whose path component starts with a non-alphanumeric character (e.g.
// `.venv`) encodes to exactly `{encoded_base}--venv` (encode_path's `--`
// topic-boundary marker for a component whose first char is normalized away),
// so it satisfies the naive starts_with("{encoded_base}--") check even though
// it is a genuine, separate, nested project — not a topic-suffix alias of the
// anchor itself.
//
// Why Not Caught: BUG-003's fix (issue-031/032) added filesystem verification
// to scope::under and scope::relevant, but scope::local's own inline check in
// project_matches was never updated to match — no test exercised scope::local
// with a real, dot-prefixed nested project directory.
//
// Fix Applied: New matches_local() function mirrors matches_under's shape:
// exact match returns true; a "--"-shaped candidate is verified via
// decode_path_via_fs; if it resolves to a REAL path, only match when that path
// EXACTLY equals base_path (scope::local means the anchor itself, never a
// descendant); an unresolvable candidate (genuine synthetic topic tag, no real
// directory) is conservatively included, same fallback philosophy as
// matches_under/matches_relevant.
//
// Prevention: Always test scope::local with a real nested project directory
// whose name is non-alphanumeric-prefixed (dot-prefixed child). Create all
// directories on disk so decode_path_via_fs can resolve them.
//
// Pitfall: decode_path_via_fs returns None for deleted/remote paths. The fixed
// predicate uses map_or(true, ...) (conservative include) for unresolvable
// candidates, exactly like matches_under/matches_relevant — do not special-case
// "local" to be stricter than that fallback philosophy.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-509)
fn it_69_scope_local_excludes_nested_dot_prefixed_project()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // Simulate: anchor = module/claude_storage        (scope::local target)
  //           victim = module/claude_storage/.venv  (REAL nested project;
  //                     encodes to {encoded_anchor}--venv — collides with the
  //                     naive starts_with("{encoded_base}--") check)
  let anchor = root.path().join( "anchor" );
  let victim = anchor.join( ".venv" );

  // Directories must exist on disk: decode_path_via_fs uses is_dir() to walk.
  // Without real dirs the walker returns None → map_or(true, ...) includes all.
  std::fs::create_dir_all( &anchor ).expect( "create anchor dir" );
  std::fs::create_dir_all( &victim ).expect( "create victim dir" );

  common::write_path_project_session( &storage_root, &anchor, "session-it69-anchor", 2 );
  common::write_path_project_session( &storage_root, &victim, "session-it69-victim", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", anchor.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-it69-anchor" ),
    "must contain session-it69-anchor (anchor is the scope::local target); got:\n{s}"
  );
  assert!(
    !s.contains( "session-it69-victim" ),
    "must NOT contain session-it69-victim (anchor/.venv is a distinct nested project, not a topic alias of anchor); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Decode Display — Absent Topic Directory (issue-035)
//
// Root Cause: `decode_project_display` checks `candidate.exists()` before
// extending the decoded base path with a topic component. When the topic
// directory (`-commit`) is absent from disk, the extension is skipped and
// the function returns only the base path. Violates the display-path
// invariant: the storage key records the CWD at session start; current
// filesystem state is irrelevant to session attribution.
//
// Why Not Caught: The issue-030 fix was tested only with extant topic
// directories (`create_dir_all` before running). No test exercised the case
// where the topic directory had been deleted, so the guard was never
// challenged.
//
// Fix Applied: Remove the `candidate.exists()` guard in the topic-extension
// loop — always join unconditionally. The storage key is the authoritative
// CWD record; disk state at query time must not affect session attribution.
//
// Prevention: Every bug_reproducer for `decode_project_display` must include
// both an extant-dir variant and an absent-dir variant to exercise both
// branches.
//
// Pitfall: Do NOT remove the `h.exists()` check on the base path decode —
// that guard enables the filesystem-guided fallback for underscore/slash
// ambiguity and is correct. Only the topic-loop guard (`if candidate.exists()`
// inside `for &topic in &parts[1..]`) is the bug.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(issue-035)
fn projects_shows_topic_path_when_topic_dir_absent()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  // Base project dir must exist so decode_project_display base-path decode succeeds.
  let project = root.path().join( "myproject" );
  std::fs::create_dir_all( &project ).expect( "create project base dir" );
  // Build storage key for the topic project (base + --commit suffix).
  let encoded_base = claude_storage_core::encode_path( &project ).expect( "encode project path" );
  let topic_project_id = format!( "{encoded_base}--commit" );
  // Write session into the topic project dir. Do NOT create -commit dir on disk.
  common::write_test_session( &storage_root, &topic_project_id, "session-t01-absent-commit", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )

    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "/-commit" ),
    "display path must include topic '/-commit' even when dir is absent from disk; got:\n{s}"
  );
  assert!(
    !s.lines().any( | l | l.trim_end().ends_with( "myproject:" ) ),
    "display path must NOT be truncated to 'myproject:' when topic dir is absent; got:\n{s}"
  );
  assert!( s.contains( "session-t01-absent-commit" ), "session must appear in output; got:\n{s}" );
}

#[ test ]
fn projects_shows_topic_path_when_topic_dir_present()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  // Create base AND topic dir on disk — non-regression: behavior must match T01.
  let project   = root.path().join( "myproject" );
  let topic_dir = project.join( "-commit" );
  std::fs::create_dir_all( &topic_dir ).expect( "create myproject/-commit dir" );
  // Build storage key.
  let encoded_base = claude_storage_core::encode_path( &project ).expect( "encode project path" );
  let topic_project_id = format!( "{encoded_base}--commit" );
  common::write_test_session( &storage_root, &topic_project_id, "session-t02-present-commit", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )

    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "/-commit" ),
    "display path must include topic '/-commit' when dir is present on disk; got:\n{s}"
  );
  assert!( s.contains( "session-t02-present-commit" ), "session must appear; got:\n{s}" );
}

#[ test ]
// bug_reproducer(issue-035)
fn projects_shows_default_topic_path_when_topic_dir_absent()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "myproject" );
  std::fs::create_dir_all( &project ).expect( "create project base dir" );
  let encoded_base = claude_storage_core::encode_path( &project ).expect( "encode project path" );
  // "--default-topic" suffix: topic component "default-topic" → dir "-default_topic".
  let topic_project_id = format!( "{encoded_base}--default-topic" );
  // Write session. Do NOT create -default_topic dir on disk.
  common::write_test_session( &storage_root, &topic_project_id, "session-t03-absent-default-topic", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )

    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "/-default_topic" ),
    "display path must include '/-default_topic' even when dir is absent; got:\n{s}"
  );
  assert!(
    s.contains( "session-t03-absent-default-topic" ),
    "session must appear in output; got:\n{s}"
  );
}

#[ test ]
fn projects_shows_base_path_with_no_topic()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "myproject" );
  std::fs::create_dir_all( &project ).expect( "create project dir" );
  common::write_path_project_session( &storage_root, &project, "session-t04-no-topic", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )

    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-t04-no-topic" ), "session must appear; got:\n{s}" );
  // No topic suffix in storage key — path must not include any topic component.
  assert!(
    !s.contains( "/-commit" ),
    "no topic in storage key — must not show /-commit; got:\n{s}"
  );
  assert!(
    !s.contains( "/-default_topic" ),
    "no topic in storage key — must not show /-default_topic; got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Decode Display — Double-Topic Storage Key (issue-035, T05)
//
// Root Cause: decode_project_display decodes the full storage key in one
// call to decode_storage_base (Fix issue-035 / BUG-003). For a storage key
// `{base}--default-topic--commit`, the naive `claude_storage_core::decode_path`
// heuristic chains BOTH `--` markers into successive hyphen-prefixed display
// components on its own: base → base/-default_topic → base/-default_topic/-commit.
//
// Why Not Caught: All issue-035 tests used single-topic suffixes only
// (`--commit` or `--default-topic`). Multiple `--` separators in one key
// were not exercised.
//
// Fix Applied: No code change needed — Fix(issue-035) already handles this
// correctly. This test guards against regression.
//
// Prevention: Whenever adding topic-extension tests, include a multi-topic
// variant to verify the naive heuristic's `--`-chaining handles multiple
// topic markers in one key.
//
// Pitfall: Claude Code could in principle create `{base}--default-topic--commit`
// for a session from `base/-default_topic/-commit`. Both topic components must
// appear in the display path.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
fn projects_shows_both_topic_components_for_double_topic_key()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "myproject" );
  std::fs::create_dir_all( &project ).expect( "create project base dir" );
  let encoded_base = claude_storage_core::encode_path( &project ).expect( "encode project path" );
  // Storage key with two topic components: "--default-topic--commit".
  let topic_project_id = format!( "{encoded_base}--default-topic--commit" );
  // Write session. Do NOT create topic dirs on disk — absence must not drop either topic.
  common::write_test_session( &storage_root, &topic_project_id, "session-t05-double-topic", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )

    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "/-default_topic" ),
    "display path must include first topic '/-default_topic'; got:\n{s}"
  );
  assert!(
    s.contains( "/-commit" ),
    "display path must include second topic '/-commit'; got:\n{s}"
  );
  assert!( s.contains( "session-t05-double-topic" ), "session must appear in output; got:\n{s}" );
}

// ─────────────────────────────────────────────────────────────────────────────
// Decode Display — Dot-Prefixed Path Component (BUG-003)
//
// Root Cause: Fix(BUG-366) generalized encode_path's substitution from
// `_`-only to the full non-alphanumeric character class, so a dot-prefixed
// path component (e.g. `.hidden_base`) now produces the identical `--`
// marker as a genuine `--topic` suffix. `walk_fs` had no DFS option for the
// empty split piece this produces at a `--` boundary, so it silently dropped
// the leading special character and could never resolve the real directory.
//
// Why Not Caught: IT-23/IT-24 cover underscore-named and hyphen-prefixed
// components, but neither used a project path with a `.`-prefixed MID-PATH
// component that is not itself a topic suffix.
//
// Fix Applied: `walk_fs` gained option C — on an empty piece, commit the
// current segment first, then try `.`, `_`, `-` as the candidate first
// character of the next component, accepting whichever exists on disk.
//
// Prevention: Any project path built under a dot/underscore/hyphen-prefixed
// directory now exercises this path.
//
// Pitfall: Use an EXPLICIT dot-prefixed directory name, not
// `tempfile::TempDir`'s own incidental `.tmpXXXXXX` naming — the test's
// intent must be self-evident and independent of the temp-file crate's
// internal naming scheme.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-003)
fn it_65_decode_display_resolves_dot_prefixed_path_component()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  // Explicit dot-prefixed mid-path component — not a topic suffix.
  let hidden_base = root.path().join( ".hidden_base" );
  let project = hidden_base.join( "child" );
  std::fs::create_dir_all( &project ).expect( "create .hidden_base/child dir" );
  common::write_path_project_session( &storage_root, &project, "session-it65-dot-prefixed", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "/.hidden_base/child" ),
    "display path must resolve the dot-prefixed component; got:\n{s}"
  );
  assert!( s.contains( "session-it65-dot-prefixed" ), "session must appear; got:\n{s}" );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::under / scope::relevant — dot-prefixed sibling with a similar name
// (BUG-003 combinatorial gap: it_25/it_26 cover sibling-exclusion without a
// dot-prefix; it_65 covers dot-prefix without a sibling-name collision. This
// pair combines both — `.my_config` vs `.my_config_extra` — since neither
// prior test exercises the interaction between walk_fs option C (dot-prefix
// resolution) and the sibling-exclusion fs verification in matches_under /
// matches_relevant. Directories must exist on disk, same as it_25/it_26:
// decode_path_via_fs uses is_dir()/exists() to walk; without real dirs the
// walker returns None and the conservative-include fallback would wrongly
// include the sibling.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-003)
fn it_67_scope_under_excludes_dot_prefixed_similar_named_sibling()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // base is `.my_config`; sibling `.my_config_extra` must NOT be treated as
  // nested under base even though it shares the encoded `-my-config` prefix.
  let base    = root.path().join( ".my_config" );
  let sibling = root.path().join( ".my_config_extra" );
  let child   = base.join( "child" );

  std::fs::create_dir_all( &child ).expect( "create .my_config/child dir" );
  std::fs::create_dir_all( &sibling ).expect( "create .my_config_extra dir" );

  common::write_path_project_session( &storage_root, &base,    "session-it67-base",    2 );
  common::write_path_project_session( &storage_root, &sibling, "session-it67-sibling", 2 );
  common::write_path_project_session( &storage_root, &child,   "session-it67-child",   2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", base.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-it67-base" ),  "must include base itself; got:\n{s}" );
  assert!( s.contains( "session-it67-child" ), "must include child of base; got:\n{s}" );
  assert!(
    !s.contains( "session-it67-sibling" ),
    "scope::under must EXCLUDE dot-prefixed sibling `.my_config_extra`; got:\n{s}"
  );
}

#[ test ]
// bug_reproducer(BUG-003)
fn it_68_scope_relevant_excludes_dot_prefixed_similar_named_sibling()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // cwd is deep inside `.my_config_extra`; sibling `.my_config` must NOT be
  // treated as an ancestor even though its encoded prefix matches.
  let sibling   = root.path().join( ".my_config" );
  let base_path = root.path().join( ".my_config_extra" ).join( "sub" );
  let unrelated = root.path().join( "other" );

  std::fs::create_dir_all( &sibling ).expect( "create .my_config dir" );
  std::fs::create_dir_all( &base_path ).expect( "create .my_config_extra/sub dir" );
  std::fs::create_dir_all( &unrelated ).expect( "create other dir" );

  common::write_path_project_session( &storage_root, &sibling,   "session-it68-sibling",   2 );
  common::write_path_project_session( &storage_root, &base_path, "session-it68-base",      2 );
  common::write_path_project_session( &storage_root, &unrelated, "session-it68-unrelated", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::relevant" )
    .arg( format!( "path::{}", base_path.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-it68-base" ), "must include base project; got:\n{s}" );
  assert!(
    !s.contains( "session-it68-sibling" ),
    "scope::relevant must EXCLUDE dot-prefixed sibling `.my_config` despite shared encoded prefix; got:\n{s}"
  );
  assert!( !s.contains( "session-it68-unrelated" ), "must exclude unrelated project; got:\n{s}" );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::local — Single-Special-Character Nested Directory Bypass (BUG-510)
//
// Root Cause: when a real nested project's ENTIRE path component is a single
// non-alphanumeric character (e.g. `_` or `-`), encode_path collapses it to a
// bare `--` marker with NOTHING after it. decode_path_via_fs's walk_fs then
// splits the encoded name into two consecutive empty pieces at that boundary.
// walk_fs's candidate-resolution loop tried `.` as the first candidate
// character regardless of whether anything followed it; when the remaining
// piece is itself empty, this produces a bare `.` segment, and
// `base.join(".")` trivially `.exists()` (Path's Components iterator drops
// non-leading `.` components, so `base.join(".") == base` under PartialEq) —
// so the walk always "resolved" back to the anchor itself, never reaching the
// real single-char-named directory.
//
// Why Not Caught: BUG-509's own regression test (IT-69) only exercised a
// multi-character topic-suffix component (`.venv`); no existing test used a
// nested directory whose name was ONLY a single special character.
//
// Fix Applied: walk_fs now skips the bare `.` candidate specifically when the
// remaining piece is empty (that shape can never correspond to a real,
// encodable directory name), letting the `_`/`-` candidates run and correctly
// resolve to the real single-char-named directory instead.
//
// Prevention: When testing filesystem-walk decoders, always include a
// same-shape variant where the ENTIRE component collapses to nothing after
// its leading-character marker is stripped, not just multi-character ones.
//
// Pitfall: do not special-case this in matches_local — the fix belongs in
// walk_fs, since matches_under/matches_relevant share the exact same hole.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-510)
fn it_70_scope_local_excludes_single_char_nested_project()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // victim's entire path component is a single underscore — encode_path
  // collapses it to a bare `--` marker with nothing after it.
  let anchor = root.path().join( "anchor" );
  let victim = anchor.join( "_" );

  std::fs::create_dir_all( &anchor ).expect( "create anchor dir" );
  std::fs::create_dir_all( &victim ).expect( "create victim dir" );

  common::write_path_project_session( &storage_root, &anchor, "session-it70-anchor", 2 );
  common::write_path_project_session( &storage_root, &victim, "session-it70-victim", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", anchor.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-it70-anchor" ),
    "must contain session-it70-anchor (anchor is the scope::local target); got:\n{s}"
  );
  assert!(
    !s.contains( "session-it70-victim" ),
    "must NOT contain session-it70-victim (anchor/_ is a distinct nested project, not the anchor itself); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::local — Trailing-Special-Character Anchor Name Bypass (BUG-510)
//
// Root Cause: encode_path produces the identical `--` marker whether a
// component's FIRST character is special (the case walk_fs's Option C
// already handled) or the PRECEDING component's LAST character is special —
// the component's own trailing normalized `-` and the ordinary `-` separator
// before the next component concatenate into the same two bytes. When the
// scope::local ANCHOR's own final path component ends in a special character
// (e.g. a directory literally named `myproject-`), its encoded form itself
// ends in a stray `-`; the fast-reject `starts_with("{eb}--")` then
// false-positives on an unrelated but genuinely nested real project, and
// walk_fs's decode (which only ever tried attaching the special character to
// the START of the NEXT piece, never the END of the current one) could never
// reconstruct the anchor's own trailing character to disprove the match —
// falling through to the conservative-include fallback.
//
// Why Not Caught: every existing scope::local regression test used an anchor
// name containing only alphanumeric characters; none varied the ANCHOR's own
// trailing character, only the nested victim's.
//
// Fix Applied: walk_fs now also tries appending a trailing special character
// to the accumulated segment (committing it as a directory) before
// continuing the walk fresh from the next piece — trying both "the marker
// belongs to what comes after" and "the marker belongs to what came before"
// interpretations, exactly as the module's own doc comment already commits
// to doing via filesystem verification for other ambiguous cases.
//
// Prevention: scope::local regression tests must vary the ANCHOR's own
// trailing character (`.`, `_`, literal `-`), not only the nested victim's
// leading character — both sides of a `--` boundary are ambiguous.
//
// Pitfall: matches_under/matches_relevant share the same walk_fs machinery
// and the same hole; the fix lives in walk_fs so all three benefit.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-510)
fn it_71_scope_local_excludes_trailing_special_char_anchor_nested_project()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // anchor's OWN name ends in a trailing literal hyphen; victim is a REAL,
  // separate nested project (dot-prefixed) one level under it.
  let anchor = root.path().join( "myproject-" );
  let victim = anchor.join( ".venv" );

  std::fs::create_dir_all( &anchor ).expect( "create anchor dir" );
  std::fs::create_dir_all( &victim ).expect( "create victim dir" );

  common::write_path_project_session( &storage_root, &anchor, "session-it71-anchor", 2 );
  common::write_path_project_session( &storage_root, &victim, "session-it71-victim", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", anchor.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-it71-anchor" ),
    "must contain session-it71-anchor (anchor is the scope::local target); got:\n{s}"
  );
  assert!(
    !s.contains( "session-it71-victim" ),
    "must NOT contain session-it71-victim (myproject-/.venv is a distinct nested project, not the anchor itself); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::local — Arbitrary Non-Alphanumeric Character Bypass (BUG-511)
//
// Root Cause: walk_fs's old candidate-character set for resolving a `--`
// boundary was hardcoded to `.`, `_`, and literal `-` — but encode_path (and
// docs/invariant/001_path_encoding.md's documented contract) normalizes
// EVERY non-alphanumeric byte identically, not just those three. A nested
// project whose leading path component starts with any OTHER special
// character (e.g. `!`) produces the same `--` marker but could never be
// resolved by the fixed candidate set, falling through to the
// conservative-include fallback that matches_local relies on to distinguish
// a real nested project from a topic-suffix alias.
//
// Why Not Caught: every existing scope::local bypass regression (IT-69/70/71)
// used `.` or `_` or a literal `-` as the special character; none tried an
// arbitrary other non-alphanumeric byte.
//
// Fix Applied: walk_fs (and decode_path_via_fs) no longer guess a candidate
// character at all — they enumerate REAL directory entries and forward-encode
// each one's name via encode_component_piece (the same function encode_path
// itself calls), matching by construction regardless of which
// non-alphanumeric byte produced a hyphen run.
//
// Prevention: scope::local bypass regressions must include at least one case
// using a special character outside the old `.`/`_`/`-` candidate set, to
// guard against ever reintroducing a finite candidate list.
//
// Pitfall: matches_under/matches_relevant share the same walk_fs machinery
// and the same hole; the fix lives in walk_fs so all three benefit.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-511)
fn it_72_scope_local_excludes_nested_project_with_arbitrary_special_leading_char()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // victim's leading path component starts with `!` — a non-alphanumeric
  // byte outside walk_fs's old hardcoded `.`/`_`/`-` candidate set.
  let anchor = root.path().join( "anchor72" );
  let victim = anchor.join( "!important" );

  std::fs::create_dir_all( &anchor ).expect( "create anchor dir" );
  std::fs::create_dir_all( &victim ).expect( "create victim dir" );

  common::write_path_project_session( &storage_root, &anchor, "session-it72-anchor", 2 );
  common::write_path_project_session( &storage_root, &victim, "session-it72-victim", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", anchor.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-it72-anchor" ),
    "must contain session-it72-anchor (anchor is the scope::local target); got:\n{s}"
  );
  assert!(
    !s.contains( "session-it72-victim" ),
    "must NOT contain session-it72-victim (anchor72/!important is a distinct nested project, not the anchor itself); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::under — Mid-Component Sibling Collision Bypass (BUG-511)
//
// Root Cause: walk_fs's old options only ever tried to resolve a `--`
// boundary as EITHER a component boundary (splitting into two components)
// OR a single trailing/leading special character — never "this whole run of
// hyphens is literal characters embedded inside one real component that is
// never split at all". A sibling directory whose own literal name extends
// the anchor's encoded prefix with an embedded `--` (e.g. anchor `sibfoo73`
// next to sibling `sibfoo73--extra`) passed the fast `starts_with("{eb}-")`
// pre-filter but could never be correctly decoded back to its own real
// (non-nested) path, falling through to the conservative-include fallback
// and letting an unrelated sibling's sessions leak into scope::under.
//
// Why Not Caught: every existing scope::under sibling-exclusion regression
// (IT-25) used a plain underscore/hyphen suffix on the sibling's name, never
// a sibling name containing an embedded double-hyphen identical to the
// anchor's own topic-boundary marker shape.
//
// Fix Applied: walk_fs now enumerates real directory entries and
// forward-encodes each one's own name via encode_component_piece, so the
// sibling's OWN full name (including its embedded `--`) is matched as a
// single real component rather than guessed at — resolving to the sibling's
// own real, non-nested path.
//
// Prevention: scope::under/relevant sibling-exclusion regressions must
// include a sibling name containing an embedded run of hyphens shaped like a
// real topic-boundary marker, not only a single extra suffix character.
//
// Pitfall: this is the same walk_fs machinery matches_local/matches_relevant
// use; the fix lives in walk_fs so all three benefit.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-511)
fn it_73_scope_under_excludes_sibling_with_embedded_double_hyphen()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // sibling's own literal name embeds a double hyphen identical in shape to
  // a real topic-boundary marker — a SIBLING of anchor, never nested under it.
  let parent = root.path().join( "parent73" );
  let anchor = parent.join( "sibfoo73" );
  let sibling = parent.join( "sibfoo73--extra" );

  std::fs::create_dir_all( &anchor ).expect( "create anchor dir" );
  std::fs::create_dir_all( &sibling ).expect( "create sibling dir" );

  common::write_path_project_session( &storage_root, &anchor, "session-it73-anchor", 2 );
  common::write_path_project_session( &storage_root, &sibling, "session-it73-sibling", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", anchor.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-it73-anchor" ),
    "must contain session-it73-anchor (anchor is the scope::under target); got:\n{s}"
  );
  assert!(
    !s.contains( "session-it73-sibling" ),
    "must NOT contain session-it73-sibling (parent73/sibfoo73--extra is a sibling, not a descendant of parent73/sibfoo73); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::local — Consecutive Leading Special Characters Bypass (BUG-511)
//
// Root Cause: walk_fs's options C/D each substituted only ONE candidate
// character per empty split-piece / `--` boundary. A nested project whose
// leading path component starts with TWO OR MORE consecutive special
// characters (e.g. a real directory literally named `--nested`) needs two
// substitutions resolved together at the same boundary — something no
// combination of single-substitution options could ever produce — so the
// walk always fell through to the conservative-include fallback.
//
// Why Not Caught: every existing scope::local bypass regression (IT-69/70/71)
// used exactly ONE special character at the ambiguous boundary; none tried
// two or more consecutive special characters in the same component.
//
// Fix Applied: walk_fs now forward-encodes each real directory entry's own
// name via encode_component_piece and matches the resulting byte sequence
// (of any length) against what remains to decode, rather than substituting
// one guessed character at a time — resolving any run length by construction.
//
// Prevention: scope::local bypass regressions must include a case with two
// or more consecutive special characters at the same component boundary, to
// guard against ever reintroducing a single-substitution-per-boundary design.
//
// Pitfall: matches_under/matches_relevant share the same walk_fs machinery
// and the same hole; the fix lives in walk_fs so all three benefit.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-511)
fn it_74_scope_local_excludes_nested_project_with_consecutive_leading_specials()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // victim's entire leading path component is two literal hyphens followed
  // by ordinary characters — two consecutive special characters at once.
  let anchor = root.path().join( "anchor74" );
  let victim = anchor.join( "--nested" );

  std::fs::create_dir_all( &anchor ).expect( "create anchor dir" );
  std::fs::create_dir_all( &victim ).expect( "create victim dir" );

  common::write_path_project_session( &storage_root, &anchor, "session-it74-anchor", 2 );
  common::write_path_project_session( &storage_root, &victim, "session-it74-victim", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", anchor.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-it74-anchor" ),
    "must contain session-it74-anchor (anchor is the scope::local target); got:\n{s}"
  );
  assert!(
    !s.contains( "session-it74-victim" ),
    "must NOT contain session-it74-victim (anchor74/--nested is a distinct nested project, not the anchor itself); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::under — Synthetic Topic-Suffix Sibling Leak (BUG-512)
//
// Root Cause: matches_under's decode_path_via_fs(dir_name).map_or(true, ...)
// fallback fired on EVERY unresolvable decode, including one where walk_fs had
// already fully verified a REAL sibling base directory along the way and only
// failed to consume a trailing, non-real `--topic` suffix (pure metadata, a
// normal documented case per docs/invariant/001_path_encoding.md). The old
// Option<PathBuf> return type could not tell "found a real but different
// path" apart from "nothing real at all" — both collapsed to None, both
// triggered blind inclusion.
//
// Why Not Caught: every existing scope::under sibling-exclusion regression
// (IT-25, IT-67, IT-73) used a PLAIN sibling storage key with no topic
// suffix — none combined a real sibling with an unresolvable `--topic` tag,
// exactly the untested combination scope.rs's own pre-fix Fix(BUG-003)
// comment on matches_under named explicitly.
//
// Fix Applied: decode_path_via_fs now returns FsDecodeOutcome (Full/Partial/
// NotFound) instead of Option<PathBuf>. matches_under uses a Partial result's
// verified real path for the same starts_with(base_path) check a Full match
// uses, plus an ancestor-of(base_path) disjunct that also protects a
// deleted-project-with-real-shallow-ancestor case (see
// cli_cmd_show_test.rs's T02 and scope.rs's own Fix(BUG-512) doc comment for
// why the disjunct is needed); only NotFound still conservatively includes.
//
// Prevention: any scope::under/scope::relevant sibling-exclusion regression
// going forward should include a topic-suffixed variant, not only a plain one.
//
// Pitfall: matches_relevant shares the identical root cause and fix shape,
// mirrored in the ancestor direction — see IT-76.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-512)
fn it_75_scope_under_excludes_sibling_with_synthetic_topic_suffix()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let anchor  = root.path().join( "anchor75" );
  let child   = anchor.join( "sub" );
  let sibling = root.path().join( "anchor75_extra" ); // real dir; NOT under anchor

  std::fs::create_dir_all( &child ).expect( "create anchor/sub dir" );
  std::fs::create_dir_all( &sibling ).expect( "create sibling dir" );
  // Deliberately do NOT create any "synthtopic75" directory anywhere on disk —
  // the topic tag below is pure metadata (documented normal case).

  common::write_path_project_session( &storage_root, &anchor,  "session-it75-anchor",  2 );
  common::write_path_project_session( &storage_root, &child,   "session-it75-child",   2 );
  // Control: plain sibling, no topic suffix — already covered by IT-25/IT-73,
  // repeated here so a future regression in EITHER shape fails THIS test too.
  common::write_path_project_session( &storage_root, &sibling, "session-it75-sibling-plain", 2 );
  // Attack: same sibling, WITH a synthetic (non-real-dir) topic suffix.
  let sibling_encoded = claude_storage_core::encode_path( &sibling ).expect( "encode sibling path" );
  let sibling_topic_id = format!( "{sibling_encoded}--synthtopic75" );
  common::write_test_session( &storage_root, &sibling_topic_id, "session-it75-sibling-topic", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", anchor.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-it75-anchor" ), "must include anchor itself; got:\n{s}" );
  assert!( s.contains( "session-it75-child" ),  "must include child under anchor; got:\n{s}" );
  assert!(
    !s.contains( "session-it75-sibling-plain" ),
    "CONTROL: plain sibling (no topic) must be excluded; got:\n{s}"
  );
  assert!(
    !s.contains( "session-it75-sibling-topic" ),
    "must NOT contain session-it75-sibling-topic (anchor75_extra is a sibling of anchor75, \
     never nested under it, regardless of any topic suffix on its storage key); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::relevant — Synthetic Topic-Suffix Sibling Leak (BUG-512, mirror)
//
// Root Cause/Why Not Caught/Fix Applied: identical to IT-75, mirrored in the
// ancestor direction (matches_relevant's base_path.starts_with(p) check
// instead of matches_under's p.starts_with(base_path)).
//
// Prevention: see IT-75.
//
// Also guards BUG-516: this exact name shape (sibling "it76siba" is a
// literal PREFIX of target "it76siba_extra", and target — the LONGER name —
// is itself the `path::` anchor) caught a regression in Fix(BUG-516)'s own
// first draft: an extension-sibling heuristic added for a different defect
// (see IT-80) let target win a tie-break against sibling purely because
// target's piece textually extends sibling's, even though target was never
// a valid decode of sibling_topic_id's own encoded storage key at all. See
// scope.rs's own Fix(BUG-516) doc comment, Pitfall 2.
//
// Pitfall: sibling/target names chosen so the encoded-prefix collision from
// the `_` vs `/` lossy-encoding ambiguity is present (sibling "it76siba" is a
// literal string-prefix of target "it76siba_extra") — this is what makes the
// naive is_relevant_encoded string check alone insufficient and requires the
// filesystem-verified decode this test exercises.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-512)
fn it_76_scope_relevant_excludes_sibling_with_synthetic_topic_suffix()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // sibling ("it76siba") is NOT an ancestor of target ("it76siba_extra") despite
  // the encoded-prefix collision from the `_` vs `/` lossy-encoding ambiguity.
  let sibling = root.path().join( "it76siba" );
  let target  = root.path().join( "it76siba_extra" );

  std::fs::create_dir_all( &sibling ).expect( "create sibling dir" );
  std::fs::create_dir_all( &target ).expect( "create target dir" );

  common::write_path_project_session( &storage_root, &target, "session-it76-target", 2 );
  // Control: plain sibling, no topic suffix.
  common::write_path_project_session( &storage_root, &sibling, "session-it76-sibling-plain", 2 );
  // Attack: same sibling, WITH a synthetic (non-real-dir) topic suffix.
  let sibling_encoded = claude_storage_core::encode_path( &sibling ).expect( "encode sibling path" );
  let sibling_topic_id = format!( "{sibling_encoded}--synthtopic76" );
  common::write_test_session( &storage_root, &sibling_topic_id, "session-it76-sibling-topic", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::relevant" )
    .arg( format!( "path::{}", target.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-it76-target" ), "must include target project itself; got:\n{s}" );
  assert!(
    !s.contains( "session-it76-sibling-plain" ),
    "CONTROL: plain sibling (no topic) must be excluded; got:\n{s}"
  );
  assert!(
    !s.contains( "session-it76-sibling-topic" ),
    "must NOT contain session-it76-sibling-topic (it76siba is NOT an ancestor of \
     it76siba_extra, regardless of any topic suffix on its storage key); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// .count target::projects scope::under — Synthetic Topic-Suffix Sibling Leak
// (BUG-512, cross-command spot-check)
//
// Root Cause/Why Not Caught/Fix Applied: identical to IT-75 — `.count`'s
// `target::projects scope::under` shares the exact same
// `resolve_scoped_projects`/`project_matches`/`matches_under` resolver
// `.projects` uses, so this confirms the fix is not `.projects`-specific.
//
// Prevention: see IT-75.
//
// Pitfall: `.count`'s own `path::` is a storage-root override, not a scope
// anchor (docs/cli/param/09_path.md) — anchor scope via cwd instead, and pass
// the storage root straight to CLAUDE_STORAGE_ROOT (no nested `.claude/`).
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-512)
fn it_77_count_target_projects_scope_under_excludes_sibling_with_synthetic_topic_suffix()
{
  let root = TempDir::new().unwrap();

  let anchor  = root.path().join( "anchor77" );
  let sibling = root.path().join( "anchor77_extra" );
  std::fs::create_dir_all( &anchor ).expect( "create anchor dir" );
  std::fs::create_dir_all( &sibling ).expect( "create sibling dir" );

  common::write_path_project_session( root.path(), &anchor,  "session-it77-anchor",  2 );
  common::write_path_project_session( root.path(), &sibling, "session-it77-sibling-plain", 2 );
  let sibling_encoded = claude_storage_core::encode_path( &sibling ).expect( "encode sibling path" );
  let sibling_topic_id = format!( "{sibling_encoded}--synthtopic77" );
  common::write_test_session( root.path(), &sibling_topic_id, "session-it77-sibling-topic", 2 );

  let under_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &anchor )
    .arg( ".count" )
    .arg( "target::projects" )
    .arg( "scope::under" )
    .output()
    .unwrap();

  assert_exit( &under_out, 0 );
  let out_str = stdout( &under_out );
  let n : usize = out_str.trim().parse().unwrap_or_else( |_| panic!(
    "count output must be a bare integer; got: '{out_str}'"
  ));
  // Expect exactly 1 (the anchor project itself). A count > 1 means at least
  // one sibling project (plain or topic-tagged) leaked into scope::under.
  assert_eq!(
    n, 1,
    ".count target::projects scope::under must count only the anchor project \
     itself (1); a count > 1 means a sibling project leaked in (got {n})"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::local — 200-Character Truncation Boundary Leak (BUG-513)
//
// Root Cause: walk_fs's per-component matching only ever knew
// encode_component_piece — the PER-COMPONENT half of encode_path's rule. It
// had no knowledge of encode_path's separate, outer 200-character truncation
// + djb2-hash-suffix step (claude_storage_core/src/path.rs:234-246), applied
// ONCE to the fully concatenated string. Any real project whose encoded path
// crossed that boundary could never be matched by incremental per-component
// consumption, so decode_path_via_fs unconditionally returned unresolvable
// for it, and the old map_or(true, ...) fallback admitted it into
// scope::local's results even though it was a distinct, deeply nested
// project, not the anchor itself.
//
// Why Not Caught: every existing scope::local bypass regression (IT-69/70/
// 71/72/74) used a SHORT nested path, well under 200 characters once
// encoded — none exercised encode_path's separate truncation step at all,
// which lives entirely outside walk_fs's per-component knowledge.
//
// Fix Applied: walk_fs now also tries the REAL encode_path (truncation
// included) directly against real candidate paths once incremental
// per-component matching gets stuck (new search_encoded_subtree helper) —
// this recognizes a truncated-hash match by asking the actual production
// encoder, never by reimplementing its truncation rule locally.
//
// Prevention: any future scope::local/scope::under/scope::relevant
// regression suite touching walk_fs should include at least one fixture
// whose encoded path deliberately exceeds 200 characters.
//
// Pitfall: the victim's leading component starts with `!` specifically to
// retain the `{eb}--` prefix shape matches_local's own fast-reject requires
// before it will even attempt decode_path_via_fs — a plain long alphanumeric
// name would fail that fast-reject for an unrelated reason and not exercise
// this defect at all.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-513)
fn it_78_scope_local_excludes_project_past_200_char_truncation_boundary()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let anchor = root.path().join( "anchor78" );

  // Victim: a REAL nested directory. Leading '!' forces the "--"
  // topic-boundary marker (matches_local's own precondition to even attempt
  // decode_path_via_fs); the long alphanumeric tail pushes the FULL
  // concatenated encoded string well past encode_path's 200-char threshold.
  let long_name = format!( "!{}", "x".repeat( 220 ) );
  let victim = anchor.join( &long_name );

  std::fs::create_dir_all( &anchor ).expect( "create anchor dir" );
  std::fs::create_dir_all( &victim ).expect( "create victim dir" );

  // Sanity: confirm this construction genuinely triggers encode_path's
  // truncation/hash fallback (test-setup self-check, not the defect itself).
  let encoded_anchor = claude_storage_core::encode_path( &anchor ).expect( "encode anchor path" );
  let encoded_victim = claude_storage_core::encode_path( &victim ).expect( "encode victim path" );
  assert!(
    encoded_victim.len() > 200,
    "test setup sanity: encoded_victim must exceed 200 chars (pre-truncation-boundary construction failed): len={}, value={encoded_victim}",
    encoded_victim.len()
  );
  assert!(
    encoded_victim.starts_with( &format!( "{encoded_anchor}--" ) ),
    "test setup sanity: encoded_victim must retain the '{{eb}}--' prefix shape after truncation \
     (required for matches_local to even attempt decode_path_via_fs): encoded_anchor={encoded_anchor}, encoded_victim={encoded_victim}"
  );

  common::write_path_project_session( &storage_root, &anchor, "session-it78-anchor", 2 );
  common::write_path_project_session( &storage_root, &victim, "session-it78-victim", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", anchor.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-it78-anchor" ),
    "must contain session-it78-anchor (anchor is the scope::local target); got:\n{s}"
  );
  assert!(
    !s.contains( "session-it78-victim" ),
    "must NOT contain session-it78-victim (victim's real path is a distinct, deeply-nested project \
     past encode_path's 200-char truncation boundary, not the anchor itself; scope::local is \
     documented as 'Exact match of CWD only' — docs/cli/type/07_scope_value.md:48); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::under — Multi-Byte-Unicode Sibling Wins best_partial's Tie-Break
// Over the Real Anchor (BUG-514)
//
// Root Cause: walk_fs's ambiguity-resolution heuristic chose between multiple
// Partial filesystem-decode candidates of otherwise EQUAL specificity (same
// number of remaining characters consumed) by comparing the resulting
// PathBuf's raw UTF-8 BYTE length. encode_component_piece normalizes every
// non-ASCII-alphanumeric Rust char to a SINGLE `-` byte regardless of how
// many UTF-8 bytes that char occupies on disk, so a directory name built
// from multi-byte Unicode characters (e.g. emoji, 4 bytes each) contributes
// far more bytes to a resulting PathBuf's length than an ASCII-special-
// character name of the same *character* count, even though both encode to
// an IDENTICAL piece.
//
// Why Not Caught: no existing regression exercised two real siblings that
// collide under encode_component_piece (identical resulting piece) using
// components of different UTF-8 byte width — every prior sibling-collision
// test used same-byte-width names on both sides.
//
// Fix Applied: walk_fs's tie-break now compares the consumed LENGTH OF THE
// ENCODED STRING (piece.len() + inner_consumed) rather than the decoded
// PathBuf's own byte length — see scope.rs's Fix(BUG-514) doc comment.
//
// Prevention: any future best_partial/walk_fs tie-break regression suite
// should include a multi-byte-vs-single-byte sibling collision, not only
// same-width siblings.
//
// Pitfall: do not "fix" this by comparing PathBuf::components().count()
// instead of consumed encoded length — see scope.rs's own Fix(BUG-514) doc
// comment for why that metric is equally wrong.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-514)
fn it_79_scope_under_multibyte_sibling_wins_best_partial_over_real_anchor()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let anchor_r8 = root.path().join( "anchor79" );
  let underscore_dir = anchor_r8.join( "_".repeat( 10 ) );
  let emoji_dir = anchor_r8.join( "🎉".repeat( 10 ) );

  std::fs::create_dir_all( &underscore_dir ).expect( "create underscore_dir" );
  std::fs::create_dir_all( &emoji_dir ).expect( "create emoji_dir" );

  let base_path = underscore_dir.clone();
  let eb = claude_storage_core::encode_path( &base_path ).expect( "encode base_path" );

  // Sanity: confirm the encoding collision this test depends on actually
  // holds (test-setup self-check, not the defect itself).
  let eb_emoji = claude_storage_core::encode_path( &emoji_dir ).expect( "encode emoji_dir" );
  assert_eq!(
    eb, eb_emoji,
    "test setup sanity: underscore_dir and emoji_dir must encode identically \
     (both are 10 non-alphanumeric chars) — got eb={eb}, eb_emoji={eb_emoji}"
  );

  let dir_name = format!( "{eb}-decoyzzz" );

  common::write_path_project_session( &storage_root, &base_path, "session-it79-anchor", 2 );
  common::write_test_session( &storage_root, &dir_name, "session-it79-ambiguous", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", base_path.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-it79-anchor" ),
    "must contain session-it79-anchor (anchor itself is always scope::under-relevant); got:\n{s}"
  );
  assert!(
    s.contains( "session-it79-ambiguous" ),
    "must conservatively INCLUDE session-it79-ambiguous: dir_name's shared prefix is \
     equally consistent with a deleted child of underscore_dir (the real path::\
     anchor) as with a deleted child of the unrelated emoji_dir sibling — \
     best_partial's byte-length tie-break must not silently resolve this \
     irreducible ambiguity in favor of exclusion; got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::under — Extension-Named Sibling Past 200-Char Truncation Boundary,
// No Topic Suffix (BUG-516, control)
//
// Root Cause: a real sibling whose OWN name textually EXTENDS the winning
// match's name (e.g. `anc80ctl` matches, `anc80ctl-extra-<220 z's>` is a
// real, separate, non-nested sibling) never becomes a forward-matching
// candidate in walk_fs's own per-component loop at all once its own piece is
// LONGER than what fits in `remaining` past encode_path's 200-char
// truncation — so there is no tie for Fix(BUG-514)'s consumed-length
// tie-break to break; only the shorter match ever appears as a candidate.
// And the resulting Partial(shorter_match) is USELESS as a
// search_encoded_subtree anchor in decode_path_via_fs, because the sibling
// is not inside the shorter match's own subtree.
//
// Why Not Caught: IT-78 (BUG-513) exercises the 200-char truncation boundary
// only in the DESCENDANT direction (victim nested under anchor); this
// construction is a SIBLING (not nested under anchor at all) whose name
// happens to extend the anchor's own name — the OLD per-level-redundant-
// search code (pre-BUG-515) had been incidentally finding this case from a
// shallower level, silently masking it until BUG-515's relocation to a
// single deepest-anchor call.
//
// Fix Applied: walk_fs now also detects real siblings whose piece textually
// extends the winning match's piece AND is itself longer than `remaining` —
// see scope.rs's own Fix(BUG-516) doc comment.
//
// Prevention: any future 200-char-truncation regression suite should include
// a SIBLING-direction case (this test, IT-80..83), not only IT-78's
// descendant-direction case.
//
// Pitfall: see IT-81/82/83 for the same construction WITH a synthetic topic
// suffix layered on top (BUG-512-style attack combined with BUG-516's own
// defect class).
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-516)
fn it_80_scope_under_excludes_extension_named_sibling_past_200_char_truncation_boundary()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let parent = root.path().join( "parent80" );
  let anchor = parent.join( "anc80ctl" );
  let long_tail = "z".repeat( 220 );
  let sibling_name = format!( "anc80ctl-extra-{long_tail}" );
  let sibling = parent.join( &sibling_name );

  std::fs::create_dir_all( &anchor ).expect( "create anchor dir" );
  std::fs::create_dir_all( &sibling ).expect( "create sibling dir" );

  common::write_path_project_session( &storage_root, &anchor, "session-it80-anchor", 2 );
  common::write_path_project_session( &storage_root, &sibling, "session-it80-sibling-plain", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", anchor.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-it80-anchor" ), "must include anchor itself; got:\n{s}" );
  assert!(
    !s.contains( "session-it80-sibling-plain" ),
    "must NOT contain session-it80-sibling-plain (anc80ctl-extra-<tail> is a real, \
     distinct, non-nested sibling of anc80ctl, not the anchor itself, regardless of \
     its name textually extending the anchor's); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::under — Extension-Named Sibling Past Truncation Boundary, WITH
// Synthetic Topic Suffix (BUG-516)
//
// Root Cause/Why Not Caught/Fix Applied: see IT-80. This variant layers a
// synthetic (non-real-dir) topic suffix on top of the sibling's own
// truncated+hashed encoding — a BUG-512-style attack combined with BUG-516's
// own defect class, confirming the fix holds under the more adversarial
// combined construction (this is the exact shape the MAAV Round 8 Dimension
// Adversary originally used to find BUG-515/516).
//
// Prevention: see IT-80.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-516)
fn it_81_scope_under_excludes_extension_named_sibling_with_synthetic_topic_suffix()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let parent = root.path().join( "parent81" );
  let anchor = parent.join( "anc81" );
  let long_tail = "z".repeat( 220 );
  let sibling_name = format!( "anc81-extra-{long_tail}" );
  let sibling = parent.join( &sibling_name );

  std::fs::create_dir_all( &anchor ).expect( "create anchor dir" );
  std::fs::create_dir_all( &sibling ).expect( "create sibling dir" );

  let encoded_sibling = claude_storage_core::encode_path( &sibling ).expect( "encode sibling path" );
  assert!(
    encoded_sibling.len() > 200,
    "test setup sanity: sibling's own encoding must cross the 200-char truncation \
     boundary for this construction to exercise the defect; len={}, value={encoded_sibling}",
    encoded_sibling.len()
  );
  let dir_name = format!( "{encoded_sibling}--topic81" );

  common::write_path_project_session( &storage_root, &anchor, "session-it81-anchor", 2 );
  common::write_test_session( &storage_root, &dir_name, "session-it81-sibling-topic", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", anchor.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-it81-anchor" ), "must include anchor itself; got:\n{s}" );
  assert!(
    !s.contains( "session-it81-sibling-topic" ),
    "must NOT contain session-it81-sibling-topic (extension-named sibling must be \
     excluded regardless of its topic suffix + 200-char truncation); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::local — Extension-Named Sibling Past Truncation Boundary, WITH
// Synthetic Topic Suffix (BUG-516)
//
// Root Cause/Why Not Caught/Fix Applied: see IT-80. Same construction as
// IT-81, mirrored onto scope::local (matches_local's own decode call site).
//
// Prevention: see IT-80.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-516)
fn it_82_scope_local_excludes_extension_named_sibling_with_synthetic_topic_suffix()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let parent = root.path().join( "parent82" );
  let anchor = parent.join( "anc82" );
  let long_tail = "z".repeat( 220 );
  let sibling_name = format!( "anc82-extra-{long_tail}" );
  let sibling = parent.join( &sibling_name );

  std::fs::create_dir_all( &anchor ).expect( "create anchor dir" );
  std::fs::create_dir_all( &sibling ).expect( "create sibling dir" );

  let encoded_sibling = claude_storage_core::encode_path( &sibling ).expect( "encode sibling path" );
  assert!(
    encoded_sibling.len() > 200,
    "test setup sanity: sibling's own encoding must cross the 200-char truncation \
     boundary for this construction to exercise the defect; len={}, value={encoded_sibling}",
    encoded_sibling.len()
  );
  let dir_name = format!( "{encoded_sibling}--topic82" );

  common::write_path_project_session( &storage_root, &anchor, "session-it82-anchor", 2 );
  common::write_test_session( &storage_root, &dir_name, "session-it82-sibling-topic", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", anchor.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-it82-anchor" ), "must include anchor itself; got:\n{s}" );
  assert!(
    !s.contains( "session-it82-sibling-topic" ),
    "must NOT contain session-it82-sibling-topic (extension-named sibling must be \
     excluded regardless of its topic suffix + 200-char truncation); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::around — Extension-Named Sibling Past Truncation Boundary, WITH
// Synthetic Topic Suffix (BUG-516)
//
// Root Cause/Why Not Caught/Fix Applied: see IT-80. Same construction as
// IT-81, mirrored onto scope::around (the matches_under OR matches_relevant
// union path).
//
// Prevention: see IT-80.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-516)
fn it_83_scope_around_excludes_extension_named_sibling_with_synthetic_topic_suffix()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let parent = root.path().join( "parent83" );
  let anchor = parent.join( "anc83" );
  let long_tail = "z".repeat( 220 );
  let sibling_name = format!( "anc83-extra-{long_tail}" );
  let sibling = parent.join( &sibling_name );

  std::fs::create_dir_all( &anchor ).expect( "create anchor dir" );
  std::fs::create_dir_all( &sibling ).expect( "create sibling dir" );

  let encoded_sibling = claude_storage_core::encode_path( &sibling ).expect( "encode sibling path" );
  assert!(
    encoded_sibling.len() > 200,
    "test setup sanity: sibling's own encoding must cross the 200-char truncation \
     boundary for this construction to exercise the defect; len={}, value={encoded_sibling}",
    encoded_sibling.len()
  );
  let dir_name = format!( "{encoded_sibling}--topic83" );

  common::write_path_project_session( &storage_root, &anchor, "session-it83-anchor", 2 );
  common::write_test_session( &storage_root, &dir_name, "session-it83-sibling-topic", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::around" )
    .arg( format!( "path::{}", anchor.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-it83-anchor" ), "must include anchor itself; got:\n{s}" );
  assert!(
    !s.contains( "session-it83-sibling-topic" ),
    "must NOT contain session-it83-sibling-topic (extension-named sibling must be \
     excluded regardless of its topic suffix + 200-char truncation); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::relevant — Independently-Truncated Ancestor False Exclusion (BUG-517)
//
// Root Cause: encode_path's 200-char truncation (path.rs Fix(BUG-366)) hashes
// the ORIGINAL, untruncated path string — by design, so two different paths
// sharing the same first-200-char body still disambiguate via different hash
// suffixes. is_relevant_encoded's fast-reject (a pure string check) assumes a
// literal string-prefix relationship between a shorter ancestor's encoded key
// and a longer descendant's — sound for single-sided truncation (appending
// path components only ever lengthens an encoding, so a real ancestor whose
// own encoding already exceeds 200 chars forces every real descendant's
// encoding to exceed 200 chars too), but broken once BOTH sides independently
// exceed 200 chars: each gets a DIFFERENT hash suffix derived from its OWN
// distinct full path string, breaking the prefix relationship even though the
// real filesystem ancestor/descendant relationship is genuine. The
// fast-reject returns false before decode_path_via_fs's real filesystem
// verification is ever attempted — a false exclusion.
//
// Why Not Caught: every existing 200-char-truncation fixture (IT-78,
// IT-79..83) keeps the anchor/shorter side comfortably UNDER 200 chars and
// only ever lets ONE side (the victim/sibling) cross the truncation boundary.
// No existing test makes BOTH sides independently cross it.
//
// Fix Applied: once both dir_name and encoded_base independently exceed 200
// chars, the string check can no longer prove a negative — conservatively
// return true and let decode_path_via_fs's real filesystem verification
// decide (scope.rs Fix(BUG-517)).
//
// Prevention: any future 200-char-truncation regression suite should include
// a case where BOTH the ancestor and descendant independently cross the
// boundary, not only one side.
//
// Pitfall: do not narrow the fix to "only when the shared 200-char body
// matches" — that is a necessary CONSEQUENCE of a genuine relationship under
// this construction, not an independent condition to re-check.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-517)
fn it_84_scope_relevant_includes_independently_truncated_ancestor()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let ancestor = root.path()
    .join( "a".repeat( 60 ) )
    .join( "b".repeat( 60 ) )
    .join( "c".repeat( 60 ) )
    .join( "d".repeat( 60 ) );
  let descendant = ancestor.join( "e".repeat( 60 ) );
  std::fs::create_dir_all( &descendant ).expect( "create ancestor/descendant dirs" );

  let encoded_ancestor = claude_storage_core::encode_path( &ancestor ).expect( "encode ancestor" );
  let encoded_descendant = claude_storage_core::encode_path( &descendant ).expect( "encode descendant" );

  // Sanity: confirm both sides independently cross the 200-char truncation
  // boundary with a shared first-200-char body but different hash suffixes
  // (test-setup self-check, not the defect itself).
  assert!(
    encoded_ancestor.len() > 200 && encoded_descendant.len() > 200,
    "test setup sanity: both sides must independently exceed 200 chars; \
     ancestor len={}, descendant len={}", encoded_ancestor.len(), encoded_descendant.len()
  );
  assert_eq!(
    &encoded_ancestor[ ..200 ], &encoded_descendant[ ..200 ],
    "test setup sanity: shared first-200-char body required; \
     ancestor={encoded_ancestor}, descendant={encoded_descendant}"
  );
  assert_ne!(
    &encoded_ancestor[ 200.. ], &encoded_descendant[ 200.. ],
    "test setup sanity: hash suffixes must differ; \
     ancestor={encoded_ancestor}, descendant={encoded_descendant}"
  );

  common::write_path_project_session( &storage_root, &ancestor, "session-it84-ancestor", 2 );
  common::write_path_project_session( &storage_root, &descendant, "session-it84-descendant", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::relevant" )
    .arg( format!( "path::{}", descendant.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-it84-descendant" ), "must include descendant itself; got:\n{s}" );
  assert!(
    s.contains( "session-it84-ancestor" ),
    "must include ancestor (a genuine real ancestor of descendant on disk) even though \
     both sides' INDEPENDENT 200-char truncations produce different hash suffixes after \
     an identical shared 200-char body; got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::under — Independently-Truncated Descendant False Exclusion (BUG-517)
//
// Root Cause/Why Not Caught/Fix Applied: see IT-84. Mirror in the descendant
// direction: matches_under's own `dir_name.starts_with(eb + "-")` fast-reject
// shares the identical failure mode.
//
// Prevention: see IT-84.
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-517)
fn it_85_scope_under_includes_independently_truncated_descendant()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let ancestor = root.path()
    .join( "f".repeat( 60 ) )
    .join( "g".repeat( 60 ) )
    .join( "h".repeat( 60 ) )
    .join( "i".repeat( 60 ) );
  let descendant = ancestor.join( "j".repeat( 60 ) );
  std::fs::create_dir_all( &descendant ).expect( "create ancestor/descendant dirs" );

  let encoded_ancestor = claude_storage_core::encode_path( &ancestor ).expect( "encode ancestor" );
  let encoded_descendant = claude_storage_core::encode_path( &descendant ).expect( "encode descendant" );
  assert!(
    encoded_ancestor.len() > 200 && encoded_descendant.len() > 200,
    "test setup sanity: both sides must independently exceed 200 chars; \
     ancestor len={}, descendant len={}", encoded_ancestor.len(), encoded_descendant.len()
  );
  assert_eq!(
    &encoded_ancestor[ ..200 ], &encoded_descendant[ ..200 ],
    "test setup sanity: shared first-200-char body required; \
     ancestor={encoded_ancestor}, descendant={encoded_descendant}"
  );
  assert_ne!(
    &encoded_ancestor[ 200.. ], &encoded_descendant[ 200.. ],
    "test setup sanity: hash suffixes must differ; \
     ancestor={encoded_ancestor}, descendant={encoded_descendant}"
  );

  common::write_path_project_session( &storage_root, &ancestor, "session-it85-ancestor", 2 );
  common::write_path_project_session( &storage_root, &descendant, "session-it85-descendant", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", ancestor.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-it85-ancestor" ), "must include ancestor itself; got:\n{s}" );
  assert!(
    s.contains( "session-it85-descendant" ),
    "must include descendant (genuinely nested under ancestor on disk) even though both \
     sides' INDEPENDENT 200-char truncations produce different hash suffixes; got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::under — Ambiguous Full-Match Resolution: Include When Genuinely
// Ambiguous, Exclude When Definitively Not Related (BUG-518)
//
// Root Cause: walk_fs's Full-match arm returned on the FIRST candidate whose
// recursive call yielded Full, with no tie/ambiguity detection analogous to
// Fix(BUG-514)'s Partial-vs-Partial tie-break — so when 2+ real sibling
// candidates at the same recursion level could each independently resolve
// `remaining` to Full via a genuine encode_component_piece collision spanning
// a component boundary, the verdict silently depended on std::fs::read_dir's
// platform-unspecified enumeration order.
//
// The first fix attempt collapsed a detected tie to Partial(base) — the tied
// candidates' shared parent — mirroring the Partial-vs-Partial tie-break.
// This was itself incomplete: a Full-tie is not the same shape of
// uncertainty as a Partial-vs-Partial tie (each tied candidate is already a
// COMPLETE resolution, not an incomplete prefix), and collapsing to their
// shared parent discards exactly which real paths are in play — a caller's
// own ancestor check against that overly-broad shared parent can be
// satisfied even when NEITHER individual tied candidate actually relates to
// the query, since nearly everything shares SOME ancestor.
//
// Why Not Caught: no existing fixture constructs 2+ real, DIFFERENT
// directories whose encodings collide via encode_component_piece's
// non-alphanumeric-to-hyphen normalization spanning a component boundary
// (e.g. `anchor/foo_bar`, `anchor-foo/bar`, and `anchor.foo/bar` all encode
// identically) — every existing collision fixture (IT-79) used a single
// ambiguous SESSION key, never 2+ REAL directories that individually,
// completely resolve the identical encoded string.
//
// Fix Applied: FsDecodeOutcome::AmbiguousFull(Vec<PathBuf>) preserves the
// full tied-candidate set instead of collapsing to a shared-ancestor
// Partial; matches_under/matches_relevant/matches_local each check their own
// relationship predicate against EVERY tied candidate and include only when
// at least one satisfies it (scope.rs Fix(BUG-518), continued).
//
// Prevention: any future Full-match-ambiguity regression must cover BOTH
// shapes — a genuine ambiguity where at least one tied candidate relates to
// the query (must include) and a tie where NONE of the tied candidates
// relates (must exclude) — collapsing to a coarse shared-ancestor anchor
// satisfies the first shape but silently breaks the second.
//
// Pitfall: this fixture doubles as both shapes in one construction: `anchor`
// (the query base) and its real child `foo_bar` genuinely collide with two
// real SIBLINGS of anchor (`anchor-foo/bar`, `anchor.foo/bar`) at the
// `bar`-child level (the genuine-ambiguity shape), while `anchor-foo` and
// `anchor.foo` ALSO collide with EACH OTHER one level shallower, at their own
// standalone encoding (neither is anchor nor related to it at all — the
// definitively-not-related shape). Do not simplify this fixture to only one
// collision level; the combination is what caught the first fix attempt's
// gap (MAAV Round 9 Fresh Challenger).
// ─────────────────────────────────────────────────────────────────────────────
#[ test ]
// bug_reproducer(BUG-518)
fn it_86_scope_under_ambiguous_full_match_includes_genuine_excludes_unrelated_tie()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let parent = root.path().join( "parent86" );
  std::fs::create_dir_all( &parent ).expect( "create parent dir" );

  // Two real siblings of `anchor`, colliding with EACH OTHER at their own
  // standalone encoding (hyphen vs dot both normalize to the same `-`) —
  // NEITHER is anchor nor related to it: the definitively-not-related shape.
  let decoy1 = parent.join( "anchor-foo" );
  let decoy1_child = decoy1.join( "bar" );
  std::fs::create_dir_all( &decoy1_child ).expect( "create anchor-foo/bar" );
  let decoy2 = parent.join( "anchor.foo" );
  let decoy2_child = decoy2.join( "bar" );
  std::fs::create_dir_all( &decoy2_child ).expect( "create anchor.foo/bar" );

  // The real anchor, with a real child `foo_bar` — colliding with BOTH
  // decoys' own `bar` children at the SAME encoded string: the
  // genuine-ambiguity shape (one of the 3 candidates IS under anchor).
  let anchor = parent.join( "anchor" );
  let true_child = anchor.join( "foo_bar" );
  std::fs::create_dir_all( &true_child ).expect( "create anchor/foo_bar" );

  // Sanity: confirm both collisions this test depends on (test-setup
  // self-check, not the defect itself).
  let enc_decoy1 = claude_storage_core::encode_path( &decoy1 ).expect( "encode decoy1" );
  let enc_decoy2 = claude_storage_core::encode_path( &decoy2 ).expect( "encode decoy2" );
  assert_eq!(
    enc_decoy1, enc_decoy2,
    "test setup sanity: anchor-foo and anchor.foo must encode identically; \
     got enc_decoy1={enc_decoy1}, enc_decoy2={enc_decoy2}"
  );
  let enc_true = claude_storage_core::encode_path( &true_child ).expect( "encode true_child" );
  let enc_decoy1_child = claude_storage_core::encode_path( &decoy1_child ).expect( "encode decoy1_child" );
  let enc_decoy2_child = claude_storage_core::encode_path( &decoy2_child ).expect( "encode decoy2_child" );
  assert_eq!(
    enc_true, enc_decoy1_child,
    "test setup sanity: anchor/foo_bar and anchor-foo/bar must encode identically; \
     got enc_true={enc_true}, enc_decoy1_child={enc_decoy1_child}"
  );
  assert_eq!(
    enc_true, enc_decoy2_child,
    "test setup sanity: anchor/foo_bar and anchor.foo/bar must encode identically; \
     got enc_true={enc_true}, enc_decoy2_child={enc_decoy2_child}"
  );

  common::write_test_session( &storage_root, &enc_decoy1, "session-it86-decoy-standalone", 2 );
  common::write_test_session( &storage_root, &enc_true, "session-it86-ambiguous-child", 2 );
  common::write_path_project_session( &storage_root, &anchor, "session-it86-anchor", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", anchor.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-it86-anchor" ), "must include anchor itself; got:\n{s}" );
  assert!(
    s.contains( "session-it86-ambiguous-child" ),
    "must conservatively INCLUDE session-it86-ambiguous-child: its key genuinely, \
     completely resolves to 3 real candidates (anchor/foo_bar, anchor-foo/bar, \
     anchor.foo/bar) and one of them IS under anchor; got:\n{s}"
  );
  assert!(
    !s.contains( "session-it86-decoy-standalone" ),
    "must EXCLUDE session-it86-decoy-standalone: its key resolves ambiguously to \
     anchor-foo OR anchor.foo, both real siblings of anchor, NEITHER related to it — \
     collapsing this tie to their shared parent (parent86, itself an ancestor of anchor) \
     would wrongly include it; got:\n{s}"
  );
}
