//! Shared `scope::`/`path::` resolution for commands documented to support
//! project-discovery scoping (`.projects`, `.list`, `.count`, `.search`,
//! `.show`, `.export`).
//!
//! Extracted from `.projects`' original private implementation — see
//! `docs/cli/param/12_scope.md` and `docs/cli/param/09_path.md` for the
//! user-facing contract this module implements.

use unilang::{ ErrorData, ErrorCode };
use claude_storage_core::{ Project, Storage };
use super::storage::resolve_path_parameter;

// ─── path decode helpers ───────────────────────────────────────────────────

/// Length, in bytes, of the longest common prefix of `a` and `b`. Always a
/// valid char-boundary index into both strings (accumulates whole chars).
fn common_prefix_len( a : &str, b : &str ) -> usize
{
  let mut len = 0;
  for ( ca, cb ) in a.chars().zip( b.chars() )
  {
    if ca != cb { break; }
    len += ca.len_utf8();
  }
  len
}

/// Check whether `encoded_base` (cwd or `path::` arg, encoded) is covered by
/// the project identified by `dir_name` (raw storage directory name).
///
/// Returns `true` when the project is an ancestor of (or equal to) the base:
/// - `encoded_base == dir_name` — same project, no topic
/// - `encoded_base.starts_with(dir_name + "-")` — base is in the project subtree
/// - same two checks after stripping a genuine `--topic` suffix from `dir_name`
///
/// Fix(BUG-003)
/// Root cause: the previous `rfind("--")` loop stripped from the LAST `--`
/// found anywhere in `dir_name`, with no way to tell whether that `--` was a
/// genuine topic-suffix marker or just incidental structure shared with
/// `encoded_base` (e.g. both paths sit under the same dot-prefixed temp-dir
/// root, which itself contains a `--`-like byte sequence once encoded).
/// Filesystem existence cannot discriminate either: a shallow shared ancestor
/// (e.g. `/tmp`) exists just as reliably as a genuine one.
/// Fix: only accept a `--` as a real topic boundary when it falls EXACTLY at
/// the point where `dir_name` and `encoded_base` diverge (the longest common
/// prefix). Shared/incidental structure can never BE the divergence point
/// between two different paths, so the topic-boundary decision itself is
/// structurally sound without relying on filesystem state. This guarantee is
/// scoped to that decision only: the naive literal-prefix check above
/// (`check(dir_name)`, no topic suffix involved) is intentionally unchanged
/// and was never filesystem-independent — it can still admit a same-prefix
/// sibling (e.g. an underscore/dot collision), left for the caller's
/// `decode_path_via_fs` verification to resolve. See
/// `docs/invariant/001_path_encoding.md § Contract` for the accepted
/// multi-candidate tradeoff this depends on.
/// Pitfall: do not reintroduce a blind `rfind("--")`/`split("--")` search
/// for the topic-boundary decision — any boundary search that ignores
/// `encoded_base` re-opens this hole. Also do not assume the naive prefix
/// check is filesystem-independent-safe; only the topic-boundary alignment is.
///
/// Fix(BUG-517)
/// Root cause: `encode_path`'s 200-char truncation (`path.rs` Fix(BUG-366))
/// hashes the ORIGINAL, untruncated path string — by design, so two
/// DIFFERENT paths sharing the same first-200-char body still disambiguate.
/// But once BOTH `dir_name` and `encoded_base` independently exceed 200
/// chars (each carries its OWN hash-of-its-OWN-full-path-string suffix),
/// that same by-design disambiguation defeats this function's literal-prefix
/// assumption even for a GENUINE ancestor/descendant pair: appending more
/// path components only ever lengthens an encoding, so a real ancestor whose
/// own encoding already exceeds 200 chars forces every real descendant's
/// encoding to exceed 200 chars too — meaning single-sided truncation
/// (the only shape every prior fixture exercises) can never hide a real
/// relationship, but double-sided truncation always CAN, unconditionally.
/// Fix: once both sides are independently truncated, the string check can no
/// longer prove a negative — conservatively return `true` and let the
/// caller's `decode_path_via_fs`-based real filesystem verification decide,
/// same conservative-defer philosophy as everywhere else in this file.
/// Pitfall: do not narrow this to "only when the shared 200-char body
/// matches" — that is a necessary CONSEQUENCE of a genuine relationship
/// under this construction, not an independent condition to re-check;
/// length exceeding 200 alone is already both necessary and sufficient to
/// know the string check is unsound for that side.
///
/// Fix(BUG-519)
/// Root cause: the Pitfall note directly above was wrong on the sufficiency
/// half of its own claim. "Length exceeding 200 alone" is necessary for a
/// genuine relationship (proven above) but not SUFFICIENT to rule out a
/// coincidence: two UNRELATED paths that merely share a shallow REAL
/// filesystem ancestor (e.g. both several long-named levels under the same
/// tmp root) can each independently exceed 200 chars without being nested in
/// each other at all. Once this unconditional bypass fires for such a pair,
/// the caller's `decode_path_via_fs`-based real verification (`matches_under`'s
/// `Partial` arm, Fix(BUG-512)) falsely includes it: `walk_fs` legitimately
/// cannot get past that shared real ancestor when the "unrelated" candidate's
/// own deeper subtree was never materialized on disk, and the conservative
/// `base_path.starts_with(&p)` disjunct then fires on that shared ancestor
/// alone.
/// Fix: `double_truncated_and_related` (defined below, after this function)
/// adds back a cheap, discriminating precondition — the two encodings'
/// literal first-200-char bodies must match, not just both exceed 200 chars.
/// See that function's own doc comment for why this boundary is both
/// necessary and sufficient for the confirmed failure shape, unlike the
/// plain length check it replaces.
/// Pitfall: see `double_truncated_and_related`'s own doc comment for the one
/// residual case this does NOT close.
fn is_relevant_encoded( dir_name : &str, encoded_base : &str ) -> bool
{
  let check = | candidate : &str | -> bool
  {
    encoded_base == candidate || encoded_base.starts_with( &format!( "{candidate}-" ) )
  };
  if check( dir_name ) { return true; }
  if double_truncated_and_related( dir_name, encoded_base ) { return true; }
  let lcp_len = common_prefix_len( dir_name, encoded_base );
  if lcp_len == 0 || lcp_len >= dir_name.len() { return false; }
  let ( before, after ) = ( &dir_name[ ..lcp_len ], &dir_name[ lcp_len.. ] );
  if !before.ends_with( '-' ) || !after.starts_with( '-' ) { return false; }
  check( &dir_name[ ..lcp_len - 1 ] )
}

/// Whether `a` and `b` are both independently truncated by `encode_path`'s
/// 200-char cap (`path.rs` Fix(BUG-366)) AND share an identical literal
/// first-200-char body — the only combination that actually signals a
/// genuine ancestor/descendant relationship rather than a coincidence.
///
/// `encode_path` concatenates one `encode_component_piece` per path
/// component, strictly additively, before ever truncating — so whenever a
/// real ancestor's OWN pre-truncation encoding already exceeds 200 chars,
/// every real descendant's pre-truncation encoding necessarily starts with
/// that same literal string, and truncating both to their first 200 chars
/// yields IDENTICAL bodies (only the trailing hash-of-the-full-untruncated-
/// path suffix differs). Two paths that merely happen to both encode past
/// 200 chars, sharing only a shallow REAL common ancestor, diverge as soon as
/// their components differ — almost always well before char 200 — so their
/// bodies do NOT match.
///
/// Fix(BUG-519): both call sites (`is_relevant_encoded` above,
/// `matches_under` below) previously bypassed their fast-reject whenever
/// both sides merely exceeded 200 chars, with no relatedness check at all —
/// see each call site's own Fix(BUG-519) note for the false-inclusion this
/// let through (an unrelated double-truncated sibling falsely absorbed by
/// the `Partial`-arm conservative-include disjunct, Fix(BUG-512)).
///
/// Pitfall: this does not prove soundness against every possible truncation
/// shape. A shared literal first-200-char body can arise from more than one
/// real-world cause, and NONE of them are closeable by any finite-prefix-
/// length string comparison: proof — once the first 200 chars of two paths'
/// pre-truncation encodings are already forced identical (for whatever
/// reason), the ENTIRE 200-byte comparison window this function (or any
/// variant of it) could ever inspect is exhausted before either side's own
/// distinguishing structure appears; there is no remaining character budget
/// left to observe anything past that boundary, for either a genuine
/// descendant OR an unrelated path — the two are informationally
/// indistinguishable from the stored (truncated) strings alone. Two confirmed
/// causes producing this shape:
/// (1) Deep shared ancestor (BUG-520, MAAV Round 11 Primary): if the paths'
/// shared REAL ancestor is itself deep enough that ITS OWN pre-truncation
/// encoding already exceeds 200 chars, two genuinely UNRELATED siblings under
/// that ancestor inherit an identical first-200-char body from it, despite
/// neither being nested in the other (`it_91`/`it_92`).
/// (2) Shallow ancestor plus a component-boundary escape collision (BUG-520
/// scope broadened, MAAV Round 15 Fresh Challenger): even with a SHORT shared
/// ancestor, one side's own diverging tail can independently exceed 200 chars
/// and collide, byte-for-byte, with an unrelated real multi-component chain
/// on the other side — because `encode_component_piece` maps a literal
/// hyphen WITHIN one component to the same output byte as the separator
/// BETWEEN two components (the same non-injectivity already documented for
/// untruncated paths, e.g. `/home/foo/bar` vs `/home/foo_bar`, now compounded
/// by truncation) (`it_117`). See BUG-520's own report for the full proof and
/// why this is an accepted, documented architectural limitation of the lossy
/// truncation+hash encoding scheme (`docs/invariant/001_path_encoding.md`),
/// not a fixable code defect — closing it would require storing full
/// untruncated paths, a storage-format change out of scope here. Do not treat
/// this function's `true` result as a final verdict; it only gates whether
/// the caller's own filesystem-backed check should run at all, exactly like
/// the length-only check it replaces.
///
/// Fix(BUG-521)
/// Root cause: the original `a[ ..200 ] == b[ ..200 ]` raw byte-index slicing
/// assumed BOTH arguments are pure-ASCII `encode_path` output. That is
/// guaranteed for the `encoded_base`/`eb` argument (always freshly produced
/// by `encode_path` in `resolve_scoped_projects`, and `encode_component_piece`
/// maps every non-ASCII-alphanumeric character to a single ASCII `-`) but was
/// NEVER guaranteed for the `dir_name` argument, which is read directly off
/// the real filesystem (`Storage::list_projects()` → `fs::read_dir()`) with
/// no validation that it actually conforms to the encoding contract. A real
/// (non-conforming) storage directory name containing a multi-byte UTF-8
/// character whose byte range straddles byte offset 200 made `a[ ..200 ]`
/// panic ("byte index 200 is not a char boundary"), crashing the entire
/// `.projects` command for every project in the same call, not just the
/// malformed one (confirmed: MAAV Round 11 Dimension Adversary).
/// Fix: compare via `common_prefix_len` (defined above in this file, already
/// char-boundary-safe by construction) instead of raw byte slicing —
/// `common_prefix_len(a, b) >= 200` is exactly equivalent to
/// `a[..200] == b[..200]` whenever both ARE valid 200+-byte ASCII strings
/// (the common case), but never panics regardless of either argument's
/// encoding, since it only ever advances by whole `char`s.
/// Pitfall: do not reintroduce direct byte-index slicing (`&s[..N]`) on
/// either argument anywhere in this function — `dir_name` must always be
/// treated as untrusted, non-guaranteed-ASCII input.
fn double_truncated_and_related( a : &str, b : &str ) -> bool
{
  a.len() > 200 && b.len() > 200 && common_prefix_len( a, b ) >= 200
}

/// Decode a storage directory name into a human-readable display path.
///
/// Path-encoded dirs start with `-` (e.g. `-home-alice-projects`). UUID dirs do not.
/// Compress `$HOME` prefix to `~` for display. Returns full path string if HOME unset.
fn tilde_compress( path : &std::path::Path ) -> String
{
  if let Ok( home ) = std::env::var( "HOME" )
  {
    if let Ok( rel ) = path.strip_prefix( std::path::Path::new( &home ) )
    {
      return format!( "~/{}", rel.display() );
    }
  }
  path.display().to_string()
}

