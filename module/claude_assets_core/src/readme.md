# src/

Layer 1 domain logic for Claude Code artifact installation.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root; module declarations and public re-exports |
| `artifact.rs` | `ArtifactKind`/`ArtifactLayout` enums and their directory mappings |
| `error.rs` | `AssetError` and `AssetPathsError` domain error types |
| `install.rs` | Symlink-only `install()` and `uninstall()` operations |
| `paths.rs` | `AssetPaths`: resolve `$PRO_CLAUDE` source and cwd target roots |
| `registry.rs` | Enumerate available and installed artifacts with install status |
