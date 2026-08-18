# Feature: Unified Model & Effort Command

> **Supersedes Feature 069** ([069_model_select_command.md](069_model_select_command.md)): `.model.select` is retired and merged into `.model` via the new `scope::` parameter. See that file for the historical, standalone `.model.select` design it replaces.

### Scope

- **Purpose**: Provide a single `clp .model` command to read and write BOTH model-related persisted config stores this workspace maintains — the Claude Code interactive session store (`~/.claude/settings.json`) and the clr subprocess-execution store (`~/.clr/config.toml`) — including, for the first time, direct CLI control of each store's **effort** key, not just its model key.
- **Responsibility**: Documents the unified `.model` command, the `scope::` parameter that selects which backing store a call targets, the `model::`/`effort_level::` set actions and `reset_model::`/`reset_effort_level::` reset actions (independently combinable, each scope-relative), the mandatory absolute-path disclosure in every mode, the two distinct model/effort validation vocabularies (session vs. subprocess — they are NOT the same enum), and the no-duplication contract with Features 034 and 062.
- **In Scope**: `.model` command; `scope::` parameter (`session` default | `subprocess`); get mode (no set/reset params) printing `scope`, resolved absolute file path, `model`, and `effort_level` together; set actions `model::VALUE` and `effort_level::VALUE`, independently applicable and combinable in one call; reset actions `reset_model::1` and `reset_effort_level::1`, independently applicable and combinable with each other or with the opposite concept's set action (e.g. `model::opus reset_effort_level::1` in one call); `format::json` get output; a new `remove_session_effort()` helper in `claude_profile_core` (counterpart to the existing `set_session_effort()`) required so `scope::session reset_effort_level::1` has a write path to call; reuse of `claude_core::toml_io`'s existing generic `get_tiered()`/`set_user_tier()`/`remove_user_tier()` against the `effort` TOML key (no new low-level I/O code needed for the subprocess-effort store — only new call sites).
- **Out of Scope**: Subprocess touch/refresh **ephemeral** model/effort override (→ 026_subprocess_model_effort.md `imodel::`/`effort::` params — deliberately separate: per-invocation only, never persisted, and — critically — already using the literal parameter name `effort::` for an unrelated, differently-valued concept; this feature intentionally does NOT reuse that name, see Design's Naming Collision Avoidance note); automatic Sonnet→Opus threshold override (→ 027_account_use_post_switch_touch.md); `set_model::` side-parameter on `.account.use`/`.usage` (→ 034_explicit_session_model_override.md — left as-is, a distinct rotation-flow convenience parameter, not merged into `.model`); global inference provider selection (→ 072_inference_provider_selection.md `.provider.select` — shares the same `~/.clr/config.toml` user-tier file and `toml_io` primitives as this feature's `scope::subprocess`, but governs a materially different axis (which backend/API endpoint, not which model/effort) and was not named in the merge instruction that produced this feature; left as its own command); `fallback_model` key in `~/.clr/config.toml` (has no clp reader or writer today, on either the old or the new design — it is an automatic error-path degradation value consulted only by `claude_runner` itself, not a user-facing preference in the same sense as `model`/`effort`; flagged here as a candidate for a future, separately-decided parameter rather than silently folded in); 4-layer config resolution with env-var and project overrides (→ `clv .config key::model` in `claude_version`).

### Design