/// Walk the filesystem to decode a lossy-encoded storage dir name to a real path.
///
/// `encode_path` normalizes every non-alphanumeric character to `-`, so the
/// encoded name alone cannot say which real character (or how many
/// consecutive ones, or whether they sit at a component boundary or inside
/// one component's own name) produced any given run of hyphens. This
/// function resolves the ambiguity against the real filesystem — see
/// `walk_fs`'s own doc comment (Fix(BUG-511)) for the resolution algorithm.
///
/// Returns `FsDecodeOutcome::NotFound` when no real filesystem entry
/// corresponds to even the first component of `encoded` (project deleted,
/// remote, or unmounted) — see `FsDecodeOutcome`'s own doc comment
/// (Fix(BUG-512), Fix(BUG-513)) for the `Full`/`Partial` distinction this
/// now also reports.
///
/// # Why only as fallback
///
/// Requires the project directory to exist on disk. Always call heuristic decode first
/// and only reach here when that result does not exist. This avoids unnecessary stat
/// calls for paths the heuristic already handles correctly.
///
/// Fix(BUG-515)
/// Root cause: `search_encoded_subtree` used to be invoked from INSIDE
/// `walk_fs`, once per recursion level, on that level's own `base` — so a
/// target that could never be found (e.g. a real sibling's encoding plus a
/// synthetic, non-real topic suffix — no real path can ever match it) forced
/// every ancestor level, all the way up to `walk_fs`'s own root call at `/`,
/// to ALSO exhaustively search its own (progressively larger) subtree before
/// giving up. At the `/`-level call this meant walking the entire real
/// filesystem — `/proc`, `/sys`, every mounted volume — with no bound, an
/// unbounded hang (confirmed: 20+ minutes sustained CPU, `scope::under`/
/// `scope::around`, MAAV Round 8 Dimension Adversary).
/// Fix: call `search_encoded_subtree` exactly ONCE here, after the plain
/// incremental walk (`walk_fs`) has already run to completion, anchored at
/// the DEEPEST real path the incremental walk actually verified. That anchor
/// is always correct and never too shallow: per-component matching against
/// `encode_component_piece` is lossless (not approximate) for any portion of
/// `encoded` before `encode_path`'s outer truncation step, so the point
/// incremental matching gets stuck is, by construction, exactly where the
/// lossless information runs out and only the truncated-hash tail remains —
/// there is no shallower point worth searching from, and no deeper one is
/// reachable without already having a real match. This bounds the search to
/// the smallest subtree that could possibly contain the answer, instead of
/// the whole filesystem.
/// Pitfall: do not move this call back inside `walk_fs`'s own recursion, and
/// do not call it more than once per `decode_path_via_fs` invocation — either
/// change reintroduces the same redundant, unbounded-at-shallow-levels search
/// this fix removes. `walk_fs` itself no longer knows about `encoded`'s full,
/// untruncated length at all; only this function does.
fn decode_path_via_fs( encoded : &str ) -> FsDecodeOutcome
{
  let inner = &encoded[ 1.. ]; // strip leading `-`
  let ( outcome, _consumed ) = walk_fs( std::path::Path::new( "/" ), inner, true, inner.len() );
  match outcome
  {
    // Fix(BUG-512), Fix(BUG-513): zero real progress past the root is
    // exactly the classic "deleted/remote project" case the original
    // conservative-include fallback existed for — normalize it to
    // `NotFound` rather than a near-meaningless `Partial("/")`, so callers
    // keep conservatively including it, same as before either bug existed.
    FsDecodeOutcome::Partial( p ) if p == std::path::Path::new( "/" ) => FsDecodeOutcome::NotFound,
    // Fix(BUG-515): single, correctly-anchored fallback — see this
    // function's own doc comment above.
    // Fix(BUG-523): `search_encoded_subtree` now returns `FsDecodeOutcome`
    // directly (ambiguity-preserving) instead of `Option<PathBuf>` — see its
    // own doc comment. `Partial`/`NotFound` from the subtree search both mean
    // "the fallback found nothing better," so both fall back to the
    // already-verified `p` anchor unchanged, same as the old `None` arm.
    FsDecodeOutcome::Partial( p ) if encoded.len() > 200 =>
      match search_encoded_subtree( &p, encoded )
      {
        found @ ( FsDecodeOutcome::Full( _ ) | FsDecodeOutcome::AmbiguousFull( _ ) ) => found,
        FsDecodeOutcome::Partial( _ ) | FsDecodeOutcome::AmbiguousPartial( _ ) | FsDecodeOutcome::NotFound => FsDecodeOutcome::Partial( p ),
      },
    // Fix(BUG-526): same single-rescue rationale as the `Partial` arm above
    // (Fix(BUG-515)), but anchored at the tied candidates' COMMON ANCESTOR —
    // the deepest point verified to contain ALL of them — rather than any
    // single candidate. Skipped when that ancestor is the filesystem root,
    // mirroring the `Partial("/")` intercept above exactly: an unbounded
    // whole-filesystem search must never run. A search that finds nothing
    // better falls back to the already-verified tied set unchanged.
    // Fix(BUG-530): use `search_encoded_subtree_tied`, passing `candidates`
    // itself (not just `anchor`) — the tied candidates, not their shallower
    // common ancestor, are where `walk_fs`'s own matching actually stalled;
    // see that function's own doc comment.
    // Fix(BUG-532): a `Full`/`AmbiguousFull` rescue result is only ever
    // reachable through ONE original candidate's own subtree at a time (the
    // recursion that found it necessarily descended from a specific real
    // directory) — replacing the WHOLE original tied set with it silently
    // drops every OTHER original candidate that the rescue neither confirmed
    // nor disproved. See `merge_tied_rescue_findings`'s own doc comment.
    FsDecodeOutcome::AmbiguousPartial( candidates ) if encoded.len() > 200 =>
    {
      let anchor = common_ancestor( &candidates );
      if anchor == std::path::Path::new( "/" )
      {
        FsDecodeOutcome::AmbiguousPartial( candidates )
      }
      else
      {
        match search_encoded_subtree_tied( &anchor, encoded, &candidates )
        {
          FsDecodeOutcome::Full( p ) => merge_tied_rescue_findings( vec![ p ], candidates ),
          FsDecodeOutcome::AmbiguousFull( found ) => merge_tied_rescue_findings( found, candidates ),
          FsDecodeOutcome::Partial( _ ) | FsDecodeOutcome::AmbiguousPartial( _ ) | FsDecodeOutcome::NotFound =>
            FsDecodeOutcome::AmbiguousPartial( candidates ),
        }
      }
    }
    other => other,
  }
}

/// Decode the base-encoded component of a storage dir name to a real filesystem path.
///
/// Returns `None` if the encoded string is malformed (non-path-encoded keys such as UUIDs).
/// When `decode_path` succeeds but the result does not exist on disk, falls back to the
/// filesystem-guided walk to resolve `_` vs `/` ambiguity (Fix(issue-029)).
fn decode_storage_base( base_encoded : &str ) -> Option< std::path::PathBuf >
{
  use claude_storage_core::decode_path;
  let h = decode_path( base_encoded ).ok()?;
  if h.exists()
  {
    Some( h )
  }
  else
  {
    // Fix(issue-029): heuristic maps '_' to '/', try filesystem-guided decode.
    // Fix(BUG-512), Fix(BUG-513): only a FULL match is preferable to the
    // heuristic here. A `Partial` match is, by definition, missing
    // information (an unresolved topic suffix or truncated remainder) that
    // the heuristic's own '--'-chain decoding already reconstructs for
    // display purposes — preferring it over `h` would silently drop that
    // suffix from the displayed path (see
    // `projects_shows_topic_path_when_topic_dir_absent` and siblings, which
    // require the heuristic's topic-inclusive guess).
    // Fix(BUG-518): `AmbiguousFull` has no single unambiguous real path to
    // prefer either — same "missing information" shape as `Partial` for
    // display purposes (here, which of 2+ real candidates), so it falls
    // back to the heuristic `h` alongside `Partial`/`NotFound` rather than
    // arbitrarily picking one tied candidate.
    // Fix(BUG-526): `AmbiguousPartial` shares that same shape exactly (2+
    // tied incomplete prefixes, no single unambiguous real path), so it
    // falls back to `h` here too.
    Some( match decode_path_via_fs( base_encoded )
    {
      FsDecodeOutcome::Full( p ) => p,
      FsDecodeOutcome::AmbiguousFull( _ ) | FsDecodeOutcome::Partial( _ ) | FsDecodeOutcome::AmbiguousPartial( _ ) | FsDecodeOutcome::NotFound => h,
    } )
  }
}

/// Return true if `dir_name` encodes a project path that is `base_path` itself or is nested
/// under `base_path` (`scope::under` predicate).
///
/// The single-hyphen fast-reject `starts_with("{eb}-")` weeds out projects with completely
/// different paths before the more expensive filesystem decode.
///
/// Fix(BUG-003)
/// Root cause: previously stripped a `--topic` suffix via `strip_topic_suffix` before
/// the filesystem-verification decode. That stripping used the same unsound blind
/// `find("--")` search being removed from `is_relevant_encoded` for the same reason.
/// Fix: decode `dir_name` directly — `decode_path_via_fs` already treats an
/// unverifiable/nonexistent path as `true` (conservative include), which covers the
/// case where a genuine topic suffix would have made the raw decode fail to exist.
///
/// Fix(BUG-512)
/// Root cause: the `map_or(true, ...)` fallback this doc comment describes above used
/// to fire on EVERY unresolvable `decode_path_via_fs` result, including one where
/// `walk_fs` had already fully verified a REAL base directory along the way (e.g. a
/// genuine sibling project) and only failed to consume a trailing, non-real `--topic`
/// suffix. `walk_fs`'s old `Option<PathBuf>` return type could not tell that case apart
/// from "nothing real at all" — both collapsed to `None`, both triggered blind
/// inclusion, letting an unrelated sibling's topic-tagged session leak into
/// `scope::under`'s results.
/// Fix: `decode_path_via_fs` now returns `FsDecodeOutcome`, distinguishing a fully-verified
/// real base with an unresolved remainder (`Partial`) from genuinely nothing real at all
/// (`NotFound`). A `Full` match is unambiguous — use the plain `starts_with` relationship
/// check. A `Partial` match, though backed by a real, verified path, only tells us where the
/// incremental walk RAN OUT of real filesystem to check against — that terminal path is
/// always at-or-shallower-than whatever `dir_name` truly represents, since the walk can never
/// skip ahead of real evidence. So a `Partial` path can land in one of two shapes: either it
/// diverged from `base_path`'s own ancestor chain entirely (a real, different, conflicting
/// path — e.g. an unrelated sibling with a topic suffix), which is genuine evidence of
/// exclusion; or it simply ran out of real entries to confirm/deny a deeper, still-plausible
/// nesting under `base_path` (e.g. a session recorded for a project whose directory has since
/// been deleted, where only some shallower ancestor of it still exists on disk) — in which
/// case there is no real conflict, only missing evidence, and the original conservative-include
/// philosophy still applies. `base_path.starts_with(&p)` (`p` is an ancestor of, or equal to,
/// `base_path`) distinguishes the second shape from the first.
/// Pitfall: this simplification is only sound because no current test combines
/// `scope::under` with a project that has a genuine topic suffix; if such a test is
/// added, re-verify this fallback still selects the correct base. Do not collapse the
/// `Full`/`Partial` arms back into one shared check — a `Full` match is unambiguous and must
/// use the strict `starts_with` check alone; only `Partial` needs the extra ancestor-of
/// disjunct, precisely because it is missing evidence a `Full` match already has.
///
/// Fix(BUG-517)
/// Same root cause and fix as `is_relevant_encoded`'s own Fix(BUG-517) note (mirrored in
/// the descendant direction: `dir_name` is the potential descendant, `eb` the anchor) —
/// once both are independently truncated past 200 chars, the literal-prefix fast-reject
/// below cannot prove `dir_name` is NOT under `eb`, so it must defer to real verification
/// instead of rejecting.
///
/// Fix(BUG-518)
/// `AmbiguousFull` (see `FsDecodeOutcome`'s own doc comment) carries the full set of
/// real, mutually-exclusive candidates a tied `walk_fs` resolution could be — check each
/// individually with the same `starts_with(base_path)` relationship `Full` uses, and
/// include when at least one qualifies. Do not collapse the set to a shared ancestor
/// first (that is precisely the false-inclusion this variant exists to avoid).
///
/// Fix(BUG-519)
/// Same root cause and fix as `is_relevant_encoded`'s own Fix(BUG-519) note (mirrored in
/// the descendant direction) — see `double_truncated_and_related`'s doc comment for the
/// full explanation. The prior unconditional both-exceed-200-chars bypass let an
/// unrelated double-truncated sibling fall through to the `Partial` arm's
/// conservative-include disjunct above (Fix(BUG-512)), which then falsely fired on their
/// shared real ancestor.
///
/// Fix(BUG-526)
/// `AmbiguousPartial` mirrors this function's own Fix(BUG-518) note one confidence level
/// down: apply BOTH `Partial` disjuncts (`p.starts_with(base_path)` and
/// `base_path.starts_with(p)`) to EVERY tied candidate individually rather than collapsing
/// the set to a shared ancestor first — see `FsDecodeOutcome::AmbiguousPartial`'s own doc
/// comment for the no-collapse rationale.
fn matches_under( dir_name : &str, eb : &str, base_path : &std::path::Path ) -> bool
{
  let double_truncated = double_truncated_and_related( dir_name, eb );
  if !double_truncated && dir_name != eb && !dir_name.starts_with( &format!( "{eb}-" ) ) { return false; }
  if dir_name == eb { return true; }
  match decode_path_via_fs( dir_name )
  {
    FsDecodeOutcome::Full( p ) => p.starts_with( base_path ),
    FsDecodeOutcome::AmbiguousFull( candidates ) => candidates.iter().any( | p | p.starts_with( base_path ) ),
    FsDecodeOutcome::AmbiguousPartial( candidates ) =>
      candidates.iter().any( | p | p.starts_with( base_path ) || base_path.starts_with( p ) ),
    FsDecodeOutcome::Partial( p ) => p.starts_with( base_path ) || base_path.starts_with( &p ),
    FsDecodeOutcome::NotFound => true,
  }
}

