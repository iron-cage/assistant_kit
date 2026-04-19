# Feature: Artifact Installer

### Scope

- **Purpose**: Document the symlink-based install/uninstall design of claude_assets_core for all six Claude Code artifact kinds.
- **Responsibility**: Describe ArtifactKind taxonomy, ArtifactLayout, AssetPaths resolution, and install/uninstall semantics including idempotency and data-loss guards.
- **In Scope**: ArtifactKind variants and their subdirectory mappings, File vs Directory layout, AssetPaths env var resolution, install() and uninstall() behavior, InstallReport outcomes.
- **Out of Scope**: CLI command design (→ `claude_assets/docs/feature/001_asset_cli.md`), symlink-only enforcement rule (→ `invariant/001_symlink_only.md`).

### Design

**Artifact taxonomy:** Six artifact kinds are supported, each with a canonical lowercase name, a source subdirectory in `$PRO_CLAUDE/`, and a target subdirectory in `.claude/`:

| Kind | Name | Layout | Extension | Source | Target |
|------|------|--------|-----------|--------|--------|
| Rule | `rule` | File | `.md` | `rules/` | `.claude/rules/` |
| Command | `command` | File | `.md` | `commands/` | `.claude/commands/` |
| Agent | `agent` | File | `.md` | `agents/` | `.claude/agents/` |
| Skill | `skill` | Directory | — | `skills/` | `.claude/skills/` |
| Plugin | `plugin` | Directory | — | `plugins/` | `.claude/plugins/` |
| Hook | `hook` | File | `.yaml` | `hooks/` | `.claude/hooks/` |

`ArtifactLayout::File` artifacts are single files with a known extension. `ArtifactLayout::Directory` artifacts are entire subdirectory trees (skills and plugins).

**Source root resolution:** `AssetPaths::from_env()` resolves the source root in priority order:
1. `$PRO_CLAUDE` — used directly if set
2. `$PRO/genai/claude/` — constructed if `$PRO` is set and `$PRO_CLAUDE` is not
3. `AssetPathsError::EnvVarNotSet` — returned when neither env var is present

The target root is always the current working directory. `AssetPaths::new(source, target)` is available for test-controlled construction.

**Install semantics:** `install(paths, kind, name)` creates a symlink from `$PRO_CLAUDE/<kind>/<name>[.ext]` to `.claude/<kind>/<name>[.ext]`. Behavior:
- Creates the `.claude/<kind>/` target subdirectory if absent (`create_dir_all`).
- If a symlink already exists at the target path, removes it and re-links — idempotent update, reports `InstallOutcome::Reinstalled`.
- If no symlink exists, creates a fresh link and reports `InstallOutcome::Installed`.
- Refuses to overwrite a regular file: returns `AssetError::NotASymlink` (data-loss guard).
- Returns `AssetError::SourceNotFound` if the source artifact does not exist.

**Uninstall semantics:** `uninstall(paths, kind, name)` removes the symlink at `.claude/<kind>/<name>[.ext]`. Behavior:
- Uses `symlink_metadata()` (not `metadata()`) to detect dangling symlinks correctly.
- Returns `UninstallOutcome::NotInstalled` (not an error) when no entry exists at the target path.
- Refuses to remove a regular file: returns `AssetError::NotASymlink` (data-loss guard).

**InstallReport:** Both functions return `InstallReport<A> { kind, name, action }`. `install()` returns `InstallReport<InstallOutcome>` where `action` is `Installed` or `Reinstalled`. `uninstall()` returns `InstallReport<UninstallOutcome>` where `action` is `Uninstalled` or `NotInstalled`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/artifact.rs` | ArtifactKind and ArtifactLayout enums with subdirectory mappings |
| source | `src/error.rs` | AssetError and AssetPathsError domain error types |
| source | `src/paths.rs` | AssetPaths env var resolution and directory computation |
| source | `src/install.rs` | install() and uninstall() implementation |
| source | `src/registry.rs` | list_available, list_installed, list_all for survey queries |
| doc | [invariant/001_symlink_only.md](../invariant/001_symlink_only.md) | Rule: install() must use symlink only, never copy |
| doc | [claude_assets/docs/feature/001_asset_cli.md](../../claude_assets/docs/feature/001_asset_cli.md) | CLI layer that calls install/uninstall |