**Why one command, `scope::`-routed, instead of two:** the previous design (`.model` for session, `.model.select` for subprocess) was evaluated once already against this project's own [cli/command_group/readme.md](../cli/command_group/readme.md) Representation Absorption Test and explicitly found NOT to qualify for merging — different dispatch functions, non-overlapping parameter sets, and "different config surfaces entirely." That evaluation was correct on its own terms (the two commands shared no handler and no parameters beyond `format::`) but is explicitly superseded here by direct design instruction: the two stores are different files, but from the operator's point of view they answer the same question — "what model/effort is currently in force, and where" — and living under two differently-named commands with two different parameter vocabularies (`set::`/`id::`) was the source of real operational confusion (a `.model.select id::X` write was mistaken for also updating `.model`'s answer). The merge documented here is a **single CLI verb with an explicit `scope::` router**, not a claim that the two backing stores became one. `command_group/readme.md`'s own Groups table is updated alongside this feature (see that file) to reflect `.model.select` moving from "Evaluated, Not Qualifying" (as a merge candidate) to retired-and-absorbed.

**`scope::` parameter** — selects the backing store every other parameter on this call applies to:

| `scope::` value | Backing store (absolute path, `$HOME`-relative) | Written/read via |
|---|---|---|
| `session` (default) | `$HOME/.claude/settings.json` | `claude_profile_core::account::{get_session_model,set_session_model,get_session_effort,set_session_effort,remove_session_effort}` |
| `subprocess` | `$HOME/.clr/config.toml` (user tier only — project-tier `.clr.toml` is read by `clr` itself, never by this command) | `claude_core::toml_io::{get_tiered,set_user_tier,remove_user_tier}` against keys `model` and `effort` |

**Get mode** (no `model::`, `effort_level::`, `reset_model::1`, or `reset_effort_level::1` present):

Reads both `model` and `effort_level` for the selected scope and prints them together with the resolved absolute path (Rule: every mode names the exact file it read from or wrote to — see Absolute Path Disclosure below).

Text output, `scope::session` (default):
```
$ clp .model
scope: session (/home/user1/.claude/settings.json)
model: sonnet
effort_level: high
```

Text output, `scope::subprocess`:
```
$ clp .model scope::subprocess
scope: subprocess (/home/user1/.clr/config.toml)
model: claude-sonnet-5
effort_level: (unset)
```

JSON output (`format::json`):
```json
{"scope":"session","path":"/home/user1/.claude/settings.json","model":"sonnet","effort_level":"high"}
```
Absent values serialize as `null`, not the string `"(unset)"`.

**Set/reset actions** — any of the four action parameters present activates write mode; multiple actions may be combined in a single call as long as no parameter is paired with its own reset (see Mutual Exclusion):

| Parameter | Effect | Session (`scope::session`) valid values | Subprocess (`scope::subprocess`) valid values |
|---|---|---|---|
| `model::VALUE` | Write the model key for the selected scope | `opus`, `sonnet`, `haiku`, `default` (shorthand, mapped via `map_model_shorthand()`) | Any non-empty full model ID string (no allow-list — run `.models` to discover valid IDs) |
| `effort_level::VALUE` | Write the effort key for the selected scope | `low`, `normal`, `high`, `max` | `low`, `medium`, `high`, `max` — **note `medium`, not `normal`** (see Naming/Vocabulary note below) |
| `reset_model::1` | Remove the model key for the selected scope | — | — |
| `reset_effort_level::1` | Remove the effort key for the selected scope | — | — |

Text output, write mode:
```
$ clp .model model::opus
model: opus  →  /home/user1/.claude/settings.json (session)

$ clp .model scope::subprocess model::claude-opus-4-8 effort_level::max
model: claude-opus-4-8  →  /home/user1/.clr/config.toml (subprocess)
effort_level: max  →  /home/user1/.clr/config.toml (subprocess)

$ clp .model reset_effort_level::1
effort_level: (reset)  →  /home/user1/.claude/settings.json (session)
```

**Mutual exclusion:** `model::` + `reset_model::1` together → exit 1, stderr `model:: and reset_model::1 are mutually exclusive`. `effort_level::` + `reset_effort_level::1` together → exit 1, stderr `effort_level:: and reset_effort_level::1 are mutually exclusive`. The two *concepts* (model vs. effort) are never mutually exclusive with each other — `model::opus reset_effort_level::1` is valid and applies both actions in the same call, satisfying the "flexible set of parameters" requirement this feature was built to meet.

**Absolute Path Disclosure (mandatory, every mode):** every text and JSON output — get, set, or reset — names the fully resolved absolute path of the file it read from or wrote to. Both backing paths are already computed as absolute (`ClaudePaths::settings_file()` in `claude_core/src/paths.rs`, joining `$HOME/.claude/settings.json`; `resolve_subprocess_config_path()` in `src/commands/model.rs`, joining `$HOME/.clr/config.toml`) — this feature adds no new path-resolution logic, only surfaces the already-resolved `PathBuf` in command output via `.display()`, which was previously computed but never printed.

**Naming/Vocabulary note (why `effort_level::`, not `effort::`):** `.usage`, `.account.use`, and `.accounts` already register a parameter literally named `effort::` (param 36, `docs/cli/param/036_effort.md`) governing the **ephemeral, never-persisted** touch/refresh subprocess effort override — vocabulary `auto`/`low`/`normal`/`high`/`max`, resolved fresh on every invocation, with no relationship to either store this feature manages. Reusing the string `effort::` here — even though there is no CLI-parsing collision, since parameter names are scoped per command — would recreate exactly the kind of same-word-different-meaning confusion that motivated this whole redesign. This feature therefore uses `effort_level::` throughout (parameter name and get-mode output label alike), matching the literal `effortLevel` JSON key it maps to on the session side and staying unambiguous on the subprocess side.

**No-Duplication Contract:**

`map_model_shorthand()` (the shared shorthand table used for `scope::session model::`) already exists as the extracted inner function from Feature 034/069's original design — no duplicate mapping table is introduced. `effort_level::` validation on `scope::session` is a plain allow-list check (`low`/`normal`/`high`/`max` are the literal values written — no shorthand-to-ID mapping table exists or is needed, unlike model). `effort_level::` validation on `scope::subprocess` is a plain allow-list check against `low`/`medium`/`high`/`max`, matching `claude_runner_core::types::EffortLevel`'s own four variants.

`get_session_model()`/`set_session_model()`/`get_session_effort()`/`set_session_effort()` already exist in `claude_profile_core/src/account/session_settings.rs` and are reused unchanged. **`remove_session_effort()` does not exist yet** and must be added, mirroring `set_session_effort()`'s existing read-modify-write pattern but removing the `effortLevel` key instead of inserting it — this is the one net-new low-level function this feature's eventual implementation requires; every other primitive it needs already exists.

**Pre-existing clobbering caveat (carried forward, now stated for both model and effort):** `apply_model_override()` (Feature 062) writes both `model` and `effortLevel` to `settings.json` unconditionally on every `.usage`/`.account.use` invocation that reaches it, regardless of whether either value actually changed. A manual `scope::session model::opus` or `effort_level::high` pin is therefore not necessarily durable — the next automatic rotation-driven `.usage`/`.account.use` call can silently overwrite it. This was already true for `model::` under the old `.model set::` design (undocumented); it is stated explicitly here because it now applies symmetrically to `effort_level::` as well, and operators reading this feature's own docs should not be surprised twice.

### Config Key Inventory

Every persisted, model/effort/provider-relevant key discovered across the workspace, and this feature's exact coverage of it:

| # | File (absolute, `$HOME`-relative) | Key | Valid values | Tier | Controlled by this feature? |
|---|---|---|---|---|---|
| 1 | `$HOME/.claude/settings.json` | `model` | shorthand: `opus`/`sonnet`/`haiku`/full ID | n/a (single file) | Yes — `scope::session model::`/`reset_model::1` |
| 2 | `$HOME/.claude/settings.json` | `effortLevel` | `low`/`normal`/`high`/`max` | n/a | Yes — `scope::session effort_level::`/`reset_effort_level::1` (**new** — no prior direct CLI control existed) |
| 3 | `$HOME/.clr/config.toml` | `model` | any non-empty full model ID | user (write); user+project (read, project wins) | Yes — `scope::subprocess model::`/`reset_model::1` |
| 4 | `$HOME/.clr/config.toml` | `effort` | `low`/`medium`/`high`/`max` | user (write); user+project (read, project wins) | Yes — `scope::subprocess effort_level::`/`reset_effort_level::1` (**new** — no prior CLI control existed at all; previously settable only via `clr`'s own `--effort` flag/env var or manual TOML edit) |
| 5 | `$HOME/.clr/config.toml` | `provider` | free-form string, default `anthropic` | user | No — deliberately out of scope, see `.provider.select` (072) |
| 6 | `$HOME/.clr/config.toml` | `fallback_model` | full model ID | user+project (read, project wins) | No — no clp reader/writer exists on either design; flagged as a future candidate, not silently included |
| — | `$HOME/.clr/prefs.json` | `subprocess_model` | — | — | No — dead, superseded by row 3 (task 410); see [schema/008_clr_prefs_json.md](../schema/008_clr_prefs_json.md) |
| — | *(ephemeral, not a file)* | `imodel::` param on `.usage`/`.account.use`/`.accounts` | `auto`/`sonnet`/`opus`/`haiku`/`keep` | n/a | No — never persisted, distinct mechanism (Feature 026) |
| — | *(ephemeral, not a file)* | `effort::` param on `.usage`/`.account.use`/`.accounts` | `auto`/`low`/`normal`/`high`/`max` | n/a | No — never persisted, distinct mechanism (Feature 026); see Naming/Vocabulary note above for why this feature avoids the same parameter name |

### Acceptance Criteria

- **AC-01**: `clp .model` (no params) → `scope::` defaults to `session`; prints `scope: session (<absolute settings.json path>)`, `model: <shorthand-or-(unset)>`, `effort_level: <value-or-(unset)>`.
- **AC-02**: `clp .model scope::subprocess` → prints `scope: subprocess (<absolute config.toml path>)`, `model: <full-id-or-(unset)>`, `effort_level: <value-or-(unset)>`.
- **AC-03**: `clp .model scope::bad` → exit 1, stderr names valid values `session`, `subprocess`.
- **AC-04**: `clp .model model::opus` (scope `session`, default) writes `"model":"claude-opus-4-8"` to `~/.claude/settings.json`. Exits 0.
- **AC-05**: `clp .model model::sonnet` writes `"model":"claude-sonnet-5"`. Exits 0.
- **AC-06**: `clp .model model::haiku` writes `"model":"claude-haiku-4-5-20251001"`. Exits 0.
- **AC-07**: `clp .model model::default` removes the `"model"` key from `~/.claude/settings.json`; other keys preserved. Exits 0.
- **AC-08**: `clp .model model::bad` (scope `session`) exits 1 with stderr listing `opus`, `sonnet`, `haiku`, `default`.
- **AC-09**: `clp .model scope::subprocess model::claude-opus-4-8` writes `model = "claude-opus-4-8"` to `~/.clr/config.toml`'s user tier. Exits 0.
- **AC-10**: `clp .model scope::subprocess model::` (empty value) exits 1, stderr requires a non-empty model ID.
- **AC-11**: `clp .model effort_level::high` (scope `session`) writes `"effortLevel":"high"` to `~/.claude/settings.json`. Exits 0.
- **AC-12**: `clp .model effort_level::bad` (scope `session`) exits 1 with stderr listing `low`, `normal`, `high`, `max`.
- **AC-13**: `clp .model scope::subprocess effort_level::medium` writes `effort = "medium"` to `~/.clr/config.toml`'s user tier. Exits 0.
- **AC-14**: `clp .model scope::subprocess effort_level::normal` exits 1 — `normal` is a session-only value; stderr for subprocess scope lists `low`, `medium`, `high`, `max`.
- **AC-15**: `clp .model reset_model::1` (scope `session`) removes the `"model"` key from `~/.claude/settings.json`. Exits 0.
- **AC-16**: `clp .model reset_effort_level::1` (scope `session`) removes the `"effortLevel"` key from `~/.claude/settings.json`. Exits 0.
- **AC-17**: `clp .model scope::subprocess reset_model::1` removes `model` from `~/.clr/config.toml`'s user tier; idempotent (exit 0) when already absent or file missing.
- **AC-18**: `clp .model scope::subprocess reset_effort_level::1` removes `effort` from `~/.clr/config.toml`'s user tier; idempotent (exit 0) when already absent or file missing.
- **AC-19**: `clp .model model::opus reset_model::1` (same scope) exits 1, stderr states `model:: and reset_model::1 are mutually exclusive`.
- **AC-20**: `clp .model effort_level::high reset_effort_level::1` (same scope) exits 1, stderr states `effort_level:: and reset_effort_level::1 are mutually exclusive`.
- **AC-21**: `clp .model model::opus reset_effort_level::1` (scope `session`, mixed concepts) applies BOTH actions in one call — writes `model`, removes `effortLevel`. Exits 0.
- **AC-22**: `clp .model scope::subprocess model::claude-opus-4-8 effort_level::max` writes both `model` and `effort` to `~/.clr/config.toml`'s user tier in the same call; other existing keys preserved.
- **AC-23**: `clp .model format::json` (get mode, scope `session`) prints `{"scope":"session","path":"<absolute path>","model":"<val-or-null>","effort_level":"<val-or-null>"}`.
- **AC-24**: `clp .model scope::subprocess model::VALUE` creates `~/.clr/config.toml` and its parent `.clr/` directory when either is absent. Exits 0.
- **AC-25**: Every get-mode and write-mode text/JSON output names the fully resolved absolute file path — never a `~`-relative path, bare filename, or path omission.
- **AC-26**: `clp .model` is listed in `clp .help` output as a single entry; `.model.select` no longer appears as a distinct listed command.
- **AC-27**: Implementation calls `get_session_model()`/`set_session_model()`/`get_session_effort()`/`set_session_effort()`/`remove_session_effort()` (scope `session`) and `toml_io::get_tiered()`/`set_user_tier()`/`remove_user_tier()` (scope `subprocess`) — no inline file I/O duplicating these primitives.
- **AC-28**: `claude_profile_core::account::remove_session_effort()` is added as a new helper, mirroring `set_session_effort()`'s existing read-modify-write pattern but removing the `effortLevel` key; no such removal helper exists prior to this feature's implementation.

### Features

| File | Relationship |
|------|--------------|
| [069_model_select_command.md](069_model_select_command.md) | Superseded — historical standalone `.model.select` design, absorbed into this feature via `scope::subprocess` |
| [034_explicit_session_model_override.md](034_explicit_session_model_override.md) | `set_model::` side-parameter on `.account.use`/`.usage` — shares `set_session_model()`/`map_model_shorthand()` with this feature; NOT merged into `.model` |
| [062_unified_session_config.md](062_unified_session_config.md) | `set_session_effort()`/`get_session_effort()` — reused unchanged by `scope::session effort_level::`; this feature adds the missing `remove_session_effort()` counterpart |
| [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | `imodel::`/`effort::` ephemeral touch/refresh params — deliberately separate mechanism; motivates this feature's `effort_level::` naming to avoid a same-string, different-meaning collision |
| [072_inference_provider_selection.md](072_inference_provider_selection.md) | `.provider.select` — shares `~/.clr/config.toml` and `toml_io` primitives via `scope::subprocess` conceptually, but remains its own command; not merged here |
| [068_models_list_command.md](068_models_list_command.md) | `.models` — discover full model IDs to pass to `scope::subprocess model::` |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/055_set.md](../cli/param/055_set.md) | Retired — superseded by `model::` (this feature) |
| [cli/param/075_scope.md](../cli/param/075_scope.md) | `scope::` parameter specification — backing-store router, new for this feature |
| [cli/param/076_model_value.md](../cli/param/076_model_value.md) | `model::` parameter specification — new for this feature |
| [cli/param/077_effort_level.md](../cli/param/077_effort_level.md) | `effort_level::` parameter specification — new for this feature |
| [cli/param/078_reset_model.md](../cli/param/078_reset_model.md) | `reset_model::` parameter specification — new for this feature |
| [cli/param/079_reset_effort_level.md](../cli/param/079_reset_effort_level.md) | `reset_effort_level::` parameter specification — new for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `src/commands/model.rs` | `.model` command handler: `scope::`-routed get/set/reset for `model` and `effort_level` across both stores |
| `src/commands/model_select.rs` | Retired — logic absorbed into `src/commands/model.rs`'s `scope::subprocess` branch |
| `src/registry.rs` | `.model` command and `scope::`/`model::`/`effort_level::`/`reset_model::`/`reset_effort_level::`/`format::` parameter registration; `.model.select` kept registered as a stub emitting a migration error (mirrors the `.token.status`/`.account.assign` precedent) |
| `claude_profile_core/src/account/session_settings.rs` | `get_session_model()`/`set_session_model()`/`get_session_effort()`/`set_session_effort()` (existing, reused); `remove_session_effort()` (new) |
| `claude_core/src/toml_io.rs` | `get_tiered()`/`set_user_tier()`/`remove_user_tier()` (existing, reused against both `model` and `effort` keys) |
| `src/usage/types.rs` | `map_model_shorthand()` (existing, reused, unchanged) |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/feature/035_model_command.md` | Feature test spec — FT-01 through FT-27, 1:1 with AC-01 through AC-27 |
| `tests/docs/cli/command/17_model.md` | Command-level integration test spec — IT-01 through IT-27, mirrors FT-01 through FT-27 |
| `tests/docs/cli/command/20_model_select.md` | `.model.select` retirement-stub test spec — IT-01 through IT-03, covers all 3 invocation forms' migration-error behavior |

### Schema

| File | Relationship |
|------|-------------|
| [schema/006_settings_json.md](../schema/006_settings_json.md) | `model`/`effortLevel` fields in `~/.claude/settings.json` — read/written by `scope::session` |
| [claude_core/docs/api/002_toml_io.md](../../../claude_core/docs/api/002_toml_io.md) | `~/.clr/config.toml` flat-TOML format — `model`/`effort` keys read/written by `scope::subprocess` |