/// Return true if `dir_name` encodes a project path that is an ancestor of `base_path`
/// (`scope::relevant` predicate).
///
/// Fix(BUG-003)
/// Root cause/Pitfall: see `matches_under` — same topic-suffix-stripping removal,
/// same conservative-include fallback in `decode_path_via_fs`.
///
/// Fix(BUG-512)
/// Same root cause and fix as `matches_under`'s own Fix(BUG-512) note — mirrored in the
/// ancestor direction (`base_path.starts_with(p)` instead of `p.starts_with(base_path)`).
/// Fix(BUG-518)
/// `AmbiguousFull` mirrors `matches_under`'s own Fix(BUG-518) note in the ancestor
/// direction: check `base_path.starts_with(p)` against each tied candidate individually
/// rather than collapsing the set to a shared ancestor first.
///
/// Fix(BUG-519)
/// No code change here — this function has no `double_truncated` logic of its own and
/// inherits the fix entirely through `is_relevant_encoded`'s own Fix(BUG-519) gate. Noted
/// for traceability only, since this function was independently probed for the same
/// false-inclusion shape during the MAAV re-verification that found BUG-519.
///
/// Fix(BUG-526)
/// `AmbiguousPartial` mirrors `matches_under`'s own Fix(BUG-526) note in the ancestor
/// direction: check `base_path.starts_with(p)` against each tied candidate individually
/// (the same predicate the `Full`/`Partial` arm shares), never against a collapsed shared
/// ancestor.
fn matches_relevant( dir_name : &str, eb : &str, base_path : &std::path::Path ) -> bool
{
  if !is_relevant_encoded( dir_name, eb ) { return false; }
  if dir_name == eb { return true; }
  match decode_path_via_fs( dir_name )
  {
    FsDecodeOutcome::Full( p ) | FsDecodeOutcome::Partial( p ) => base_path.starts_with( &p ),
    FsDecodeOutcome::AmbiguousFull( candidates ) => candidates.iter().any( | p | base_path.starts_with( p ) ),
    FsDecodeOutcome::AmbiguousPartial( candidates ) => candidates.iter().any( | p | base_path.starts_with( p ) ),
    FsDecodeOutcome::NotFound => true,
  }
}

/// Return true if `dir_name` encodes a project path that is exactly `base_path`
/// itself (`scope::local` predicate) — the anchor project only, never a
/// descendant or ancestor.
///
/// Fix(BUG-509)
/// Root cause: the previous inline check in `project_matches`'s `"local"` arm
/// was a naive `dir_name.starts_with("{eb}--")` with no filesystem
/// verification — unlike `matches_under`/`matches_relevant`, which both
/// already received the BUG-003 treatment. A REAL nested project whose
/// leading path component is non-alphanumeric-prefixed (e.g. `.venv`) encodes
/// to exactly `"{eb}--venv"` (`encode_path`'s `--` topic-boundary marker),
/// satisfying the naive check even though it is a genuine, separate, nested
/// project — not a topic-suffix alias of the anchor. This let `scope::local`
/// silently include an unrelated nested project's sessions (a cross-project
/// data leak, since `scope::local` is also the default scope for
/// `session::`-targeted `.show`/`.export`).
/// Fix: verify the `--`-shaped candidate via `decode_path_via_fs`; if it
/// resolves to a REAL path, only match when that path is EXACTLY `base_path`
/// (never merely nested under it — that is `scope::under`'s job, not
/// `scope::local`'s). An unresolvable candidate (genuine synthetic topic tag,
/// no real directory on disk) is conservatively included, same fallback
/// philosophy as `matches_under`/`matches_relevant`.
/// Pitfall: do not reuse `matches_under`'s `starts_with(base_path)` check
/// for the `Full` case — `scope::local` means the anchor itself, so a `Full`
/// match's comparison must be equality, not a `starts_with`/ancestor
/// relationship.
///
/// Fix(BUG-513)
/// Root cause: `walk_fs`'s per-component matching only ever knew
/// `encode_component_piece` — the PER-COMPONENT half of `encode_path`'s
/// rule. It had no knowledge of `encode_path`'s separate, outer
/// 200-character truncation + djb2-hash-suffix step (`path.rs:234-246`,
/// applied ONCE to the fully concatenated string). Any real project whose
/// encoded path crossed that boundary could never be matched by incremental
/// per-component consumption — `decode_path_via_fs` unconditionally returned
/// unresolvable for it, and the `map_or(true, ...)` fallback then admitted
/// it into `scope::local`'s results even though it was a distinct, deeply
/// nested project, not the anchor itself.
/// Fix: `walk_fs` now also tries the REAL `encode_path` (truncation
/// included) directly against real candidate paths once incremental
/// per-component matching gets stuck (see `walk_fs`'s own doc comment) —
/// this recognizes a truncated-hash match by asking the actual production
/// encoder, never by reimplementing its truncation rule locally.
///
/// Fix(BUG-513) (continued — Partial-directionality correction)
/// Root cause: the initial BUG-512/BUG-513 fix used the identical
/// `p == base_path` check for BOTH `Full` and `Partial` outcomes. A `Partial`
/// path is only ever at-or-shallower-than `dir_name`'s true (possibly
/// unresolvable) target — so a `Partial` terminating strictly above
/// `base_path` (e.g. a session recorded against a since-deleted nested
/// directory, where only some real shallower ancestor still exists on disk)
/// could never equal `base_path` exactly, wrongly EXCLUDING a case the
/// original conservative-include fallback existed to protect — mirroring
/// `matches_under`'s own T02 regression (`cli_cmd_show_test.rs`) one
/// directory level up.
/// Fix: for `Partial` only, also accept `base_path.starts_with(&p)` (`p` is
/// an ancestor of, or equal to, `base_path`) — see `matches_under`'s own
/// Fix(BUG-512) doc comment for the full disjunct rationale, mirrored here
/// unchanged. A `Partial` landing on a real path that DIVERGES from
/// `base_path`'s own ancestor chain (neither equal nor an ancestor) is still
/// genuine conflicting evidence and is excluded, same as before.
/// Pitfall: do not collapse the `Full`/`Partial` arms back into one shared
/// check — a `Full` match is unambiguous and must use strict equality alone;
/// only `Partial` needs the extra ancestor-of disjunct, precisely because it
/// is missing evidence a `Full` match already has (see `matches_under`'s
/// identical pitfall note).
///
/// Fix(BUG-518)
/// `AmbiguousFull` mirrors `matches_under`'s own Fix(BUG-518) note, using the same
/// exact-equality relationship `Full` uses (`scope::local` means the anchor itself) —
/// check each tied candidate individually rather than collapsing the set to a shared
/// ancestor first.
///
/// Fix(BUG-526)
/// `AmbiguousPartial` mirrors this function's own Fix(BUG-518) note one confidence level
/// down: check `p == base_path || base_path.starts_with(p)` (the `Partial` arm's own
/// conservative pair) against each tied candidate individually rather than collapsing the
/// set to a shared ancestor first.
fn matches_local( dir_name : &str, eb : &str, base_path : &std::path::Path ) -> bool
{
  if dir_name == eb { return true; }
  if !dir_name.starts_with( &format!( "{eb}--" ) ) { return false; }
  match decode_path_via_fs( dir_name )
  {
    FsDecodeOutcome::Full( p ) => p == base_path,
    FsDecodeOutcome::AmbiguousFull( candidates ) => candidates.iter().any( | p | *p == base_path ),
    FsDecodeOutcome::AmbiguousPartial( candidates ) =>
      candidates.iter().any( | p | *p == base_path || base_path.starts_with( p ) ),
    FsDecodeOutcome::Partial( p ) => p == base_path || base_path.starts_with( &p ),
    FsDecodeOutcome::NotFound => true,
  }
}

/// Outcome of walking the filesystem to resolve an encoded storage-dir name
/// (`walk_fs`/`decode_path_via_fs`) against real directory entries.
///
/// Fix(BUG-512), Fix(BUG-513)
/// Root cause: the pre-BUG-512/513 design returned a plain `Option<PathBuf>`
/// — found (full match) or not — with no way to express "walked into and
/// fully verified a REAL base directory, but a trailing remainder of the
/// encoded string couldn't be matched to any further real subdirectory".
/// That collapsed two structurally different situations into the same
/// `None`: genuinely nothing real on disk (deleted/remote project — the
/// origin of the conservative-include fallback in `matches_local`/
/// `matches_under`/`matches_relevant`) versus a real, verified, but
/// DIFFERENT project (a sibling with a topic suffix, or one past
/// `encode_path`'s 200-character truncation boundary). Both callers'
/// fallback then admitted the second case too, causing a cross-project data
/// leak.
/// Fix: report which situation actually occurred. `Full`/`Partial` are both
/// backed by a real, filesystem-verified path, but they are NOT equally
/// informative: `dir_name`'s true target (real or hypothetical) is always
/// at-or-under a `Partial` path, since the walk only ever advances by
/// matching real prefixes — so a `Partial` path is never deeper than reality
/// can support, only ever at-or-shallower-than the truth. That means a
/// caller checking a DESCENDANT-direction relationship (`scope::local`'s
/// equality, `scope::under`'s `starts_with(base_path)`) cannot always trust
/// a bare `Full`-style check on a `Partial` result: the walk may simply have
/// run out of real filesystem before reaching `base_path`'s own depth (e.g.
/// a session recorded against a since-deleted nested directory, with only a
/// shallower real ancestor left on disk) — not because it found a
/// conflicting real path, but for lack of evidence either way. Those two
/// callers additionally accept `base_path.starts_with(&p)` (`p` is an
/// ancestor of, or equal to, `base_path`) for the `Partial` case specifically
/// — see the full rationale in `matches_under`'s and `matches_local`'s own
/// Fix(BUG-512)/mirror doc comments. `scope::relevant`'s own check
/// (`base_path.starts_with(&p)`)
/// already has this shape by construction and needs no such split between
/// `Full` and `Partial`. Only `NotFound` — zero real progress past the very
/// first component — still conservatively includes outright, preserving the
/// original deleted/remote-project rationale for exactly the case it was
/// meant for.
enum FsDecodeOutcome
{
  /// `encoded`'s full string was consumed and verified — the path is
  /// unambiguously what it represents.
  Full( std::path::PathBuf ),
  /// `encoded`'s full string was consumed, but by 2+ DIFFERENT real
  /// candidates (an `encode_component_piece` collision spanning a
  /// component boundary — see `walk_fs`'s own Fix(BUG-518) doc comment).
  /// Each entry is itself a COMPLETE, mutually-exclusive resolution of the
  /// entire encoded string — the true target is exactly one of them, just
  /// unknowable which. Deliberately NOT collapsed to the candidates' common
  /// ancestor as a `Partial`: that ancestor is almost always related to
  /// unrelated queries too (nearly everything shares SOME ancestor),
  /// silently reintroducing false-inclusion. Callers must check their own
  /// relationship predicate against EVERY candidate and conservatively
  /// include only when AT LEAST ONE satisfies it.
  AmbiguousFull( Vec< std::path::PathBuf > ),
  /// A real, filesystem-verified base path was found, but a nonempty
  /// remainder past it could not be matched to any further real
  /// subdirectory (unresolvable topic tag, or a deleted/renamed
  /// descendant). Safe for relationship checks; not safe for an
  /// exact-identity conclusion beyond what the caller's own check expresses.
  Partial( std::path::PathBuf ),
  /// A real, filesystem-verified prefix was found, but 2+ DIFFERENT real
  /// candidates consumed the SAME maximal portion of `encoded` (a
  /// Partial-vs-Partial tie — the same collision class as `AmbiguousFull`,
  /// but each candidate is an INCOMPLETE prefix of the target, not a
  /// complete resolution). Carries the full tied set rather than collapsing
  /// to the candidates' shared parent: that parent is related to unrelated
  /// queries too, so a caller's conservative ancestor check against it can
  /// be satisfied even when NEITHER tied candidate relates to the query
  /// (Fix(BUG-526) — the exact shape `AmbiguousFull`'s own doc above
  /// describes, one confidence level down). Callers must apply their own
  /// `Partial`-semantics predicate to EVERY candidate and include when AT
  /// LEAST ONE qualifies — per-candidate, each entry follows the same
  /// relationship rules as a plain `Partial` path.
  AmbiguousPartial( Vec< std::path::PathBuf > ),
  /// No real filesystem entry corresponds to even the first component —
  /// nothing on disk can confirm or refute this candidate at all.
  NotFound,
}

