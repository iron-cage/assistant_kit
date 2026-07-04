# Invariant: Publish Sandbox Safety

### Scope

- **Purpose**: Guarantee that `build.rs` compiles and runs successfully when Cargo verifies the package during `cargo publish`, without assuming sibling crate directories are accessible via `CARGO_MANIFEST_DIR`-relative paths.
- **Responsibility**: Document the constraint that `build.rs` must handle absent sibling YAML files gracefully so `cargo package` succeeds without panic.
- **In Scope**: All `std::fs::read_to_string` and `include_str!` calls in `build.rs`; any `concat!(env!("CARGO_MANIFEST_DIR"), ...)` path constructions that traverse outside the crate root.
- **Out of Scope**: Runtime path resolution in `src/lib.rs` or `src/main.rs`; workspace-level `Cargo.toml` configuration; Layer 2 crate `build.rs` files.

### Invariant Statement

`build.rs` MUST exit 0 when compiled and run by `cargo package --verify`, without panic or file-not-found error:

1. **Cross-package reads are existence-guarded** — any `std::fs::read_to_string` on a path constructed via `concat!(env!("CARGO_MANIFEST_DIR"), "/../<sibling>/...")` must be preceded by a `Path::exists()` check. When files are absent (publish sandbox), `build.rs` must not panic; it must fall back to a sandbox-safe alternative (e.g., empty static registry).

2. **Sandbox-safe fallback required** — when any sibling YAML file is absent, `build.rs` must complete with exit 0 and produce a valid (possibly empty) build artifact. `cargo package` must not emit `panicked at build.rs` in stderr.

**Current status (2026-07-02):** FIX APPLIED — `build.rs` lines 117-127 add an existence guard; absent sibling YAML files produce an empty registry instead of panicking (BUG-003 fix, Option B). Full `cargo package` verification blocked: `claude_journal v0.1.0` and `claude_journal_viewer v0.1.0` are not yet published to crates.io; `cargo package --verify` fails with "no matching package named `claude_journal_viewer` found". Publication order: (1) `cargo publish -p claude_journal`, (2) `cargo publish -p claude_journal_viewer` (after index propagates), (3) re-run `cargo package --allow-dirty -p assistant` to verify. `claude_journal` has only external deps (`serde`, `serde_json`, `chrono`) and is ready to publish immediately.

### Enforcement Mechanism

`cargo package` (default behavior) extracts the package to a temp sandbox and builds it in isolation. If `build.rs` reads a file outside the package, the build fails with a panic. This is the canonical enforcement command — it fails loudly when the invariant is violated. Use `--allow-dirty` when the workspace has uncommitted changes.

Verify locally:

```bash
cargo package --allow-dirty
```

Exit 0 = invariant holds. Any panic or non-zero exit = invariant violated.

### Violation Consequences

When `build.rs` panics in the sandbox, `cargo publish` aborts with `error: failed to verify package tarball`. The crate cannot be published to any registry. Every version bump requiring a publish will fail until the invariant is restored, blocking all release operations.

### Features

| File | Relationship |
|------|--------------|
| [../feature/001_super_app_aggregation.md](../feature/001_super_app_aggregation.md) | Static YAML aggregation that `build.rs` implements |

### Sources

| File | Relationship |
|------|--------------|
| [../../build.rs](../../build.rs) | YAML path constants at lines 22-24; existence guard + empty-registry fallback at lines 117-127 — fix applied |
| [../../Cargo.toml](../../Cargo.toml) | `exclude` list; absence of `include` list — determines which files enter the package tarball |

### Tests

| File | Relationship |
|------|--------------|
| [../../tests/docs/invariant/002_publish_sandbox_safety.md](../../tests/docs/invariant/002_publish_sandbox_safety.md) | Test spec: PS-1 publish-safe build verification |
