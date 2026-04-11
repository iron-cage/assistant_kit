# claude_assets_core

| File | Responsibility |
|------|----------------|
| src/lib.rs | Crate root; module re-exports |
| src/artifact.rs | ArtifactKind and ArtifactLayout enums with inherent methods |
| src/paths.rs | AssetPaths: resolves source root from $PRO_CLAUDE env var |
| src/registry.rs | InstallStatus; list_available, list_installed, list_all |
| src/install.rs | install() and uninstall() with symlink-only semantics |
| tests/install.rs | Real-fs integration tests via tempfile |