/// Exhaustively search the real subtree rooted at `base` (`base` included)
/// for a real path whose FULL encoding — via the actual, truncating
/// `claude_storage_core::encode_path` — equals `target` exactly.
///
/// Fix(BUG-513)
/// Used as a fallback once `walk_fs`'s cheap incremental per-component walk
/// can no longer make progress: `encode_path`'s 200-character truncation +
/// djb2-hash-suffix step (`path.rs:234-246`) is applied ONCE to the fully
/// concatenated string, so it can cut a real component's own encoding off
/// mid-way and append an opaque hash — no per-component prefix match can
/// ever reconstruct that. Rather than reimplementing the truncation rule
/// locally (the exact anti-pattern `walk_fs`'s own Fix(BUG-511) doc comment
/// warns against), this calls the REAL encoder on every real candidate path
/// in the subtree and compares directly — correct by construction for any
/// truncation length or hash value, because it never needs to guess the
/// rule, only ask the function that already implements it.
///
/// Bounded by the real, currently-existing filesystem subtree under `base`
/// — never broader, and only invoked when `target` is long enough
/// (`> 200` chars) that truncation could plausibly have produced it.
///
/// Fix(BUG-515): the call site lives in `decode_path_via_fs`, invoked
/// exactly ONCE per decode, anchored at the deepest path `walk_fs`'s
/// incremental walk already verified — never inside `walk_fs`'s own
/// recursion. See `decode_path_via_fs`'s doc comment for why calling this
/// once per recursion LEVEL (the original BUG-513 shape) caused an
/// unbounded filesystem walk from shallow bases when no real match exists
/// anywhere for the target string.
///
/// Fix(BUG-522)
/// Root cause: this function only ever recognized EXACT equality
/// (`encode_path(candidate) == target`) — but `target` is not always a bare
/// project encoding; it can be a real, truncation-affected project's own
/// encoding PLUS a trailing synthetic `--topic` suffix (the same topic-tag
/// convention `matches_local`'s own fast-reject already recognizes via
/// `dir_name.starts_with("{eb}--")`). When a REAL project sits deep enough
/// that its OWN per-component piece no longer fits within `walk_fs`'s
/// truncated `remaining` budget (the exact BUG-516 shape, one level deeper —
/// here the "too-long" candidate has no shorter sibling `best` to extend at
/// all, since it may be the ONLY real child at that level), `walk_fs` never
/// even sees it as a candidate and falls back to `Partial(shallower_anchor)`.
/// The old exact-match-only search then found nothing for a topic-suffixed
/// target (no real directory's OWN encoding is EVER exactly equal to
/// `nested_encoding + "--topic"`), silently leaving the caller with the
/// too-shallow anchor — which `matches_local` (`p == base_path`) then wrongly
/// treated as "the anchor itself, with a bare metadata tag," when the true
/// target was a genuinely SEPARATE, deeper real project's topic-tagged
/// session (confirmed: MAAV Round 11 Fresh Challenger, `scope::local`
/// wrongly including a nested project's session under a shallow, untruncated
/// anchor).
/// Fix: also recognize `target.starts_with("{encoded}--")` (candidate's own
/// encoding, followed by a genuine topic boundary) as a match — mirroring
/// `matches_local`'s own already-established topic-boundary convention, just
/// applied here to the real-filesystem search instead of a bare string
/// fast-reject. Recursing into children BEFORE checking `base` itself is
/// load-bearing, not stylistic: since a shallower real ancestor's own
/// encoding is, by construction, always ALSO a literal prefix of a deeper
/// real descendant's encoding, a shallow candidate can spuriously satisfy
/// this new looser `--`-boundary condition too (whenever the descendant's
/// own next component happens to start with a special character) — checking
/// children first ensures the DEEPEST, most-specific real match wins,
/// exactly the specificity guarantee `walk_fs`'s own `best_partial` deepest-
/// consumed comparison (Fix(BUG-514)) already provides elsewhere in this
/// file. This reordering is safe for the pre-existing exact-match case too:
/// an exact `encode_path` match is unique across the whole subtree (barring
/// an astronomically unlikely hash collision), so traversal order never
/// changes which candidate exact-matches, only which is found first when
/// BOTH the new prefix condition and (at a different, deeper level) an exact
/// match are in play.
/// Pitfall: do not narrow the new condition to a specific expected suffix
/// (e.g. checking for `"--faketopic"` literally) — any `--`-bounded
/// remainder must qualify, matching `matches_local`'s own suffix-agnostic
/// fast-reject; do not drop the children-first ordering in favor of
/// checking `base` first for a "fast path" — see the specificity argument
/// above for why that would silently resurrect the shallow-anchor bug.
///
/// Fix(BUG-523)
/// Root cause: this function returned `Option<PathBuf>` and short-circuited
/// on the FIRST match found anywhere in ANY sibling's own subtree
/// (`return Some(found)` inside the `for entry in entries.flatten()` loop,
/// never examining the remaining siblings). Two real, DIFFERENT sibling
/// directories whose own `encode_path()` output collides identically —
/// e.g. one named with a leading `!` and one with a leading `_`, both
/// mapped to the same `--` escape by `encode_component_piece` — each
/// independently satisfy `encoded == target`, but only whichever
/// `std::fs::read_dir` happens to enumerate first (a platform-unspecified,
/// unstable order) was ever reported; the other was silently never checked
/// at all. This is the exact same collision class `walk_fs`'s own
/// `full_matches`/`AmbiguousFull` machinery already exists to catch
/// (Fix(BUG-518)) — this function simply never received the equivalent
/// treatment, because it predates BUG-518 and was never revisited when
/// `FsDecodeOutcome` gained its ambiguity-preserving variant (confirmed:
/// MAAV Round 12 Primary).
/// Fix: return `FsDecodeOutcome` (reusing the existing type, not a parallel
/// one) instead of `Option<PathBuf>`. Complete the FULL loop over every
/// sibling at each level — never return early — collecting every match
/// found across ALL sibling subtrees at this level into `child_matches`; a
/// single collected match returns `Full`, 2+ DISTINCT ones return
/// `AmbiguousFull` (flattening a nested call's own `AmbiguousFull`, same
/// convention `walk_fs` already uses), and only when `child_matches` is
/// completely empty does this level fall through to checking `base` itself.
/// Checking `base` only after ALL children (across ALL siblings, not just
/// the first-tried one) are exhausted preserves the load-bearing
/// children-first-wins-over-self priority this function's own Fix(BUG-522)
/// note establishes, while now also catching genuine same-level sibling
/// ties the old early-return could never reach.
/// Pitfall: do not reintroduce an early `return` inside the `for entry in
/// entries.flatten()` loop — that is precisely the short-circuit this fix
/// removes. Do not collapse `AmbiguousFull` back to a single arbitrary
/// candidate; propagate the full set exactly like `walk_fs` does.
///
/// A first version of this fix collected every match into `child_matches`
/// and treated `len() > 1` alone as `AmbiguousFull` — this over-collects:
/// two real SIBLINGS whose names are unrelated except that one is a literal
/// text prefix of the other (e.g. `anchor` and `anchor__<...>`) produce a
/// SHORT candidate that spuriously satisfies the very same
/// `target.starts_with("{encoded}--")` boundary check as the true, longer,
/// correct candidate — not because the short one has any real topic-tagged
/// session, but because the LONG sibling's own real name, once encoded,
/// happens to textually extend the short one's encoding across what looks
/// like a topic boundary but is actually just the "__" mid-name separator
/// of a wholly unrelated real directory (confirmed: MAAV Round 12 Dimension
/// Adversary, re-run against the first version of this fix — a real
/// sibling's own untagged session at the long candidate wrongly leaked into
/// a query anchored at the unrelated short candidate). Unlike the Primary's
/// own collision fixture (two siblings with BYTE-IDENTICAL encodings — a
/// genuine, irreducible ambiguity), this shape has a real, more-specific
/// answer available: the long candidate's own match consumes strictly more
/// of `target`, and when it is an EXACT match it is definitionally correct
/// in a way a same-level sibling's merely-loose prefix match cannot
/// outrank.
/// Fix (revised): rank collected candidates by specificity
/// (`rank_subtree_candidates`) before treating a multi-candidate result as
/// genuinely ambiguous — an EXACT match (`encode_path(candidate) ==
/// target`) always outranks a loose (`target.starts_with("{encoded}--")`)
/// match, and among same-tier candidates the LONGEST `encode_path` output
/// wins (mirrors `walk_fs`'s own `Partial` disambiguation, which already
/// tie-breaks by consumed length rather than treating any 2+ partial
/// candidates as automatically tied). Only candidates that remain tied
/// after BOTH rankings — same exactness tier AND same encoded length, e.g.
/// the Primary's own byte-identical-encoding fixture — produce
/// `AmbiguousFull`.
/// Pitfall: do not rank by raw string content or `read_dir` order — only by
/// (`is_exact`, `encoded_len`), computed fresh via `encode_path` on each
/// collected candidate; a length-only rank without the exactness tier would
/// wrongly let a long LOOSE match outrank a short EXACT one on some other
/// input shape even though exact equality is strictly more certain than any
/// prefix heuristic.
///
/// Fix(BUG-527)
/// Root cause: the loose `target.starts_with("{encoded}--")` self-match
/// (Fix(BUG-522)) fired at EVERY level of the search, including the search
/// ROOT — the exact point `walk_fs` had already verified as the deepest
/// real prefix. When the query anchor is a DELETED intermediate directory
/// whose first component below a surviving real ancestor starts with a
/// special character (`.config`, `_build`, ...), that component's own `--`
/// leading-character escape makes the deleted path's encoding textually
/// indistinguishable from "the surviving ancestor's own encoding plus a
/// synthetic topic suffix": the root's loose self-match then promoted the
/// ancestor to a confident `Full(ancestor)`, silently stripping the
/// conservative-include disjunct (`base_path.starts_with(&p)`) that only a
/// `Partial` result carries — `matches_under`'s and `matches_local`'s
/// strict `Full` arms then EXCLUDED a session whose true project genuinely
/// sits under the deleted anchor (confirmed: MAAV Round 13 Fresh
/// Challenger — a regression introduced by Fix(BUG-522)'s own loose match;
/// the pre-BUG-522 exact-only search returned nothing in this shape,
/// leaving `Partial` and its conservative include intact). The loose match
/// alone cannot distinguish "candidate's own synthetic topic suffix" from
/// "a deeper deleted path's own special-leading component" — but at the
/// ROOT the walk has already answered exactly that question negatively (it
/// verified that no real descendant below the anchor consumes any more of
/// the target), while a DEEPER level's loose self-match remains
/// Fix(BUG-522)'s load-bearing mechanism: a real descendant the
/// children-first recursion actually reached is filesystem-verified
/// specificity, not a guess.
/// Fix: suppress the loose self-match at the search root only (the
/// `is_root` parameter, threaded through the recursion); exact equality
/// stays enabled at every level, and every proper-descendant level keeps
/// the loose `--`-boundary match unchanged.
/// Pitfall: do not remove the loose match entirely — that reintroduces
/// BUG-522's own `matches_local` nested-descendant leak (see IT-90); and do
/// not try to discriminate the two textual shapes by narrowing the suffix
/// pattern — a synthetic topic tag and an escaped special-leading component
/// are stringwise identical by construction, only the walk's own stall
/// point distinguishes them.
fn search_encoded_subtree( base : &std::path::Path, target : &str ) -> FsDecodeOutcome
{
  // Fix(BUG-527): the search ROOT is distinguished from every deeper level
  // for the loose `--`-boundary self-match — see this function's own
  // Fix(BUG-527) doc comment above.
  search_encoded_subtree_inner( base, target, &[ base.to_path_buf() ] )
}

/// Fix(BUG-530): rescue entry point for `decode_path_via_fs`'s own
/// `AmbiguousPartial` arm. The search still starts at the tied candidates'
/// common ancestor (the only single directory guaranteed to contain the
/// whole tied set), and marks EVERY tied candidate as a stall point for
/// `search_encoded_subtree_inner`'s own suppression logic, same as before —
/// but ALSO adds the ancestor (`base`) itself to that same stall-point set.
/// Root cause of the missing ancestor suppression (originally, incorrectly,
/// argued to be unnecessary — see `search_encoded_subtree_inner`'s own
/// current Fix(BUG-530) doc comment for the corrected reasoning): when both
/// tied candidates collide via a special-LEADING-character escape (e.g. `!x`
/// and `_x` both stripping to the same piece), that escape's own `--`
/// separator is, BY CONSTRUCTION, exactly the ancestor's encoding's own
/// trailing boundary — so the ancestor's encoding, extended by `--`, is
/// always a textual prefix of BOTH tied candidates' encodings in this shape,
/// not a coincidence specific to one fixture (confirmed: `/tst_fix`
/// continuation of the BUG-530 investigation, IT-96 — a genuine
/// byte-identical sibling collision via special-leading-character escape,
/// same fixture class BUG-523 established, newly regressed by BUG-530's own
/// incomplete fix). Since the ancestor is by definition SHALLOWER than the
/// tied candidates that `walk_fs` already found consuming strictly more of
/// `target`, a `Full(ancestor)` verdict is never more informative than the
/// tied set itself — the ancestor's loose match must be suppressed
/// unconditionally here, not only at the candidates.
///
/// Fix(BUG-531)
/// Root cause: the stall-point set covered only the tied candidates
/// themselves plus their common ancestor — never any REAL intermediate
/// directory strictly between the two. When one tied candidate sits at
/// GREATER depth than the other relative to `base` (an asymmetric-depth tie,
/// e.g. `base/mid/.a` tied against `base/b__a`), `mid` is a real directory
/// `search_encoded_subtree_inner`'s own children-first recursion walks
/// through en route to the deeper candidate, but `mid` is a member of
/// neither the candidate set nor the ancestor singleton — its own loose
/// `--`-boundary self-match stayed live, and since `mid`'s encoding is, by
/// construction, always a literal prefix of the deeper candidate's own
/// encoding, the same false-promotion shape Fix(BUG-527)/Fix(BUG-530)
/// eliminated at the documented stall points reappeared one level shallower,
/// at an undocumented non-stall-point neither fix's set was ever extended to
/// cover (confirmed: `/tst_fix` MAAV Round 15 re-verification, Primary).
/// Fix: walk from each tied candidate up to `base` via `Path::parent`,
/// adding every directory on that chain — not just the candidate's own
/// endpoint — to `stall_points`. `base` is guaranteed to terminate this walk
/// exactly (never `parent() == None` first) because `common_ancestor`
/// (this function's own caller) constructs `base` as a literal,
/// component-wise prefix ancestor of every candidate via repeated `pop()` on
/// a real `PathBuf`.
/// Pitfall: do not special-case only ONE level of intermediate nesting —
/// arbitrarily deep asymmetric ties (candidate reached via 2+ real
/// intermediate directories) need every one of them suppressed, not just the
/// immediate parent.
fn search_encoded_subtree_tied( base : &std::path::Path, target : &str, candidates : &[ std::path::PathBuf ] ) -> FsDecodeOutcome
{
  let mut stall_points = vec![ base.to_path_buf() ];
  for candidate in candidates
  {
    let mut cur = candidate.as_path();
    while cur != base
    {
      if !stall_points.iter().any( | p | p.as_path() == cur )
      {
        stall_points.push( cur.to_path_buf() );
      }
      cur = cur.parent().expect( "candidate is a real descendant of base per common_ancestor's own construction" );
    }
  }
  search_encoded_subtree_inner( base, target, &stall_points )
}

