# claude_storage_core

Pure library for Claude Code's filesystem-based storage access (zero dependencies).

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Crate manifest and dependency configuration |
| `src/` | Core library implementation (20 modules) |
| `tests/` | Test suite for storage access logic |
| `docs/` | Behavioral requirements: features, invariants, API, algorithms, data structures |
| `examples/` | Usage examples for the storage API |
| `changelog.md` | Release history in Keep a Changelog format, semver-versioned. |
| `license` | License text for this crate |
| `verb/` | Shell scripts for each `do` protocol verb. |

## overview

This is the core library extracted from the monolithic `claude_storage` crate (2025-11-29). It provides safe, structured read/write access to Claude Code's conversation storage at `~/.claude/` with **zero runtime dependencies**.

## features

- **Zero dependencies**: No runtime dependencies for fast compilation and minimal attack surface
- **Hand-written JSON parser**: ~690 lines, supports all JSON types, Unicode escaping
- **Path encoding/decoding**: Filesystem path encoding for storage directories
- **Statistics aggregation**: Fast counting and analytics without full parsing
- **Format validation**: JSONL structure validation with detailed error messages
- **Safety guarantees**: Append-only operations, atomic writes, graceful error handling

## usage

```toml
[dependencies]
claude_storage_core = { workspace = true }
```

```rust,no_run
use claude_storage_core::{ Storage, ProjectId };

fn main() -> claude_storage_core::Result< () >
{
  // List all projects
  let storage = Storage::new()?;
  for project in storage.list_projects()?
  {
    println!( "Project: {:?}", project.id() );

    // List sessions within project
    for mut session in project.sessions()?
    {
      println!( "  Session: {}", session.id() );
      println!( "  Entries: {}", session.count_entries()? );
    }
  }
  Ok( () )
}
```

## architecture

**Storage model**: Claude Code uses filesystem-native storage at `~/.claude/`
```text
~/.claude/
├── projects/
│   ├── {uuid}/           # UUID projects (web/IDE sessions)
│   └── -{path-encoded}/  # Path projects (CLI sessions)
├── history.jsonl
└── .credentials.json
```

**Core types**:
- `Storage` - Entry point for all operations
- `Project` - Directory containing sessions (UUID or path-based)
- `Session` - Single conversation (JSONL file)
- `Entry` - Individual message (user or assistant)

**Path encoding**: `/home/user/pro` → `-home-user-pro`

## performance

- **Lazy loading**: Entries loaded on-demand, not at construction
- **Fast counting**: 100x speedup (~5ms vs 500ms for 1000 entries)
- **Selective parsing**: Statistics without loading all data
- **JSON parser**: ~80ns per operation

## testing

**277 tests** — 75 unit tests inline in `src/` (covering the private parsing and encoding
internals that integration tests cannot reach through the public API) plus 202 integration
tests in `tests/`, 14 of whose files are bug reproducers. Doc examples add ~26 more.

```bash
cargo nextest run --all-features  # 259 unit + integration
cargo test --doc --all-features   # doc examples
cargo clippy --all-targets --all-features -- -D warnings
```

Two `canonical_tests.rs` cases are `#[ cfg( unix ) ]`-gated. No test is `#[ ignore ]`d —
environment-dependent tests skip at runtime with a printed reason instead, per the policy in
[tests/readme.md](tests/readme.md).

## related crates

- **claude_storage**: CLI tool wrapping this library for command-line storage exploration
- **claude_profile**: Account and token management (uses this for session detection)

## migration

**From monolithic claude_storage**:
```diff
[dependencies]
- claude_storage = { path = "../claude_storage" }
+ claude_storage_core = { workspace = true }
```

```diff
- use claude_storage::{ Storage, ProjectId };
+ use claude_storage_core::{ Storage, ProjectId };
```

See `../claude_storage/docs/MIGRATION.md` for complete migration guide.

## documentation

- **Documentation**: `docs/` - Behavioral requirements, API contracts, algorithms, invariants
- **Format docs**: `../claude_storage/docs/` - JSONL format, storage organization, advanced features
- **Examples**: `examples/` - Usage examples and integration patterns

## license

MIT
