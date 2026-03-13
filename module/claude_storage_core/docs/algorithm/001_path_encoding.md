# Algorithm: Path Encoding

### Scope

- **Purpose**: Encode filesystem paths as directory names in Claude Code's storage layout and decode them back reliably, including backward compatibility with Claude Code's original lossy encoding.
- **Responsibility**: Documents the v1 (legacy) and v2 (current) encoding schemes, the decoding heuristic, and their backward-compatibility guarantee.
- **In Scope**: Encoding rules for both versions, decoding heuristic, asymmetry rationale, known limitations.
- **Out of Scope**: Project type distinction (→ `data_structure/001_storage_hierarchy.md`), filesystem operations.

### Abstract

Claude Code stores CLI sessions in directories whose names are filesystem-safe encodings of the project's working directory path. This library implements the same encoding to locate sessions by path and introduces an improved v2 encoding that eliminates ambiguity in paths containing hyphen-prefixed components (like `-default_topic`).

### Algorithm

**Encoding (v2, current):**
1. Prefix the entire path with `-`.
2. Replace each `/` separator with `-`.
3. Replace each `/-` sequence (a path component that starts with a hyphen) with `--` (double-hyphen).
4. Preserve `_` characters within components unchanged.

Example: `/commands/-default_topic` → `-commands--default_topic`.

The key improvement over v1: `--` unambiguously means "path component starting with hyphen", eliminating the ambiguity where `-a--b-c` could be interpreted as `/-a_b/c` or `/-a_b_c`.

**Encoding (v1, legacy — Claude Code's original):**
1. Prefix with `-`.
2. Replace all `/` with `-`.
3. Replace all `_` with `-` (lossy — underscores become indistinguishable from separators).

**Decoding (v1 heuristic — universally applied for backward compatibility):**
- `--` (double hyphen) → `/-` (component starting with hyphen).
- `-` after a normal component → `/` (path separator).
- `-` within a hyphen-prefixed component → `_` (underscore restoration, heuristic).

The decoder uses v1 heuristics for all paths. This means v2-encoded paths with preserved underscores (`-foo_bar-baz`) decode correctly. The only asymmetry: encoding `/foo/-bar_baz` produces `-foo--bar_baz`; decoding that back yields `/foo/-bar_baz` (correct). The heuristic successfully handles both v1 and v2 encoded paths in practice.

**Backward-compatibility guarantee:**
- All v1-encoded paths decode correctly.
- New v2 paths are unambiguous.
- Mixed storage (v1 and v2 paths coexisting) works transparently without any migration flag day.

**Known limitation:** The decoder cannot improve the decode success rate for existing v1-encoded paths that lost underscore information. Decode success rate for pre-existing storage is approximately 72.5%. A future v2-aware decoder using filesystem validation could improve this.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/path.rs` | encode_path(), decode_path() implementations |
| test | `../../tests/path_encoding_double_slash_bug.rs` | Bug reproducer for double-slash ambiguity fix |
| test | `../../tests/path_decoding_hyphen_component_bug.rs` | Hyphen-prefixed component decoding correctness |
| test | `../../tests/underscore_encoding_compatibility.rs` | v1/v2 underscore compatibility tests |
| doc | `../data_structure/001_storage_hierarchy.md` | Project type that uses path encoding for directory names |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Combined specification; path encoding section extracted here |