/// Recursive body of `search_encoded_subtree`/`search_encoded_subtree_tied`.
///
/// Fix(BUG-530)
/// Root cause: `is_root` was a single `bool`, true only at the search's own
/// top-level `base` — correct for the single-`Partial` rescue
/// (`search_encoded_subtree`'s `base` IS exactly where `walk_fs`'s
/// incremental matching stalled), but wrong for the `AmbiguousPartial`/tied
/// rescue: there, the search is anchored at the tied candidates' COMMON
/// ANCESTOR — a directory `walk_fs` matched UNAMBIGUOUSLY, one level
/// shallower than the real stall point, which occurred independently at
/// EACH tied candidate. A single root-level `bool` cannot mark 2+ non-root
/// children as stall-equivalent, so the loose `--`-boundary self-match
/// (Fix(BUG-522)) stayed live at the tied candidates themselves,
/// manufacturing an overconfident `Full`/`AmbiguousFull` out of a deleted
/// deep descendant's own special-leading component — exactly the
/// false-confidence shape Fix(BUG-527) eliminated for the single-candidate
/// case, surviving intact for the tied case (confirmed: `/tst_fix` MAAV
/// Round 14 re-verification, P2b).
/// Fix: replace `is_root : bool` with `stall_points : &[PathBuf]` — every
/// directory `walk_fs` actually stalled at (one entry via
/// `search_encoded_subtree`), PLUS the tied candidates' own common ancestor
/// (`search_encoded_subtree_tied` adds its own `base` to the set alongside
/// the candidates themselves — see that function's own doc comment). The
/// loose self-match is suppressed whenever the CURRENT recursion level's own
/// `base` is a member of that set, regardless of depth — every other level,
/// including any real descendant reached via the children-first loop below,
/// keeps the loose match exactly as Fix(BUG-522) intended.
/// Pitfall: do not suppress the loose match ONLY at the tied candidates,
/// omitting their common ancestor — an earlier version of this fix did
/// exactly that, reasoning the ancestor was "already genuinely,
/// unambiguously verified by `walk_fs`" and suppressing there "changes
/// nothing observable." That reasoning is false whenever the tied candidates
/// collide via a special-LEADING-character escape (IT-96): the escape's own
/// `--` separator IS the ancestor's encoding's own trailing boundary by
/// construction, so the ancestor's loose match fires there just as
/// spuriously as it did at the candidates pre-BUG-530, silently discarding
/// the tied set for an overconfident, strictly-less-specific `Full(ancestor)`.
/// The ancestor is never deeper than the candidates `walk_fs` already found
/// tied on consuming more of `target` — suppress it unconditionally too.
fn search_encoded_subtree_inner( base : &std::path::Path, target : &str, stall_points : &[ std::path::PathBuf ] ) -> FsDecodeOutcome
{
  let mut child_matches : Vec< std::path::PathBuf > = Vec::new();
  if let Ok( entries ) = std::fs::read_dir( base )
  {
    for entry in entries.flatten()
    {
      match search_encoded_subtree_inner( &entry.path(), target, stall_points )
      {
        FsDecodeOutcome::Full( p ) => { if !child_matches.contains( &p ) { child_matches.push( p ); } }
        FsDecodeOutcome::AmbiguousFull( found ) =>
        {
          for p in found { if !child_matches.contains( &p ) { child_matches.push( p ); } }
        }
        FsDecodeOutcome::Partial( _ ) | FsDecodeOutcome::AmbiguousPartial( _ ) | FsDecodeOutcome::NotFound => {}
      }
    }
  }
  if !child_matches.is_empty()
  {
    return rank_subtree_candidates( child_matches, target );
  }
  if let Ok( encoded ) = claude_storage_core::encode_path( base )
  {
    // Fix(BUG-527), Fix(BUG-530): the loose `--`-boundary self-match is
    // suppressed at every stall point (see this function's own Fix(BUG-530)
    // doc comment above) — there, `walk_fs` has already verified that no
    // real descendant consumes any more of `target` for that specific
    // candidate, so a stall-point-level loose match cannot distinguish a
    // genuine topic suffix from a deleted deeper path's own special-leading
    // component, and the overconfident `Full` it produced would silently
    // strip the conservative-include disjunct only `Partial`/`AmbiguousPartial`
    // carry. Exact equality stays enabled at every level; every OTHER level
    // keeps the loose match unchanged (Fix(BUG-522)).
    let is_stall_point = stall_points.iter().any( | p | p.as_path() == base );
    if encoded == target || ( !is_stall_point && target.starts_with( &format!( "{encoded}--" ) ) )
    {
      return FsDecodeOutcome::Full( base.to_path_buf() );
    }
  }
  FsDecodeOutcome::NotFound
}

/// Ranks `search_encoded_subtree`'s collected same-level candidates by
/// specificity — see that function's own Fix(BUG-523) doc comment for why a
/// flat `len() > 1 == ambiguous` collection over-collects unrelated
/// sibling-prefix matches. An exact `encode_path` match always outranks a
/// loose topic-boundary match; among same-tier candidates, the longest
/// `encode_path` output wins. Only candidates tied on BOTH axes are
/// genuinely ambiguous.
fn rank_subtree_candidates( candidates : Vec< std::path::PathBuf >, target : &str ) -> FsDecodeOutcome
{
  let mut ranked : Vec< ( std::path::PathBuf, usize, bool ) > = candidates.into_iter()
    .filter_map( | p |
    {
      let encoded = claude_storage_core::encode_path( &p ).ok()?;
      Some( ( p, encoded.len(), encoded == target ) )
    } )
    .collect();

  if ranked.is_empty()
  {
    return FsDecodeOutcome::NotFound;
  }

  let any_exact = ranked.iter().any( | ( _, _, exact ) | *exact );
  if any_exact
  {
    ranked.retain( | ( _, _, exact ) | *exact );
  }
  let max_len = ranked.iter().map( | ( _, len, _ ) | *len ).max().expect( "ranked non-empty" );
  ranked.retain( | ( _, len, _ ) | *len == max_len );

  if ranked.len() == 1
  {
    return FsDecodeOutcome::Full( ranked.into_iter().next().expect( "len checked == 1" ).0 );
  }
  FsDecodeOutcome::AmbiguousFull( ranked.into_iter().map( | ( p, _, _ ) | p ).collect() )
}

/// Fix(BUG-532)
/// Root cause: `decode_path_via_fs`'s `AmbiguousPartial` arm handed
/// `search_encoded_subtree_tied`'s `Full`/`AmbiguousFull` result straight
/// back to its caller, unconditionally REPLACING the whole original tied
/// candidate set. That rescue result is only ever reachable through ONE
/// original candidate's own subtree at a time — the recursion that found it
/// necessarily descended from a specific real directory — so wholesale
/// replacement silently discards every OTHER original candidate the rescue
/// neither confirmed nor disproved, even though `AmbiguousPartial`'s own
/// documented contract (`invariant/001_path_encoding.md`, Contract section)
/// is to conservatively include when at least one tied candidate still
/// qualifies (confirmed: `/tst_fix` MAAV Round 15 re-verification, Dimension
/// Adversary).
/// Fix: partition the original candidates into those COVERED by a found path
/// (the found path is the candidate itself or a real descendant of it —
/// `found.starts_with(candidate)`) and those left uncovered. When every
/// original candidate is covered, the found result already accounts for the
/// whole tied set — return it unchanged (`Full` singular, `AmbiguousFull`
/// otherwise), preserving today's behavior for the common case (e.g.
/// IT-96's byte-identical, equally-terminal sibling pair). When any
/// candidate is left uncovered, its own epistemic status is UNCHANGED by the
/// rescue — still merely a real, verified stall point with unknown content
/// below it, exactly as uncertain after the rescue as before it ran — so
/// merge it into the result as an `AmbiguousPartial` (never `AmbiguousFull`)
/// member alongside the found path(s), preserving its own ancestor-OR-
/// descendant conservative-include disjunct rather than silently dropping or
/// promoting it.
/// Pitfall: do not return `AmbiguousFull` for the merged result merely
/// because it also contains genuinely `Full`-confidence found path(s) —
/// `AmbiguousPartial` and `AmbiguousFull` differ in more than name
/// (`matches_local`'s own arms: `AmbiguousFull` checks equality alone,
/// `AmbiguousPartial` additionally allows `base_path.starts_with(candidate)`
/// — see that function's own Fix(BUG-526) doc comment); collapsing an
/// uncovered candidate into `AmbiguousFull` would silently strip the
/// conservative disjunct its own continued uncertainty still requires,
/// re-introducing the false-exclusion shape this fix exists to close for any
/// not-yet-discovered deleted content below it. A `Full`-confidence found
/// path folded into `AmbiguousPartial` loses nothing observable in exchange:
/// every relationship `AmbiguousFull` would have matched is a subset of what
/// `AmbiguousPartial` also matches (its checks are a strict superset), so
/// this never drops a case the stricter variant would have caught, only
/// widens a genuinely real, filesystem-confirmed leaf's inclusion to also
/// (rarely, harmlessly) cover a hypothetical query anchored strictly deeper
/// than that leaf's own confirmed-empty subtree.
fn merge_tied_rescue_findings( found : Vec< std::path::PathBuf >, original : Vec< std::path::PathBuf > ) -> FsDecodeOutcome
{
  let uncovered : Vec< _ > = original.into_iter()
    .filter( | c | !found.iter().any( | f | f.starts_with( c ) ) )
    .collect();
  if uncovered.is_empty()
  {
    return match found.len()
    {
      1 => FsDecodeOutcome::Full( found.into_iter().next().expect( "len checked == 1" ) ),
      _ => FsDecodeOutcome::AmbiguousFull( found ),
    };
  }
  let mut merged = found;
  for c in uncovered
  {
    if !merged.contains( &c ) { merged.push( c ); }
  }
  FsDecodeOutcome::AmbiguousPartial( merged )
}

/// Longest common ancestor of a non-empty candidate set, used to anchor
/// `decode_path_via_fs`'s single `search_encoded_subtree` rescue for
/// `FsDecodeOutcome::AmbiguousPartial` (Fix(BUG-526)): every tied candidate
/// lies somewhere under the level where `walk_fs` detected the tie, so
/// their common ancestor is the deepest point guaranteed to contain all of
/// them — and any truncation-hidden target among their extending siblings.
/// Component-wise (never raw-byte) comparison via `starts_with`/`pop`, so
/// `anc-foo` and `anc.foo` correctly yield their parent, never a string
/// prefix that is not itself a real directory.
fn common_ancestor( candidates : &[ std::path::PathBuf ] ) -> std::path::PathBuf
{
  let mut iter = candidates.iter();
  let Some( first ) = iter.next() else { return std::path::PathBuf::new() };
  let mut ancestor = first.clone();
  for p in iter
  {
    while !p.starts_with( &ancestor ) && ancestor.pop() {}
  }
  ancestor
}

/// Adds one verified-`Partial` candidate to `walk_fs`'s best-prefix
/// competition, keeping only the candidates holding the maximal consumed
/// length and collecting (rather than merely flagging) every distinct path
/// tied at that maximum — a genuine Partial-vs-Partial tie is reported as
/// `FsDecodeOutcome::AmbiguousPartial` with the full set preserved
/// (Fix(BUG-526)); see `walk_fs`'s own Fix(BUG-514)/Fix(BUG-526) doc
/// comments for the consumed-length metric and the no-collapse rationale.
fn consider_partial( best_candidates : &mut Vec< std::path::PathBuf >, best_consumed : &mut Option< usize >, p : std::path::PathBuf, consumed : usize )
{
  match *best_consumed
  {
    None =>
    {
      *best_consumed = Some( consumed );
      best_candidates.push( p );
    }
    Some( cur ) if consumed > cur =>
    {
      *best_consumed = Some( consumed );
      best_candidates.clear();
      best_candidates.push( p );
    }
    Some( cur ) if consumed == cur && !best_candidates.contains( &p ) => best_candidates.push( p ),
    _ => {}
  }
}

