# Algorithm: Path Encoding

### Scope

- **Purpose**: Encode filesystem paths as directory names in Claude Code's storage layout and decode them back reliably.
- **Responsibility**: Documents the single lossy encoding scheme this library implements to match Claude Code's own encoding, the decoding heuristic, and their known limitations.
- **In Scope**: Encoding rules, decoding heuristic, lossiness rationale, known limitations.
- **Out of Scope**: Project type distinction (→ `data_structure/001_storage_hierarchy.md`), filesystem operations.

### Abstract

Claude Code stores CLI sessions in directories whose names are filesystem-safe encodings of the project's working directory path. This library implements the same encoding to locate sessions by path. The encoding is fundamentally **lossy**: `/` (path separator), `_` (underscore), and `-` (hyphen already in a name) all collapse to `-` in the encoded output, so the decoder can only reconstruct the original path heuristically.

### Algorithm

<!-- BUG-366 ../../../task/claude_storage_core/bug/unverified/366_encode_path_dot_handling_divergence.md — documents only 3 substitutions (/, _, existing -); missing dot/non-alphanumeric handling and the 200-char/hash fallback the real Claude Code algorithm uses -->

**Encoding:**
1. Strip leading and trailing `/` from the input path.
2. Split on `/` into path components. Error if the result is empty (e.g., root path `/`) or the path is not valid UTF-8.
3. Replace every `_` (underscore) with `-` (hyphen) in each component — lossy; the original underscore is not recoverable from the encoded form.
4. Prefix the entire result with `-`.
5. For each component after the first, insert a separator before it:
   - If the (post-normalization) component begins with `-`: insert `--` (double-hyphen), then the component body without its leading `-`.
   - Otherwise: insert a single `-`, then the component as-is.
   - The first component follows the same rule, using a single `-` for the leading marker instead of a path separator.

Example: `/commands/-default_topic` → `-commands--default-topic` (the underscore in `default_topic` becomes a hyphen, same as the `/` separators).

`--` unambiguously means "path component starting with hyphen", which resolves the ambiguity where `-a--b-c` could otherwise be read as `/-a_b/c` or `/-a_b_c`.

**Decoding (heuristic — the encoding is lossy, so decoding cannot be exact):**
- `--` (double hyphen) → `/-` (component starting with hyphen).
- `-` within a hyphen-prefixed component → `_` (underscore restoration, heuristic — the decoder assumes hyphen-prefixed directories use underscores internally, matching real-world Claude Code usage like `-default_topic`).
- `-` after a normal component → `/` (path separator) by default, with two positional exceptions the decoder applies to reconstruct conventional layouts (see `decode_component()`, `src/path.rs`):
  - **Crate-name reconstruction**: immediately after a `module`/`modules` segment, the hyphen introducing the next segment decodes to `/` (the `module/` boundary), and the hyphen joining that segment's second word decodes to `_` — reconstructing an underscore-joined crate name for exactly **two-word** crate names (see `test_real_world_my_agent_path` in `src/path.rs`, which decodes a `module`-adjacent `my-agent` segment to `my_agent`, not `my/agent`). **Known limitation**: for three-or-more-word hyphenated crate names, only the first two words join with `_`; every word from the third onward falls through to the generic "after module name" rule below and becomes its **own** separate `/`-delimited segment — an N-word crate name produces **N-2 independent splits**, not a single trailing split regardless of N. For example: `my-cool-agent` (3 words) decodes to `my_cool/agent` (1 split), but `my-super-cool-agent` (4 words) decodes to `my_super/cool/agent` (2 splits) and a 5-word name produces 3 splits — each additional word beyond the second adds one more `/` boundary rather than collapsing into a single fallthrough split.
  - **Unknown-segment preservation**: for hyphens earlier in the component (before any `module`/`modules` segment), the decoder treats the hyphen as `/` only when at least one adjacent segment is a recognized filesystem/project keyword (`home`, `usr`, `opt`, `tmp`, `var`, `etc`, `bin`, `lib`, `src`, `projects`, `user`, `root`, `module`, `modules`, `crates`, `crate`, `tests`, `examples`); otherwise the hyphen is preserved literally, assuming it belongs to a real hyphenated name rather than a path boundary. When no `module`/`modules` segment exists in the component, every hyphen defaults to `/`.

**Historical note:** an earlier version of this library (v1.0.1) preserved underscores instead of replacing them, which diverged from Claude Code's actual encoding and caused session lookups to fail for any path containing an underscore (see `tests/underscore_encoding_compatibility.rs` for the full root-cause record). The fix made `encode_path()` replace `_` with `-`, matching Claude Code's behavior exactly. There is no alternate underscore-preserving mode today — the algorithm above is the only one this library implements.

**Known limitation:** decoding is a lossy inverse — `/foo-bar`, `/foo_bar`, and `/foo/bar` all encode to `-foo-bar`, so the decoder cannot always distinguish which original character produced a given `-`. The decoder's heuristics (above) pick the most likely original form; they are not guaranteed correct for every encoded string.

### Edge Cases

| Scenario | Behavior |
|----------|----------|
| Root path `/` | Error: empty after normalization |
| Path with non-UTF-8 bytes | Error: UTF-8 validation failure |
| Hyphen-prefixed component (`/-name`) | Double-hyphen prefix in output: `--name` |
| Underscore in component (`my_project`) | Lossy: becomes `my-project`; decoder uses heuristics |
| Consecutive hyphen-prefixed dirs (`/-a/-b`) | `--a--b` — each gets a `--` prefix |
| Path with literal hyphen (`/foo-bar`) | `-foo-bar` — indistinguishable from `/foo/bar` encoded |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/path.rs` | encode_path(), decode_path() implementations |
| test | `../../tests/path_encoding_double_slash_bug.rs` | Bug reproducer for double-slash ambiguity fix |
| test | `../../tests/path_decoding_hyphen_component_bug.rs` | Hyphen-prefixed component decoding correctness |
| test | `../../tests/underscore_encoding_compatibility.rs` | Regression tests for the fixed v1.0.1 underscore-preservation defect — encoder/decoder lossy-transform agreement |
| doc | `../data_structure/001_storage_hierarchy.md` | Project type that uses path encoding for directory names |
| doc | `../feature/004_continuation_detection.md` | Continuation detection — uses Df() to locate prior session storage |
| doc | `../invariant/001_safety_guarantees.md` | Path safety invariant — encode_path()/decode_path() prevent path traversal |
| doc | `../api/001_public_api.md` | Public API surface — exposes encode_path()/decode_path() |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Combined specification; path encoding section extracted here |
