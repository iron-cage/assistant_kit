# pathToClaudeCodeExecutable

Overrides which `claude` binary the SDK spawns.

### Forms

| | Value |
|-|-------|
| TS Field | `pathToClaudeCodeExecutable?: string` |
| Python Field | not confirmed in pages fetched for this crate; likely `path_to_claude_code_executable` by snake_case analogy, unconfirmed |
| CLI Equivalent | — (N/A; the CLI binary doesn't spawn itself) |

### Type

string (filesystem path)

### Default

Auto-resolved — the SDK locates a `claude` binary itself (bundled optional dependency for the TypeScript package, or `PATH` lookup)

### Since

SDK GA

### Description

Exists specifically because the TypeScript package "bundles a native Claude Code binary for your platform as an optional dependency" (per the official overview page) — if a package manager skips optional dependencies, or a caller wants to pin a specific already-installed `claude` version rather than whatever got bundled, this field points the SDK at that binary explicitly. This is the field most directly relevant to any Rust bridge strategy that wants explicit control over exactly which `claude` binary gets driven, rather than relying on auto-resolution (see [`../pattern/002_rust_bridge_strategies.md`](../pattern/002_rust_bridge_strategies.md)) — together with `spawnClaudeCodeProcess` (a full spawn-override callback, not curated as its own instance here) it is the clearest documented evidence that `query()`'s default behavior is "locate and spawn a `claude` binary" (see [S1](../behavior/001_s1_sdk_wraps_same_binary.md)).

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master curated parameter table |
| behavior | [../behavior/001_s1_sdk_wraps_same_binary.md](../behavior/001_s1_sdk_wraps_same_binary.md) | The subprocess-spawn fact this field's existence evidences |
| pattern | [../pattern/002_rust_bridge_strategies.md](../pattern/002_rust_bridge_strategies.md) | Rust bridge relevance |