/// Recursive helper for `decode_path_via_fs`, resolving the encoding by
/// construction against REAL directory entries rather than guessing which
/// candidate character produced a run of hyphens.
///
/// At each step, list `base`'s actual directory entries; for each entry,
/// forward-encode that entry's own name via
/// `claude_storage_core::encode_component_piece` — the exact same
/// per-component rule `encode_path` itself calls to build the storage key in
/// the first place — and keep only the entries whose encoding is an actual
/// prefix of `remaining`. A match strips that prefix and recurses into the
/// entry with the leftover suffix; `is_first` flips to `false` after the
/// first component, matching `encode_path`'s own first-vs-rest branch. An
/// empty `remaining` at any point means the accumulated path is the answer.
///
/// Fix(BUG-511)
/// Root cause: `encode_path` only ever special-cases each component's own
/// LEADING character — nothing in the encoded output distinguishes a
/// mid-component run of special characters from a component boundary, says
/// how many special characters a run represents, or limits which
/// non-alphanumeric byte produced it. The previous design (options A-D; see
/// `git log` for the removed implementation) tried to invert this by
/// guessing: a fixed three-character candidate set (`.`, `_`, `-`) for "the"
/// special character, one candidate consumed per boundary. This was
/// incomplete in three compounding ways, each independently confirmed by a
/// MAAV Round 6 dimension agent: (1) any OTHER non-alphanumeric byte (e.g.
/// `!`, `@`) was never in the candidate set, even though `encode_path` (and
/// `docs/invariant/001_path_encoding.md`'s documented contract) treats every
/// non-alphanumeric character identically; (2) two or more CONSECUTIVE
/// leading special characters in one real component could never resolve,
/// because each option only ever substituted a single candidate character
/// per empty split-piece; (3) no option ever tried "this whole run of
/// hyphens is literal characters embedded inside one real component that is
/// never split at all" — so a same-level SIBLING directory whose name
/// happens to extend the anchor's own encoded prefix (e.g. anchor `sibfoo`
/// next to sibling `sibfoo--extra`) could be wrongly walked into as if it
/// were a nested descendant.
/// Fix: stop guessing candidate characters entirely. `encode_component_piece`
/// is the same function `encode_path` itself calls per component — so
/// instead of inverting the rule by hand, enumerate every REAL entry in
/// `base`, forward-encode each one's name with that same function, and keep
/// only the entries whose forward-encoding is an actual prefix of what is
/// left to decode. This is complete by construction for any non-alphanumeric
/// byte, any run length, and any component-boundary-vs-mid-component
/// ambiguity, because it never needs to guess what produced a run of
/// hyphens — it only ever checks what a real candidate entry's name WOULD
/// encode to, which is exactly the computation `encode_path` already
/// performed when the directory was first named.
/// Pitfall: do not reintroduce a hardcoded candidate-character list (`.`,
/// `_`, `-`, or any other finite set) for any reason, including performance
/// — the whole point of this design is that no finite set can ever be
/// complete against an encoding that normalizes ALL non-alphanumeric
/// characters identically. If the encoding rule itself changes, change
/// `encode_component_piece` (shared with `encode_path`) and this function
/// picks up the new rule automatically — never re-derive the rule locally.
///
/// Fix(BUG-512), Fix(BUG-513)
/// Root cause: this BUG-511 redesign was complete against the ambiguity
/// class it targeted (finite-candidate-character guessing) but incomplete
/// against two OTHER ambiguity sources it never considered: (a) a genuine
/// topic suffix (or a deleted/renamed descendant) leaving a nonempty,
/// unmatchable remainder after a REAL base was already fully verified —
/// the old `Option<PathBuf>` return type discarded that verified progress
/// entirely, returning bare `None`, indistinguishable from finding nothing
/// real at all; (b) `encode_path`'s separate 200-character truncation +
/// djb2-hash-suffix step, applied to the fully concatenated string, which
/// no per-component match can ever reconstruct.
/// Fix: return `FsDecodeOutcome` instead of `Option<PathBuf>`, tracking the
/// DEEPEST verified real path reached even when the full string isn't
/// consumed (fixes (a) — see `FsDecodeOutcome`'s own doc comment), and fall
/// back to `search_encoded_subtree`'s real-encoder-based exhaustive search
/// once incremental matching is stuck and `full_target` is long enough to
/// have been truncated (fixes (b)).
/// Pitfall: do not reintroduce a hardcoded candidate-character list (see the
/// Fix(BUG-511) pitfall above — still applies unchanged). Do not drop the
/// `best_partial` depth comparison in the loop below in favor of just
/// returning `base`'s own `Partial` on any non-`Full` result — among
/// multiple real sibling candidates, only the DEEPEST verified partial is a
/// meaningful (most-specific) proof; falling back to `base` itself throws
/// that specificity away (see BUG-512's own regression test for a
/// sibling-vs-anchor case where this exact simplification would reintroduce
/// the leak).
///
/// Fix(BUG-514)
/// Root cause: `best_partial`'s tie-break compared candidates by their
/// `PathBuf`'s raw OS-string BYTE length, using it as a proxy for "most
/// specific real match." But `encode_component_piece` normalizes every
/// non-alphanumeric character to exactly one ASCII `-` regardless of the
/// source character's own UTF-8 byte width — a component name built from
/// multi-byte characters (e.g. emoji, 3-4 bytes each) has a large `PathBuf`
/// byte length but encodes to relatively FEW bytes of `remaining` consumed,
/// while a shorter plain-ASCII sibling can consume MORE. Byte-length
/// comparison can therefore select the wrong candidate as "more specific,"
/// even discarding a candidate that is the true, most-specific real match
/// for the string actually being decoded (confirmed: MAAV Round 8 Fresh
/// Challenger, 10-underscore vs. 10-emoji sibling directories).
/// Fix: track how many bytes of the ENCODED `remaining` string each
/// candidate's own subtree actually consumed (`piece.len()` at this level
/// plus the callee's own reported consumed length, returned as this
/// function's second tuple element) and compare THAT instead of `PathBuf`
/// byte length — this is the metric that actually means "more specific" for
/// a decode of an encoded string. When two distinct real candidates consume
/// the identical number of encoded bytes, this is a genuine, unresolvable
/// tie — collapse to `base`'s own `Partial`, matching the conservative
/// inclusion `matches_local`/`matches_under` already apply to `NotFound`.
/// Pitfall: do not swap the byte-length metric for
/// `PathBuf::components().count()` (directory-tree depth) either — two
/// real, DIFFERENT siblings at the same tree depth (e.g. `anchor75` and
/// `anchor75_extra`, both direct children of the same parent) can consume
/// very different amounts of the encoded string, and treating them as tied
/// on depth alone silently reintroduces the BUG-512 sibling-leak this file
/// already fixed (hand-verified against IT-75's own fixture before writing
/// this fix — depth-based tie-break would have collapsed `anchor75`/
/// `anchor75_extra` to their shared parent, re-including the topic-suffixed
/// sibling IT-75 asserts must be excluded). Do not reintroduce the
/// `full_target: &str` parameter this function used to take — the single,
/// correctly-anchored `search_encoded_subtree` fallback now lives in
/// `decode_path_via_fs` instead (Fix(BUG-515), see that function's own doc
/// comment); this function no longer needs to know the untruncated target
/// at all.
///
/// Fix(BUG-516)
/// Root cause: a real sibling whose OWN name textually EXTENDS the winning
/// match's name (e.g. `ancX8` matches, `ancX8-extra-<220 z's>` is a real,
/// separate, non-nested sibling) can defeat BOTH Fix(BUG-514)'s tie-break
/// AND Fix(BUG-515)'s single-anchor fallback at once: it never becomes a
/// forward-matching candidate in this loop (its own piece is LONGER than
/// what fits in `remaining` once `encode_path`'s 200-char truncation has cut
/// the target off mid-way through encoding it), so there is no tie to
/// break — only the shorter match (`ancX8`) ever appears as a candidate at
/// all. And the resulting `Partial(ancX8)` is USELESS as a
/// `search_encoded_subtree` anchor in `decode_path_via_fs`, because the true
/// candidate (the sibling) is not inside `ancX8`'s own subtree — it is a
/// sibling of it, unreachable from that anchor no matter how the subtree is
/// searched. `matches_local`/`matches_under`/`matches_relevant` then
/// conservatively include the sibling's session under the shorter match, an
/// exact repeat of the BUG-512 sibling-leak this file already fixed, just
/// reintroduced through a gap neither BUG-514 nor BUG-515's fix covered
/// (confirmed: MAAV Round 8 Dimension Adversary, and the same construction
/// with no topic suffix at all — a sibling's OWN un-suffixed, exact
/// encoding, past the truncation boundary — regressed a case Fix(BUG-513)
/// itself originally covered, because the OLD code's call-`search_encoded_
/// subtree`-at-every-level approach incidentally found it from a SHALLOWER
/// level than the deepest partial, a property Fix(BUG-515)'s relocation to
/// a single deepest-anchor call silently dropped).
/// Fix: detect real siblings whose piece textually extends the winning
/// match's piece AND whose piece is itself LONGER than `remaining` — a
/// purely structural, in-memory string comparison against already-
/// `read_dir`'d entries, no filesystem search and no knowledge of the
/// untruncated target required — and let the LONGEST such extension win the
/// specificity comparison outright (its bare piece length always beats the
/// shorter match's total consumed length, so no change to the comparison
/// logic itself is needed, only to what counts as a candidate). This
/// relocates `walk_fs`'s own returned `Partial` anchor to the more specific
/// real sibling, so `decode_path_via_fs`'s EXISTING single
/// `search_encoded_subtree` call — already anchored at whatever `walk_fs`
/// returns — transparently gets a chance to resolve the sibling's own
/// subtree instead of the shorter match's dead-end one. No second call site
/// is introduced.
/// Pitfall 1: do not call `search_encoded_subtree` from inside this function
/// to verify an extension-sibling exactly — that would require re-threading
/// the untruncated target string back into `walk_fs`'s own recursion,
/// undoing Fix(BUG-515)'s entire point (exactly one call site, in
/// `decode_path_via_fs`). Let the extension win purely on the structural
/// piece-vs-piece comparison; `decode_path_via_fs`'s own existing fallback
/// call resolves (or fails to resolve) it exactly on its own, same as any
/// other returned `Partial` anchor.
/// Pitfall 2: the `piece.len() > remaining.len()` gate is NOT optional —
/// without it this fix regresses IT-76 (`scope::relevant`, mirror-direction
/// of IT-75): when the winning match's OWN name is itself a literal prefix
/// of a real sibling's name (e.g. `it76siba` matches, `it76siba_extra` is a
/// real, unrelated sibling that ALSO happens to be the query's own
/// `path::` target), that sibling always textually extends the winning
/// piece, but `remaining` here is short — never truncated — and the sibling
/// was ALREADY, definitively rejected as a forward-match candidate by this
/// function's own main loop above (its full piece does not literally
/// prefix-match `remaining`). Only a piece LONGER than `remaining` could
/// possibly have been truncation's doing; a shorter one that fails to
/// forward-match was never a truncation victim, just a non-match, and must
/// not be resurrected here.
///
/// Fix(BUG-524)
/// Two independent, compounding defects were found in the paragraph above's
/// extension-sibling mechanism, by two independent MAAV Round 12 dispatches:
///
/// (1) Root cause (Dimension Adversary): the `piece.len() > remaining.len()`
/// gate uses `remaining`'s own CURRENT length as a proxy for "was this piece
/// truncated by `encode_path`'s outer 200-char cut" — but once truncation
/// has actually happened, `remaining`'s tail is no longer real per-component
/// data at all; it is `encode_path`'s own appended `-<djb2-hash>` suffix
/// (`path.rs:241-245`) plus, when a topic tag was appended on top (the
/// standard `{encoded}--topic` convention), that tag's own bytes too. Both
/// inflate `remaining.len()` with bytes that belong to no real candidate's
/// piece at all, which can push it PAST a genuinely-truncated candidate's
/// own (shorter) untruncated piece length — silently defeating the gate for
/// the exact case it exists to catch (confirmed: a candidate whose own
/// untruncated piece is 68 bytes, while `remaining` — inflated by a 60+-byte
/// hash-plus-topic-tail — measures 70, so `68 > 70` is false and the
/// candidate that SHOULD be rescued never is).
///
/// (2) Root cause (Fresh Challenger): even when the gate DOES fire, the
/// candidate it selects is promoted with NO verification of its own
/// (no recursive `walk_fs` call — deliberately, see Pitfall 1 above — so
/// nothing confirms the promoted candidate is actually the correct target
/// rather than just the first structural match) and NO tie-detection when
/// 2+ real siblings simultaneously qualify (the loop's `piece.len() >
/// cur_consumed` comparison is a strict `>`, so among equal-length
/// qualifying candidates only the first one iterated ever wins — silent,
/// `read_dir`-order-dependent, unlike every OTHER ambiguity-resolution path
/// in this function, which all explicitly detect and report ties).
///
/// Fix: stop trying to pick a winning candidate via in-memory string-length
/// heuristics entirely. Instead, detect whether a real sibling might
/// plausibly have been cut off by the truncation boundary using an EXACT
/// check against `encode_path`'s own known, fixed 200-character constant —
/// `consumed_so_far + piece.len() > 199` (199, not 200: `total_len` is
/// measured on `inner`, which has already had the encoding's leading `-`
/// stripped by `decode_path_via_fs`, so the boundary shifts down by exactly
/// one) — rather than comparing against `remaining.len()`, which (1) above
/// proves is untrustworthy once truncation has occurred. `consumed_so_far`
/// is exact by construction (`total_len - remaining.len()`, and `remaining`
/// is always `inner` with a `consumed_so_far`-length real prefix already
/// stripped), so this check is immune to whatever garbage bytes actually
/// occupy the untrustworthy tail — it never inspects them at all. When this
/// fires for ANY real sibling other than the current `best` that also
/// textually extends `best`'s own piece, do not promote that sibling (or
/// any other candidate) — instead return `Partial(base)` unconditionally,
/// deferring to the PARENT level. This is always safe: `decode_path_via_fs`'s
/// existing single `search_encoded_subtree` fallback (Fix(BUG-515)), once
/// anchored at this returned `base`, searches base's ENTIRE real subtree —
/// every sibling, not just the one this function might have guessed — and,
/// since Fix(BUG-523), safely reports a genuine tie instead of guessing
/// among candidates the same way this function used to. Deferring is
/// strictly more correct than a heuristic guess and costs nothing extra:
/// the exact same single subtree search was always going to run once
/// `encoded.len() > 200`; only WHERE it is anchored changes.
/// Pitfall: do not reintroduce a length comparison against `remaining` for
/// this decision — (1) above is a proof, not a probabilistic argument, that
/// `remaining.len()` cannot be trusted once truncation has occurred. Do not
/// try to verify the deferred candidate from inside this function (see
/// Pitfall 1 on the original Fix(BUG-516) note — still applies unchanged:
/// exactly one `search_encoded_subtree` call site, in `decode_path_via_fs`).
///
/// Fix(BUG-525)
/// Root cause: Fix(BUG-524)'s boundary check compared `consumed_so_far +
/// piece.len()` against `encode_path`'s fixed 200-character truncation
/// boundary but never asked whether the encoding being decoded actually
/// REACHES that boundary — so a long real sibling piece (~195 bytes) at a
/// shallow level (`consumed_so_far` ~19) tripped the defer on a ~34-char
/// encoding where truncation is impossible by construction. `walk_fs` then
/// retreated to a `Partial` anchor one level SHALLOWER than the candidate
/// it had actually verified, and because `encoded.len() <= 200`,
/// `decode_path_via_fs`'s `search_encoded_subtree` rescue gate — whose
/// opening Fix(BUG-524)'s own defer rationale silently assumed ("the exact
/// same single subtree search was always going to run once
/// `encoded.len() > 200`", above) — never opened. The spuriously-retreated anchor flowed
/// straight to callers, whose conservative `Partial` disjuncts fired on it:
/// falsely INCLUDING an unrelated session via `matches_under`'s
/// `base_path.starts_with(&p)` (an underscore-sibling's ghost descendant
/// leaking into `scope::under`) and via `matches_local`'s
/// `p == base_path` (a nested project's topic-tagged session leaking into
/// the parent's `scope::local`, the BUG-509 class) (confirmed: MAAV Round
/// 13 Primary, two real-filesystem fixtures).
/// Fix: gate the defer on `total_len >= 200` — `total_len` is `inner.len()`
/// = `encoded.len() - 1`, so this is exactly `decode_path_via_fs`'s own
/// `encoded.len() > 200` rescue-gate condition restated. The defer now
/// fires ONLY in the regime where the rescue search is guaranteed to run,
/// which is precisely what Fix(BUG-524)'s zero-cost defer rationale always
/// assumed.
/// Pitfall: do not drop this guard to "tighten" the boundary arithmetic —
/// an UNRESCUED defer is strictly worse than none at all, and any defer
/// that can fire while `encoded.len() <= 200` is unrescued by construction.
///
/// Fix(BUG-526)
/// Root cause: Fix(BUG-514)'s tie handling collapsed a genuine
/// Partial-vs-Partial tie to `Partial(base)` — the tied candidates' shared
/// parent — discarding exactly which real subtrees were in play. That is
/// the precise false-inclusion shape Fix(BUG-518)'s continued note above
/// already eliminated for the FULL arm (`AmbiguousFull`): a caller's
/// conservative ancestor check (`matches_under`'s
/// `base_path.starts_with(&p)`) against the overly-broad shared parent is
/// satisfied even when NEITHER tied candidate relates to the query. The
/// Partial arm survived unpatched because BUG-518 only amended the
/// `full_matches` path — and a synthetic `--topic` suffix (Fix(BUG-512)'s
/// mechanism) forces resolution through the Partial arm by preventing
/// `remaining.is_empty()` from ever being reached (confirmed: MAAV Round 13
/// Dimension Adversary — real siblings `anc-foo`/`anc.foo` with identical
/// encodings tied at equal consumed length; collapsing to their shared
/// parent leaked the unrelated sibling's session into the query anchor's
/// `scope::under` results).
/// Fix: preserve the tied candidate set as
/// `FsDecodeOutcome::AmbiguousPartial` (Partial semantics per candidate —
/// each is an incomplete prefix, not the complete resolutions
/// `AmbiguousFull` carries; see its own doc comment) and push the
/// relationship check to each caller, exactly as Fix(BUG-518) did for
/// `AmbiguousFull`. Consumed-length credit reported to the parent call is
/// still 0 on a tie, unchanged from the collapse version: a tie's deeper
/// consumption is ambiguous, so it must not outrank an undisputed sibling's
/// verified progress at the parent level. Fix(BUG-524)'s defer block below
/// now runs only for a single undisputed best — a tied set is preserved
/// verbatim, and the rescue search from the tied candidates' common
/// ancestor (see `decode_path_via_fs`'s own Fix(BUG-526) note) covers any
/// truncation victim among their extending siblings.
/// Pitfall: do not re-collapse this set to `Partial(base)` anywhere, and do
/// not reuse `AmbiguousFull` for it — callers apply STRICT per-candidate
/// checks to `AmbiguousFull` (each candidate fully resolved the encoding),
/// which would falsely EXCLUDE the conservative `base_path.starts_with(&p)`
/// direction these incomplete prefixes legitimately require (see IT-79's
/// fixture: the query anchor itself sits inside the tied set — per-candidate
/// `Partial` semantics keep it included exactly as the pre-fix collapse
/// did, with no shared-parent broadening).
fn walk_fs( base : &std::path::Path, remaining : &str, is_first : bool, total_len : usize ) -> ( FsDecodeOutcome, usize )
{
  if remaining.is_empty() { return ( FsDecodeOutcome::Full( base.to_path_buf() ), 0 ); }
  let Ok( entries ) = std::fs::read_dir( base ) else { return ( FsDecodeOutcome::NotFound, 0 ) };

  // Collected once (rather than streamed) so the Fix(BUG-516) extension
  // check below can cross-reference every real sibling at this level
  // regardless of `read_dir`'s own (unspecified, platform-dependent) order.
  let candidates : Vec< ( std::path::PathBuf, String ) > = entries
  .flatten()
  .filter_map( | entry |
  {
    let name = entry.file_name();
    let name_str = name.to_str()?;
    Some( ( entry.path(), claude_storage_core::encode_component_piece( name_str, is_first ) ) )
  } )
  .collect();

  // Fix(BUG-518): a `Full` result used to win the whole call on its FIRST
  // occurrence. But another, not-yet-tried sibling candidate later in this
  // same loop can ALSO recursively resolve `remaining` to `Full` at a
  // DIFFERENT real path whenever `encode_component_piece` collides across
  // sibling names spanning a component boundary (the same collision class
  // Fix(BUG-514) already tie-breaks for Partial-vs-Partial) — so returning
  // on the first hit silently depended on `std::fs::read_dir`'s
  // platform-unspecified enumeration order rather than any documented rule.
  //
  // Fix(BUG-518) (continued — Partial(base) collapse was itself too lossy)
  // Root cause: the first version of this fix collapsed a detected tie to
  // `Partial(base)` (`base` = the tied candidates' own common parent). But
  // a Full-tie is not the same shape of uncertainty as a Partial-vs-Partial
  // tie: each tied candidate is already a COMPLETE resolution of the whole
  // encoded string, not an incomplete prefix — collapsing to their shared
  // parent discards exactly which real paths are in play, and a caller's
  // own ancestor check (`matches_under`'s `base_path.starts_with(&p)`)
  // against that overly-broad shared parent can be satisfied even when
  // NEITHER individual tied candidate relates to the query (confirmed:
  // MAAV Round 9 Fresh Challenger's own probe fixture, where `anchor-foo`
  // and `anchor.foo` — both real siblings of query anchor `anchor`, neither
  // nested under it — collide with each other one level shallower than the
  // probe's own intended three-way `bar`-child collision; collapsing to
  // their shared parent `fc9parent` falsely satisfied
  // `anchor.starts_with(fc9parent)`, including both siblings' sessions
  // under `scope::under(anchor)` even though neither individually
  // qualifies).
  // Fix: preserve the full tied-candidate set as `FsDecodeOutcome::
  // AmbiguousFull` (see its own doc comment) instead of collapsing to a
  // single `Partial` path, and push the relationship check down to each
  // caller, which checks its own predicate against EVERY tied candidate
  // and includes only when at least one satisfies it.
  // Pitfall: do not re-collapse `AmbiguousFull` back to a single `Partial`
  // anchor anywhere in this file. A nested call's own `AmbiguousFull` must
  // be flattened into the current level's `full_matches` (not treated as a
  // single opaque `Full`/discarded) so a multi-level collision still
  // surfaces its complete real leaf-candidate set at the top.
  let mut full_matches : Vec< std::path::PathBuf > = Vec::new();
  // Fix(BUG-526): a Partial-vs-Partial tie at this level is no longer
  // collapsed to `Partial(base)` (see this function's own Fix(BUG-526) doc
  // comment above) — tied candidates are COLLECTED with their consumed
  // lengths instead of tracking a single `best` plus a bare `tied` flag, so
  // the full set can be returned as `AmbiguousPartial`. A nested call's own
  // `AmbiguousPartial` is flattened into this same competition exactly like
  // `AmbiguousFull` above: each tied leaf competes on its own merits.
  let mut best_candidates : Vec< std::path::PathBuf > = Vec::new();
  let mut best_consumed : Option< usize > = None;
  for ( path, piece ) in &candidates
  {
    let Some( rest ) = remaining.strip_prefix( piece.as_str() ) else { continue };
    match walk_fs( path, rest, false, total_len )
    {
      ( FsDecodeOutcome::Full( p ), _ ) =>
      {
        if !full_matches.contains( &p ) { full_matches.push( p ); }
      }
      ( FsDecodeOutcome::AmbiguousFull( inner ), _ ) =>
      {
        for p in inner { if !full_matches.contains( &p ) { full_matches.push( p ); } }
      }
      ( FsDecodeOutcome::Partial( p ), inner_consumed ) =>
      {
        consider_partial( &mut best_candidates, &mut best_consumed, p, piece.len() + inner_consumed );
      }
      ( FsDecodeOutcome::AmbiguousPartial( inner ), inner_consumed ) =>
      {
        for p in inner
        {
          consider_partial( &mut best_candidates, &mut best_consumed, p, piece.len() + inner_consumed );
        }
      }
      ( FsDecodeOutcome::NotFound, _ ) => {}
    }
  }

  // An unambiguous Full match (never conflicted with another real path) is
  // strictly more informative than any Partial candidate and wins outright.
  // 2+ distinct full matches are STILL more informative than any Partial
  // candidate (each is a complete resolution, not a prefix) — see
  // Fix(BUG-518)'s continued doc comment above for why this returns the
  // full tied set rather than collapsing to a shared-ancestor Partial.
  match full_matches.len()
  {
    1 => return ( FsDecodeOutcome::Full( full_matches.into_iter().next().expect( "len checked == 1" ) ), 0 ),
    n if n > 1 => return ( FsDecodeOutcome::AmbiguousFull( full_matches ), 0 ),
    _ => {}
  }

  // Fix(BUG-524), Fix(BUG-525): rather than guessing which real sibling
  // should win via an in-memory string-length heuristic (the original
  // Fix(BUG-516) design, proven unsound by both its missing
  // verification/tie-detection and its corruptible `remaining.len()` gate —
  // see this function's own Fix(BUG-524) doc comment above), detect whether
  // ANY real sibling might plausibly have been cut off by `encode_path`'s
  // 200-char truncation using an exact check against that known, fixed
  // boundary, and defer entirely to the parent level when so — never
  // promote a guessed candidate. Fix(BUG-525): the defer is gated on
  // `total_len >= 200`, so it can only fire when `decode_path_via_fs`'s
  // rescue search is guaranteed to run. Fix(BUG-526): the defer runs only
  // for a single undisputed best — a tied set is preserved verbatim as
  // `AmbiguousPartial` below.
  //
  // Fix(BUG-529): `total_len >= 200` alone answers "is the STORED KEY long
  // enough", not "could `encode_path`'s truncation have actually fired on
  // the REAL PATH". The two diverge whenever a synthetic `--topic` suffix
  // (Fix(BUG-512)'s mechanism, appended AFTER `encode_path` already ran) is
  // long enough to push `total_len` past 200 on its own, even though the
  // real path's own encoding is short and genuinely complete (confirmed:
  // MAAV Round 14 Dimension Adversary — a deleted-project ghost session
  // whose own path encoding is well under 200 chars, paired with a long
  // synthetic topic tag, false-included a session under an unrelated
  // anchor through this exact gap).
  //
  // A first attempt gated the defer on the WINNING candidate's own verified
  // depth (`consumed_so_far + winning_piece.len() >= 199`) — the wrong
  // candidate to key on. It is a COMPETING (non-winning) candidate whose
  // piece extends the winner's that might be a truncation victim, never the
  // winner itself, which is already a complete, verified forward-match.
  // Gating on the winner's own depth regressed the BUG-516 extension-sibling
  // class (IT-80/81/83) and this file's own IT-110: there, the winning
  // candidate is a short, verified real anchor — just as shallow as this
  // fixture's `main_extra` — so a signal keyed to the winner's own depth
  // cannot tell the two shapes apart, and blocking the defer for one
  // necessarily blocks it for the other.
  //
  // Fix: measure how much of `remaining` ITSELF (not the winning piece) a
  // competing candidate's own on-disk piece can account for before the two
  // diverge. `remaining`'s real content ends at a FIXED position —
  // `inner`'s first 199 chars, before the djb2-hash suffix and any topic
  // tag riding after it — so once a competitor's piece agrees with
  // `remaining` all the way out to that boundary, `encode_path`'s
  // truncation could plausibly have cut its own tail short: genuine
  // ambiguity worth deferring for. When a competitor's agreement with
  // `remaining` ends well short of the boundary — a real but unrelated
  // sibling that merely shares a short textual prefix with the winner, e.g.
  // this fixture's `decoy` — the defer must not fire: there is no
  // truncation in play, just an early, definitive mismatch.
  // Pitfall: do not key this check to `winning_piece` at all, neither its
  // own length nor as a `starts_with` filter on the competing candidate —
  // the winner's own depth says nothing about whether ANOTHER candidate was
  // truncated, and requiring textual overlap with the winner specifically
  // excludes nothing a direct `remaining`-agreement check doesn't already
  // cover more precisely.
  if best_candidates.len() == 1
  {
    let winning_path = &best_candidates[ 0 ];
    let consumed_so_far = total_len - remaining.len();
    let truncation_ambiguous = total_len >= 200 && candidates.iter().any( | ( path, piece ) |
      path != winning_path && consumed_so_far + common_prefix_len( piece, remaining ) >= 199
    );
    if truncation_ambiguous
    {
      return ( FsDecodeOutcome::Partial( base.to_path_buf() ), 0 );
    }
  }

  // Fix(BUG-526): a genuine tie (2+ distinct real candidates at the same
  // maximal consumed length) preserves the full set as `AmbiguousPartial`
  // rather than collapsing to `Partial(base)` — see this function's own
  // Fix(BUG-526) doc comment above.
  //
  // Fix(BUG-528): consumed-length credit reports the tie's own verified
  // `best_consumed` value, not a hardcoded 0. BUG-526's original credit
  // conflated two DIFFERENT kinds of ambiguity — WHICH candidate wins the
  // tie is genuinely ambiguous, but HOW MANY bytes every tied candidate
  // consumed is not (by construction, every entry in `best_candidates`
  // consumed exactly `best_consumed` bytes — that equality is what makes it
  // a tie). Reporting 0 discarded a verified quantity because a different,
  // unrelated quantity was unresolved, starving this function's own
  // per-child `consider_partial` competition above: a nested
  // `AmbiguousPartial` propagated at credit 0 could never outrank an
  // unrelated shallower sibling with ANY nonzero partial progress, even
  // when the tied subtree's real, undisputed depth was far greater.
  // Pitfall: do not revert to crediting 0 here — the ambiguity this arm
  // exists to preserve is the winning PATH's identity (`AmbiguousPartial`'s
  // own `Vec<PathBuf>`), never the consumed count.
  match best_candidates.len()
  {
    0 => ( FsDecodeOutcome::Partial( base.to_path_buf() ), 0 ),
    1 => ( FsDecodeOutcome::Partial( best_candidates.into_iter().next().expect( "len checked == 1" ) ), best_consumed.expect( "set when candidates non-empty" ) ),
    _ => ( FsDecodeOutcome::AmbiguousPartial( best_candidates ), best_consumed.expect( "set when candidates non-empty" ) ),
  }
}

