# Algorithm: Config Resolution

### Scope

- **Purpose**: Document the 4-layer resolution algorithm that computes the effective value of a settings key for the `.config` command.
- **Responsibility**: Specify the resolution order, project config search, catalog lookup, and source annotation.
- **In Scope**: Layer priority, env var mapping, project config file location, user config file location, catalog lookup, absent-key semantics.
- **Out of Scope**: Settings I/O atomics (→ `feature/003_settings_management.md`), type inference for writes (→ `algorithm/001_settings_type_inference.md`).

### Abstract

The `.config` command resolves the effective value of a key by checking four layers in strict priority order. The first layer that supplies a non-absent value wins. This algorithm governs show-all mode (all keys, all layers) and get mode (single key, all layers).

### Algorithm

**Input:** key name K (UTF-8 string), working directory W (for project config search)

**Output:** `ResolvedValue { value: Option<String>, source: Layer }` where `Layer` ∈ {Env, Project, User, Default, Absent}

---

**Step 1 — Environment variable check:**

Look up the env var mapping for K from the catalog:

| Key | Env var |
|-----|---------|
| `model` | `CLAUDE_MODEL` |
| `CLAUDE_CODE_AUTO_CONTINUE` | (no mapping — env-only key, not in settings.json) |

If the catalog maps K to an env var E, read `std::env::var(E)`:
- If set and non-empty → return `ResolvedValue { value: Some(v), source: Env }`. Stop.
- Otherwise → proceed to Step 2.

If K has no env var mapping → proceed to Step 2.

---

**Step 2 — Project config check:**

Search for `.claude/settings.json` starting from W, walking up to filesystem root (stopping at root or a git repository boundary):
- If found, read the file and look up K.
- If K is present → return `ResolvedValue { value: Some(v), source: Project }`. Stop.
- If file parse fails → treat as absent for this key; continue.
- If not found or K absent → proceed to Step 3.

---

**Step 3 — User config check:**

Read `~/.claude/settings.json` (requires HOME set; if HOME unset → treat as absent, proceed to Step 4):
- If K is present → return `ResolvedValue { value: Some(v), source: User }`. Stop.
- If file absent or K absent → proceed to Step 4.

---

**Step 4 — Catalog default:**

Look up K in the known settings catalog:
- If K has a registered default → return `ResolvedValue { value: Some(default), source: Default }`. Stop.
- Otherwise → return `ResolvedValue { value: None, source: Absent }`.

---

### Catalog

The known settings catalog is implemented in `claude_version_core::config_catalog`. Each entry defines:

| Key | Type | Env var | Default | Notes |
|-----|------|---------|---------|-------|
| `model` | String | `CLAUDE_MODEL` | `claude-sonnet-4-6` | Active model name |
| `preferredVersionSpec` | String | — | `stable` | Version channel: stable/beta/exact |
| `preferredVersionResolved` | String | — | — (absent) | Last resolved concrete version |
| `autoUpdates` | Bool | — | `true` | Auto-update on launch |
| `theme` | String | — | `system` | UI theme: system/light/dark |
| `hasCompletedOnboarding` | Bool | — | `false` | First-run onboarding flag |
| `env.DISABLE_AUTOUPDATER` | String | — | — (absent) | Disable autoupdate via settings env block |

Non-catalog keys are accepted by `.config` with no env mapping and no default.

### Show-All Mode

For show-all mode (no `key::` param), the resolution is applied across all keys:
1. Union all keys from: env var mappings (catalog), project config, user config, catalog keys.
2. For each key in sorted order, apply Steps 1–4 to produce a `ResolvedValue`.
3. Display each key with its resolved value and source layer.

### Source Annotation

Text output annotates each key with its source layer in parentheses: `(env)`, `(project)`, `(user)`, `(default)`, `(absent)`.

JSON output includes a `source` field per key in the output object.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [feature/006_config_command.md](../feature/006_config_command.md) | `.config` command using this algorithm |
| doc | [algorithm/001_settings_type_inference.md](001_settings_type_inference.md) | Type inference for set operations |
| source | `../../../claude_version_core/src/config_resolve.rs` | Resolution engine implementation |
| source | `../../../claude_version_core/src/config_catalog.rs` | Catalog registry (SettingDef) |

### Sources

| File | Notes |
|------|-------|
| User design session 2026-06-09 | 4-layer resolution: env → project → user → catalog default |
