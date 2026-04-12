# Invariant: Symlink Only

### Scope

- **Purpose**: Guarantee that install operations create symbolic links only — never copy files — so that `$PRO_CLAUDE` remains the single source of truth.
- **Governs**: The `install()` function in `src/install.rs` and any future installation paths added to claude_assets_core.
- **In Scope**: All calls to the install operation; any code that writes into `.claude/<kind>/` directories.
- **Out of Scope**: Read operations (`list_all`, `list_available`, `list_installed`), the uninstall operation's removal mechanism.

### Rule

`install()` MUST call `std::os::unix::fs::symlink()` to create the target entry. It MUST NOT call `std::fs::copy()`, `std::fs::hard_link()`, or any other mechanism that duplicates file content.

**Rationale:** Symlinks preserve `$PRO_CLAUDE` as the single source of truth. Any edit to a rule, command, or skill in `$PRO_CLAUDE` propagates instantly to every project that has installed it. If files were copied, each project would hold a stale snapshot requiring manual re-installation after every change.

**Data-loss guard for install:** Before creating a symlink at the target path, `install()` checks whether an existing entry is a symlink using `symlink_metadata()`. If the entry is a regular file (not a symlink), the operation returns `AssetError::NotASymlink` rather than overwriting it. This prevents accidental destruction of hand-crafted files that happen to share a name with an artifact.

**Data-loss guard for uninstall:** `uninstall()` uses `symlink_metadata()` (not `metadata()`) to classify the target entry before removing it. `symlink_metadata()` does not follow symlinks, so it correctly identifies dangling symlinks (pointing to a deleted source) as symlinks rather than as missing files. If the target is a regular file, `uninstall()` returns `AssetError::NotASymlink` and leaves it untouched.

**Why `symlink_metadata()` and not `metadata()`:** `metadata()` follows symlinks. A dangling symlink (source deleted) returns `Err(NotFound)` from `metadata()`, which would be misinterpreted as "nothing to uninstall." `symlink_metadata()` sees the symlink entry itself and returns its metadata correctly regardless of whether the source exists.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/install.rs` | install() and uninstall() — the only permitted install paths |
| feature | [feature/001_artifact_installer.md](../feature/001_artifact_installer.md) | Full installer design including idempotency semantics |
| test | `tests/install.rs` | Real-fs integration tests verifying symlink creation |