/// Decode a storage dir name to the longest real filesystem path it represents.
///
/// # Why the `starts_with('-')` guard
///
/// `decode_path()` requires its input to be a valid path-encoded string. UUID project
/// directories (e.g. `deadbeef-1234-...`) do not start with `-` and are NOT path-encoded.
/// Calling `decode_path` on a UUID returns `Err` — but more importantly, it would be
/// semantically wrong. UUID dirs represent web/IDE sessions without filesystem paths.
/// The guard ensures they fall through to the raw string return at the end.
///
/// # Topic components: metadata vs real directories
///
/// Topic-scoped project dirs are named `-path--topic` (double dash before topic).
/// Topics are often pure metadata tags (e.g. `--commit`), but they can also be real
/// hyphen-prefixed directories (e.g. `--default-topic` → `-default_topic/`).
///
/// Examples:
/// - `-...-src--default-topic`         → `src/-default_topic`
/// - `-...-src--default-topic--commit` → `src/-default_topic/-commit`
/// - `-...-src--commit`                → `src/-commit`
///
/// # Why a single `decode_storage_base` call is sufficient
///
/// Fix(BUG-003)
/// Root cause: this used to call `split_storage_key` to break `dir_name` into a base
/// component plus a list of `--topic` components, then re-join each topic as a
/// separate `-{topic}` path segment. That split relied on the same unsound blind
/// `find("--")` search removed from `is_relevant_encoded` for the same reason — it
/// could not tell a genuine topic boundary from incidental shared structure.
/// Fix: `claude_storage_core::decode_path`'s own heuristic already chains multiple
/// `--`-separated segments correctly on its own (each `--` starts a new
/// hyphen-prefixed segment while the rest of that segment maps `-` → `_`), so passing
/// the whole `dir_name` straight through reconstructs the same multi-topic display
/// path with no external split/append loop needed.
/// Pitfall: do not reintroduce a `--`-splitting loop here — `decode_storage_base`
/// (via `decode_path`) already handles the full string, topics included.
///
/// # Why the filesystem fallback for the base
///
/// Fix(issue-029)
/// Root cause: `decode_path` heuristic defaults to path separator `/` for all
/// unrecognized `-` boundaries. Paths with underscore-named dirs (e.g. `my_project`,
/// `claude_tools`) display incorrectly as `wip/core`, `claude/tools`.
/// Pitfall: Only call the filesystem walk as fallback — never primary — because it
/// requires the project directory to exist on disk. Deleted/remote projects fall
/// back to the raw encoded storage dir name.
pub( super ) fn decode_project_display( dir_name : &str ) -> String
{
  if !dir_name.starts_with( '-' ) { return dir_name.to_string(); }
  let Some( path ) = decode_storage_base( dir_name ) else { return dir_name.to_string() };
  tilde_compress( &path )
}

