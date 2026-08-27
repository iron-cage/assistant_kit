# API: Version Surface

### Scope

- **Purpose**: Pin the contract of the `version` module — detection, alias and marker resolution, installation, lock-state inspection, and preference persistence.
- **In Scope**: Every public item in `src/version.rs`.
- **Out of Scope**: The other four modules and `CoreError` (→ [001_core_surface.md](001_core_surface.md)); the `.version*` CLI commands built on this (→ `../../../claude_version/docs/cli/`).

### Compile-time tables

| Item | Contract |
|------|----------|
| `pub struct VersionAlias { name, value, description }` | A named alias resolving to a semver, or to the empty string for the dynamic `latest`. |
| `pub const VERSION_ALIASES : &[ VersionAlias ]` | All built-in aliases, in display order: `latest` (value `""`) and `stable` (a pinned semver). |
| `pub struct VersionRecord` | One compiled-in release-history record: version, date, one-line summary. |
| `pub const VERSION_HISTORY : &[ VersionRecord ]` | Compiled-in release history, newest first. Fallback source when the on-disk history cache is unavailable. |

`latest` carries `value : ""` rather than a semver — it is resolved dynamically by the
installer. Any caller that treats an alias's `value` as a version string must handle the empty
case; `resolve_version_spec` already does.

The pinned `stable` literal is mirrored across many documentation and test files. Bumping it is
governed by [invariant/002_alias_literal_consistency.md](../invariant/002_alias_literal_consistency.md).

### Custom markers

User-defined aliases persisted in `~/.claude/version-markers.json`. Unlike `VERSION_ALIASES`,
these are created and deleted at runtime and always pin a concrete semver — never a dynamic
`latest`-style value.

| Signature | Contract |
|-----------|----------|
| `pub struct CustomMarker` | A user-defined alias record. |
| `parse_markers_json( json : &str ) -> Vec< CustomMarker >` | Pure. Parses the markers array from file content. Malformed input yields an empty or partial vector — it does not error. |
| `load_custom_markers() -> Vec< CustomMarker >` | Reads and parses the markers file. A missing or unreadable file yields an empty vector, not an error. |
| `validate_marker_name( name : &str ) -> Result< (), CoreError >` | Rejects names that would collide with, or be ambiguous against, built-in aliases and semver forms. Errors as `ParseError`. |
| `save_custom_marker( name, value, description ) -> Result< (), CoreError >` | Validates, then persists. |
| `remove_custom_marker( name : &str ) -> Result< bool, CoreError >` | `Ok( false )` when no such marker existed — absence is not an error. |

The read paths swallow failure by design: an unreadable markers file degrades to "no custom
markers" rather than blocking version resolution. Only the write paths return `Result`.

### Detection and resolution

| Signature | Contract |
|-----------|----------|
| `extract_semver( raw : &str ) -> &str` | Pure, borrowing. Extracts the semver token (digits and dots) from a raw string — strips a leading `v`/`V`, and finds the token inside verbose output such as `claude 1.2.3`. |
| `get_version_from_symlink() -> Option< String >` | Reads the installed version from the `~/.local/bin/claude` symlink target. No subprocess. |
| `get_claude_version_raw() -> Option< String >` | Runs `claude --version` and returns trimmed stdout. **Spawns a subprocess** — the slow path. |
| `get_installed_version() -> Option< String >` | The version in use. Prefers symlink detection, falling back to the subprocess only when needed. |
| `resolve_version_spec( spec : &str, custom : &[ CustomMarker ] ) -> String` | Total — returns the value handed to the official installer. Never fails; validate first if the spec is untrusted. |
| `validate_version_spec( spec : &str, custom : &[ CustomMarker ] ) -> Result< (), CoreError >` | A spec is valid if it is a known alias, a supplied custom marker, or a 3-part semver. Errors as `ParseError`. |

`resolve_version_spec` is infallible and `validate_version_spec` is separate: resolution never
rejects, so calling resolve on an unvalidated spec silently produces an installer argument that
may be meaningless. Validate first.

Both take `custom` as a parameter rather than calling `load_custom_markers()` internally, which
keeps them pure and testable without a filesystem — the same design reason `parse_response`
takes `now_ms` in `claude_auth`.

### Paths

| Signature | Contract |
|-----------|----------|
| `versions_dir_path() -> String` | `~/.local/share/claude/versions`. |
| `binary_symlink_path() -> String` | `~/.local/bin/claude`. |
| `version_history_cache_path() -> String` | `~/.claude/.transient/version_history_cache.json`. |

