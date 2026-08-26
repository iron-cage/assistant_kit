# Algorithm: Config Resolution

**Status**: Implemented | **Since**: 1.4.1

### Scope

- **Purpose**: Document the 4-layer resolution algorithm that computes the effective value of a settings key, implemented by `resolve()`/`resolve_all()` in `config_resolve.rs` over the catalog in `config_catalog.rs`.
- **Responsibility**: Specify the resolution order, project config search, catalog lookup, and source annotation carried on `ResolvedValue`.
- **In Scope**: Layer priority, env var mapping, project config file location, user config file location, catalog lookup, absent-key semantics, the `resolve_all()` multi-key union.
- **Out of Scope**: How the `.config` CLI command renders a `ResolvedValue` as text/JSON output (→ `claude_version/docs/algorithm/002_config_resolution.md` § Source Annotation — the consuming Layer 2 crate's own concern), settings I/O atomics (→ `claude_core::settings_io`), type inference for writes.

## Description

`resolve(key, home_dir, cwd, catalog)` computes one settings key's effective value by checking four layers in strict priority order, stopping at the first layer that supplies a value. `resolve_all(home_dir, cwd, catalog)` unions the key set from the catalog, the project config file, and the user config file, then applies `resolve()` to each in sorted order — this is what backs "show all keys" callers without them re-implementing the union themselves.

## Interface

```rust
pub enum Layer { Env, Project, User, Default, Absent }  // impls Display

pub struct ResolvedValue
{
  pub value  : Option< String >,
  pub source : Layer,
}

pub fn resolve( key : &str, home_dir : Option< &str >, cwd : &Path, catalog : &[ SettingDef ] ) -> ResolvedValue;
pub fn resolve_all( home_dir : Option< &str >, cwd : &Path, catalog : &[ SettingDef ] ) -> Vec< ( String, ResolvedValue ) >;

// config_catalog.rs
pub struct SettingDef { pub key : &'static str, pub env_var : Option< &'static str >, pub default : Option< &'static str > }
pub fn catalog() -> &'static [ SettingDef ];  // 10 entries
```

## Algorithm

**Input:** key name K (UTF-8 string), working directory W (for project config search)

**Output:** `ResolvedValue { value: Option<String>, source: Layer }`

---

**Step 1 — Environment variable check:**

Look up K's env var mapping from the catalog. Of the 10 catalog entries, only `model` maps to an env var (`CLAUDE_MODEL`); the other 9 have `env_var: None`.

If the catalog maps K to an env var E, read it:
- Set and non-empty → return `ResolvedValue { value: Some(v), source: Env }`. Stop.
- Otherwise → proceed to Step 2.

If K has no env var mapping → proceed to Step 2 directly.

---

**Step 2 — Project config check:**

Search for `.claude/settings.json` starting from W, walking up to the filesystem root (stopping at root or a git repository boundary):
- Found and K present → return `ResolvedValue { value: Some(v), source: Project }`. Stop.
- File parse fails → treat as absent for this key; continue.
- Not found or K absent → proceed to Step 3.

---

**Step 3 — User config check:**

Read `~/.claude/settings.json` (`home_dir` unset → treat as absent, proceed to Step 4):
- If K has an `env.` prefix, look up the remainder inside the nested `"env"` sub-object rather than flat-matching the whole dotted key — `env.DISABLE_AUTOUPDATER` and `env.DISABLE_UPDATES` are stored as nested fields of the `env` object, not flat top-level keys.
- K present (flat, or inside the nested `env` object for `env.`-prefixed keys) → return `ResolvedValue { value: Some(v), source: User }`. Stop.
- File absent or K absent → proceed to Step 4.

---

**Step 4 — Catalog default:**

Look up K in `catalog()`:
- Registered default → return `ResolvedValue { value: Some(default), source: Default }`. Stop.
- Otherwise → return `ResolvedValue { value: None, source: Absent }`.

---

## Catalog

`catalog()` returns a static 10-entry slice — the source of truth for which keys `resolve_all()` reports even when absent from every config file:

| Key | Env var | Default |
|-----|---------|---------|
| `model` | `CLAUDE_MODEL` | `claude-sonnet-5` |
| `preferredVersionSpec` | — | — (absent) |
| `preferredVersionResolved` | — | — (absent) |
| `autoUpdates` | — | `true` |
| `theme` | — | `dark` |
| `hasCompletedOnboarding` | — | `false` |
| `env.DISABLE_AUTOUPDATER` | — | — (absent) |
| `autoUpdatesChannel` | — | `latest` |
| `minimumVersion` | — | — (absent) |
| `env.DISABLE_UPDATES` | — | — (absent) |

Non-catalog keys are accepted by `resolve()` with no env mapping and no default — they resolve via Steps 2-3 only, or `Absent` if unset everywhere.

## resolve_all() — Multi-Key Union

1. Union all keys from: catalog entries, project config file, user config file.
2. For each key in sorted order, apply Steps 1-4 above.
3. Return the `(key, ResolvedValue)` pairs — callers (e.g. a CLI's "show all" mode) format/display from here.

## Behavioral Contract

- Layer priority is fixed and total: Env > Project > User > Default > Absent — no caller-configurable override
- `env.`-prefixed keys are the only ones read from a nested JSON sub-object in Step 3; every other key is a flat top-level lookup
- A malformed project or user config file is treated as absent for the affected key, not as an error — resolution never fails, it degrades to the next layer
- `resolve_all()` reports a key even when it is absent from every layer, as long as it appears in the catalog or in either config file — a key with no catalog entry and no config file presence is simply not in the union at all

## Sources

- `../../src/config_resolve.rs` — `resolve()`, `resolve_all()`, `Layer`, `ResolvedValue`
- `../../src/config_catalog.rs` — `SettingDef`, `catalog()`

## See Also

| File | Relationship |
|------|-------------|
| [claude_version/docs/algorithm/002_config_resolution.md](../../../claude_version/docs/algorithm/002_config_resolution.md) | CLI-layer treatment: `.config` command output formatting (text `(source)` annotation, JSON `source` field), current catalog-coverage gap vs. the ~21 known settings.json keys |
| [claude_version/docs/feature/006_config_command.md](../../../claude_version/docs/feature/006_config_command.md) | `.config` command using this algorithm |