// ─── scope validation ──────────────────────────────────────────────────────

/// Validate and lowercase a `scope::` parameter value.
///
/// Accepts (case-insensitively): `relevant`, `local`, `under`, `global`, `around`.
/// When `scope_raw` is `None`, falls back to `default` (assumed already valid —
/// every call site passes one of the 5 accepted values as its own default).
///
/// # Errors
///
/// Returns `ErrorData` when `scope_raw` is `Some` but not one of the 5 valid values.
/// The error message embeds the raw (pre-lowercase) input, matching
/// `docs/cli/param/12_scope.md`'s documented `relevant|local|under|global|around`
/// word order.
pub( crate ) fn validate_scope( scope_raw : Option< &str >, default : &str ) -> core::result::Result< String, ErrorData >
{
  let raw = scope_raw.unwrap_or( default );
  let scope = raw.to_lowercase();
  if !matches!( scope.as_str(), "local" | "relevant" | "under" | "global" | "around" )
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "scope must be relevant|local|under|global|around, got {raw}" ),
    ) );
  }
  Ok( scope )
}

// ─── scoped project resolution ─────────────────────────────────────────────

/// Resolve the scope anchor: `path_raw` when given, else the current directory.
///
/// Shared by `resolve_scoped_projects` and `.usage`'s depth filter, which both
/// anchor on the same `path::`-or-cwd base.
///
/// # Errors
///
/// Returns `ErrorData` when path resolution fails or cwd is unreadable.
pub( super ) fn resolve_base_path( path_raw : Option< &str > ) -> core::result::Result< std::path::PathBuf, ErrorData >
{
  if let Some( p ) = path_raw
  {
    resolve_path_parameter( p )
      .map( std::path::PathBuf::from )
      .map_err( | e | ErrorData::new(
        ErrorCode::InternalError,
        format!( "Failed to resolve path '{p}': {e}" ),
      ) )
  }
  else
  {
    std::env::current_dir()
      .map_err( | e | ErrorData::new(
        ErrorCode::InternalError,
        format!( "Failed to get current directory: {e}" ),
      ) )
  }
}

/// Resolve the set of projects matching `scope`, anchored at `path_raw` (or cwd
/// when `path_raw` is `None`).
///
/// Scope semantics:
/// - `local`    — the anchor project only
/// - `relevant` — every project whose path is an ancestor of (or equal to) the anchor
/// - `under`    — every project whose path is the anchor or nested under it
/// - `around`   — union of `under` + `relevant`
/// - `global`   — all projects in storage (anchor ignored)
///
/// `scope` must already be validated (see `validate_scope`) — an unrecognized
/// value falls through every match arm and yields an empty result rather than
/// an error, matching the original `.projects` closure's own fallback.
///
/// # Errors
///
/// Returns `ErrorData` when path resolution or storage access fails.
pub( crate ) fn resolve_scoped_projects(
  storage  : &Storage,
  scope    : &str,
  path_raw : Option< &str >,
) -> core::result::Result< Vec< Project >, ErrorData >
{
  let all_projects = storage.list_projects()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list projects: {e}" ) ) )?;

  if scope == "global"
  {
    return Ok( all_projects );
  }

  let base_path = resolve_base_path( path_raw )?;

  // Fix(issue-024)
  // Root cause: encode_path() maps both '_' and '/' to '-', so decode_component()
  // defaults unknown pairs to '/', turning `my_project` → `wip-core` → `wip/core`.
  // Decoded paths never match the real base_path, causing silent 0-result returns.
  // Pitfall: Never decode storage dir names for path comparison — encoding is
  // deterministic but decoding is lossy. Compare encoded ↔ encoded instead.
  let encoded_base = claude_storage_core::encode_path( &base_path )
    .map_err( | e | ErrorData::new(
      ErrorCode::InternalError,
      format!( "Failed to encode base path '{}': {e}", base_path.display() ),
    ) )?;

  Ok(
    all_projects
      .into_iter()
      .filter( | p | project_matches( p, scope, &encoded_base, &base_path ) )
      .collect()
  )
}

/// Does `project` qualify under `scope`, anchored at `base_path`/`encoded_base`?
///
/// Compares encoded base against raw storage directory name — no decode step.
/// UUID project dirs start with a hex character (not `-`), so they never match
/// path-based comparisons and are correctly excluded from non-global scopes.
/// `global` never reaches here (`resolve_scoped_projects` early-returns).
fn project_matches(
  project      : &Project,
  scope        : &str,
  encoded_base : &str,
  base_path    : &std::path::Path,
) -> bool
{
  let dir_name = project
    .storage_dir()
    .file_name()
    .and_then( | n | n.to_str() )
    .unwrap_or( "" );
  match scope
  {
    // Fix(BUG-509): delegate to matches_local (filesystem-verified), replacing
    // a naive starts_with("{eb}--") check that could not distinguish a real
    // nested project from a topic-suffix alias of the anchor itself.
    "local" => matches_local( dir_name, encoded_base, base_path ),
    // Fix(issue-031)
    // Root cause: starts_with on encoded strings cannot distinguish a child
    //   directory (base/sub → `base-sub`) from a same-level sibling whose name
    //   uses an underscore (base_extra → `base-extra`): both share the `base-`
    //   prefix. Path::starts_with is component-wise and correctly excludes siblings.
    // Pitfall: strip the `--topic` suffix from dir_name before calling
    //   decode_path_via_fs. The `--topic` part encodes a hyphen-prefixed directory
    //   like `-default_topic`; left in place, the walker searches for a dir named
    //   `topic` under the project root, returns None, and the fallback silently
    //   includes everything — the sibling exclusion is bypassed.
    "under" => matches_under( dir_name, encoded_base, base_path ),
    // Fix(issue-032)
    // Root cause: is_relevant_encoded uses string starts_with to check if
    //   dir_name's encoded path is a prefix of encoded_base, so a sibling
    //   `base` (encoded `base-`) falsely matches when base_path is `base_extra`
    //   (encoded `base-extra`). Both `_` and `/` map to `-`, making siblings
    //   indistinguishable from ancestors by string comparison alone.
    //   base_path.starts_with(decoded_path) is component-wise and rejects siblings.
    // Pitfall: strip the `--topic` suffix before calling decode_path_via_fs —
    //   same requirement as the issue-031 fix for scope::under.
    "relevant" => matches_relevant( dir_name, encoded_base, base_path ),
    // Union of under + relevant — bidirectional neighborhood.
    "around" =>
      matches_under( dir_name, encoded_base, base_path )
        || matches_relevant( dir_name, encoded_base, base_path ),
    _ => false,
  }
}