All three read `HOME` with `unwrap_or_default()`, so with `HOME` unset they return a
*relative* path rather than failing. Prefer `paths::ClaudeVersionPaths`, whose constructor
rejects unset-or-empty `HOME` up front — see [001_core_surface.md](001_core_surface.md).

### Lock state

| Signature | Contract |
|-----------|----------|
| `pub enum VersionsDirLockMode { Locked, Unlocked, Unknown, Absent }` | `Debug + Clone + Copy + PartialEq + Eq`. `Display` renders `555`, `755`, `unknown`, `absent`. |
| `read_versions_dir_lock_mode() -> VersionsDirLockMode` | Read-only; performs no mutation. **Platform-variant** — see below. |

| Variant | Condition |
|---------|-----------|
| `Locked` | Mode `555` (read + execute) — matches a pinned install |
| `Unlocked` | Mode `755` (read + write + execute) — matches an unpinned `latest` install |
| `Unknown` | Directory exists with some other mode, **or** any I/O error other than not-found |
| `Absent` | Directory does not exist, or the platform cannot report POSIX mode bits |

`Unknown` deliberately absorbs I/O errors such as permission-denied on a parent directory, so
a genuine anomaly is flagged rather than silently reported as "nothing installed". Only a
`NotFound` maps to `Absent`.

**Platform variance:** two `cfg`-gated definitions exist. Under `cfg( unix )` the mode bits are
read via `PermissionsExt`. Under `cfg( not( unix ) )` the function unconditionally returns
`Absent`, because file mode bits are unavailable. Callers must therefore treat `Absent` as
"no reliable signal either way" and never as evidence of a lock mismatch — on non-Unix it is
the *only* value ever returned.

### Installation

These functions mutate the filesystem and, in some cases, `~/.claude/settings.json`.

| Signature | Contract |
|-----------|----------|
| `hot_swap_binary() -> Option< String >` | Moves the existing `claude` binary aside so a new install replaces it cleanly. Returns the path it moved, if any. |
| `purge_stale_versions( versions_dir : &str, keep : &str )` | Removes every cached binary except `keep`. Infallible by signature — per-entry failures are absorbed. |
| `unlock_versions_dir()` | Chmods the versions directory writable so the installer can write. |
| `lock_version( is_latest : bool, resolved : &str )` | Applies the pinned lock, or unlocks for `latest`, after a successful install. |
| `unlock_settings_for_install()` | Lifts the settings-level update locks so the official installer can run. |
| `verify_install_outcome( resolved : &str, is_latest : bool, installed : Option< &str > ) -> bool` | Decides whether an installer run actually produced the requested outcome. |
| `perform_install( resolved : &str, is_latest : bool ) -> Result< (), CoreError >` | The full sequence: settings-unlock → hot-swap → dir-unlock → install → verify → lock. |

`perform_install` is the only entry point that returns `Result`; the individual steps are
infallible by signature and absorb their own errors, with `verify_install_outcome` as the
after-the-fact check that the sequence achieved its goal. Calling the steps individually
therefore gives no error signal — use `perform_install` unless deliberately composing a
custom sequence, and check `verify_install_outcome` if you do.

`perform_install` runs the official installer from `https://claude.ai/install.sh` and requires
network access.

### Preference persistence

| Signature | Contract |
|-----------|----------|
| `read_preferred_version() -> Option< ( String, Option< String > ) >` | Reads `~/.claude/settings.json`. The tuple is `( preferredVersionSpec, preferredVersionResolved )`; the second element is `None` when the spec has not been resolved yet. |
| `store_preferred_version( spec : &str, resolved : &str, is_latest : bool ) -> Result< (), CoreError >` | Persists both keys. |

Both keys are registered in `config_catalog` with no env var and no default, so they resolve
through the `User` layer only — see [001_core_surface.md](001_core_surface.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [001_core_surface.md](001_core_surface.md) | `CoreError`, paths, catalogs, resolution engine |
| doc | [invariant/002_alias_literal_consistency.md](../invariant/002_alias_literal_consistency.md) | Rule for bumping the pinned `stable` literal |
| doc | `../../../claude_version/docs/feature/001_version_management.md` | CLI feature spec built on this surface |
| doc | `../../../claude_version/docs/pattern/002_parameter_trace.md` | The traced-mutating-function set referenced from `src/version.rs` |
| source | `../../src/version.rs` | The implementation this contract pins |
| test | `../../tests/version_test.rs` | Semver extraction, alias resolution, spec validation |
