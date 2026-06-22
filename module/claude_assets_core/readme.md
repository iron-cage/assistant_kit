# claude_assets_core

| File | Responsibility |
|------|----------------|
| src/lib.rs | Crate root; module re-exports |
| src/artifact.rs | ArtifactKind and ArtifactLayout enums with inherent methods |
| src/error.rs | AssetError enum for domain-level operation failures |
| src/paths.rs | AssetPaths: resolves source root from $PRO_CLAUDE env var |
| src/registry.rs | InstallStatus; list_available, list_installed, list_all |
| src/install.rs | install() and uninstall() with symlink-only semantics |
| `docs/` | Behavioral requirements: feature and invariant doc instances |
| `tests/` | Integration test suite directory |
| tests/install.rs | Real-fs integration tests via tempfile |
| `verb/` | Shell scripts for each `do` protocol verb. |
