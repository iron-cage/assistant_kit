# API: Core Surface

### Scope

- **Purpose**: Pin the contract of the crate-level error type and the four supporting modules — `paths`, `config_catalog`, `config_resolve`, `params_catalog`.
- **In Scope**: `CoreError`; `paths::ClaudeVersionPaths`; the settings catalog and 4-layer resolution engine; the parameter catalog.
- **Out of Scope**: The `version` module (→ [002_version_surface.md](002_version_surface.md)); CLI behavior (→ `../../../claude_version/docs/`).

### `CoreError`

The single error type for every fallible operation in this crate. Derives `Debug`; implements
`Display` and `core::error::Error`, with `source()` returning the inner `io::Error` for
`IoError` and `None` otherwise. `From< std::io::Error >` is implemented, so `?` works directly
on I/O.

| Variant | `Display` prefix | Meaning |
|---------|------------------|---------|
| `CoreError::IoError( std::io::Error )` | `io: ` | An I/O operation failed |
| `CoreError::ParseError( String )` | `parse: ` | A parse or validation failure with a human-readable message |
| `CoreError::ProcessError( String )` | `process: ` | A subprocess or process-level failure |

Layer 2 adapts at the call site — `.map_err( |e| ErrorData::new( code, e.to_string() ) )`.
This crate never names `ErrorData`; see
[invariant/001_layer_one_boundary.md](../invariant/001_layer_one_boundary.md).

### `paths::ClaudeVersionPaths`

A thin composition over `claude_core::ClaudePaths`, holding it privately and exposing the
clv-specific locations. `Debug + Clone`.

| Signature | Contract |
|-----------|----------|
| `ClaudeVersionPaths::new() -> Option< Self >` | Reads `HOME`. Returns `None` if `HOME` is unset **or empty** — the empty case is checked explicitly, because the path helpers below would otherwise silently compose absolute-looking paths rooted at `/`. |
| `.settings_file() -> PathBuf` | `~/.claude/settings.json`; delegates to `ClaudePaths::settings_file()`. |
| `.versions_dir() -> PathBuf` | `~/.local/share/claude/versions`; delegates to `version::versions_dir_path()`. |
| `.binary_symlink() -> PathBuf` | `~/.local/bin/claude`; delegates to `version::binary_symlink_path()`. |
| `.version_history_cache_file() -> PathBuf` | `~/.claude/.transient/version_history_cache.json`. |
| `.project_settings_file( cwd : &Path ) -> Option< PathBuf >` | Nearest ancestor `.claude/settings.json` walking up from `cwd`. `None` if none is found before the git-repository boundary or the filesystem root. |

The free functions this type delegates to (`version::versions_dir_path()` and friends) read
`HOME` themselves with `unwrap_or_default()`, so calling them directly with `HOME` unset
yields a relative path. Prefer `ClaudeVersionPaths`, whose constructor rejects that state up
front.

### `config_catalog`

| Signature | Contract |
|-----------|----------|
| `pub struct SettingDef { key, env_var, default }` | All fields `&'static str` / `Option< &'static str >`. `Debug`. |
| `catalog() -> &'static [ SettingDef ]` | The authoritative registry of known settings keys. `#[must_use]`. |

Ten entries, of which exactly two carry an env-var mapping — `model` → `CLAUDE_MODEL` and
`env.DISABLE_UPDATES` → `DISABLE_UPDATES`. A key absent from this catalog can still be read
from a settings file, but it has no env-var layer and no default; see `Layer::Absent` below.

### `config_resolve`

| Signature | Contract |
|-----------|----------|
| `pub enum Layer { Env, Project, User, Default, Absent }` | `Debug + Clone + Copy + PartialEq + Eq`. `Display` renders lowercase: `env`, `project`, `user`, `default`, `absent`. |
| `pub struct ResolvedValue { value : Option< String >, source : Layer }` | `value` is `None` exactly when `source` is `Absent`. |
| `resolve( key : &str, home_dir : &Path, cwd : &Path, catalog : &[ SettingDef ] ) -> ResolvedValue` | Resolves one key through the four layers in priority order. `#[must_use]`. |
| `resolve_all( home_dir : &Path, cwd : &Path, catalog : &[ SettingDef ] ) -> Vec< ( String, ResolvedValue ) >` | Resolves the **union** of catalog keys, project-config keys, and user-config keys, in sorted key order — not just the catalog. A key present only in a settings file therefore appears in the output, resolved to `User` or `Project`. `#[must_use]`. |

**Priority order is fixed:** `Env` → `Project` → `User` → `Default`, falling through to
`Absent`. The env-var layer consults the catalog mapping only — an arbitrary
`CLAUDE_`-prefixed variable does not participate. The project layer walks upward from `cwd`
and stops at the git-repository boundary or the filesystem root, whichever comes first.

**An empty env var does not win the resolution.** `resolve` requires the variable to be both
set and non-empty before returning `Layer::Env`; `CLAUDE_MODEL=""` falls through to the
project layer rather than resolving to `Some( "" )`. This is deliberate — an exported-but-blank
variable is far more often an unset-shell artifact than an intentional override — but it means
`Env` can never be the source of an empty string. The lower layers apply no such filter, so a
settings file *can* yield `Some( "" )`, and that outcome remains distinct from `Absent`.

### `params_catalog`

| Signature | Contract |
|-----------|----------|
| `pub struct ParamDef { name, cli_flag, env_var, config_key, default }` | One entry per logical Claude Code parameter. `Debug`. |
| `ParamDef::is_cli_only( &self ) -> bool` | `true` when the parameter has a CLI flag but neither env var nor config key — it cannot be persisted or observed outside a running process. |
| `ParamDef::has_config( &self ) -> bool` | `true` when `config_key` is `Some`. |
| `ParamDef::has_env( &self ) -> bool` | `true` when `env_var` is `Some`. |
| `lookup( name : &str ) -> Option< &'static ParamDef >` | Exact match on `name`, the canonical short key. |
| `params_catalog() -> &'static [ ParamDef ]` | The full catalog. `#[must_use]`. |

Structural guarantees — no duplicate `name`, every entry observable through at least one form,
entries sorted alphabetically — are asserted by `tests/params_catalog_test.rs` rather than
enforced by the type. Adding an entry out of order fails that test, not the compiler.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [002_version_surface.md](002_version_surface.md) | The `version` module's contract |
| doc | [invariant/001_layer_one_boundary.md](../invariant/001_layer_one_boundary.md) | Why `CoreError` and not `ErrorData` |
| doc | `../../../claude_version/docs/algorithm/002_config_resolution.md` | The resolution algorithm these types implement |
| source | `../../src/lib.rs`, `../../src/paths.rs`, `../../src/config_catalog.rs`, `../../src/config_resolve.rs`, `../../src/params_catalog.rs` | The implementation this contract pins |
| test | `../../tests/config_resolve_test.rs` | AT-01–AT-04 and further layer-precedence cases |
| test | `../../tests/params_catalog_test.rs` | Catalog structural integrity and `lookup` |
| test | `../../tests/settings_io_test.rs` | Type inference, escaping, and settings round-trips |
