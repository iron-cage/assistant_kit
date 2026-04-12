# Invariant: Source Root Resolution

### Scope

- **Purpose**: Guarantee that every install and list operation resolves a valid source root before accessing the filesystem, providing a clear error when neither `$PRO_CLAUDE` nor `$PRO` is set.
- **Governs**: `AssetPaths::from_env()` called at the start of `list_routine`, `install_routine`, and `uninstall_routine` in `src/commands.rs`.
- **In Scope**: All three mutating and surveying commands (`.list`, `.install`, `.uninstall`); any future command that accesses `$PRO_CLAUDE`.
- **Out of Scope**: `.kinds` — this command degrades gracefully when `$PRO_CLAUDE` is unset, printing the literal string `$PRO_CLAUDE` as the source root.

### Rule

Every command that reads from or installs into `$PRO_CLAUDE` MUST call `AssetPaths::from_env()` and propagate `AssetPathsError::EnvVarNotSet` as an `InternalError` before performing any filesystem access.

**Resolution order:**
1. `$PRO_CLAUDE` — used directly if set in the environment
2. `$PRO/genai/claude/` — constructed if `$PRO` is set and `$PRO_CLAUDE` is not
3. Error — `AssetPathsError::EnvVarNotSet` when neither variable is present

**Rationale:** The installer has no fallback source root to guess from. Without `$PRO_CLAUDE` (or the `$PRO` fallback), any install operation would write symlinks pointing to nonexistent paths. The error message produced by `AssetPathsError::EnvVarNotSet` includes the exact export command needed to fix the problem, making it actionable without consulting documentation.

**`.kinds` exception:** `.kinds` is a read-only introspection command that is useful even before `$PRO_CLAUDE` is configured. It falls back to printing `$PRO_CLAUDE` as a placeholder string, which still communicates the expected directory layout to the operator.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/commands.rs` | list_routine, install_routine, uninstall_routine — all call from_env() |
| source | `claude_assets_core/src/paths.rs` | AssetPaths::from_env() resolution logic and error type |
| feature | [feature/001_asset_cli.md](../feature/001_asset_cli.md) | CLI command design including .kinds graceful degradation |
