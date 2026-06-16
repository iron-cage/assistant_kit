# Task 003 — Implement `.config` Command with 4-Layer Resolution

## Execution State

- **State:** 🎯 (Verified)
- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **Priority:** 2
- **Value:** 8
- **Easiness:** 4
- **Safety:** 7
- **Advisability:** 448
- **Dir:** .
- **Validated By:** null
- **Validation Date:** null
- **Blocked Reason:** null
- **Closes:** Q-01, Q-02

## MOST Goal

- **Motivated:** The three `.settings.*` commands expose only the raw JSON value from `~/.claude/settings.json`, ignoring env var overrides, project-level config, and catalog defaults. Users cannot determine the effective model, theme, or other settings without consulting multiple sources manually. A unified `.config` command with 4-layer resolution solves this.
- **Observable:** `cm .config` prints all resolved settings with source annotations; `cm .config key::model` prints the effective model value (env var if set, else user config, else `claude-sonnet-4-6` default); `cm .config key::theme value::dark` writes atomically to user settings; `cm .config key::theme unset::1` removes the key. All 17 integration tests in `tests/docs/cli/command/013_config.md` pass. All 12 FT cases in `tests/docs/feature/006_config_command.md` pass. Level 3 clean.
- **Scoped:** Changes confined to `claude_version_core/src/` (two new modules) and `claude_version/src/` (command handler + lib.rs registration). No changes to `render.rs`, `adapter.rs`, `output.rs`, or any other crate.
- **Testable:** `bash verb/test l::3` passes. `cm .config key::model` executed locally shows `claude-sonnet-4-6 (default)` when `CLAUDE_MODEL` is unset. `cm .config key::model` shows env value when `CLAUDE_MODEL=claude-opus-4-6`.

## In Scope

- `claude_version_core/src/config_catalog.rs` — new module: `SettingDef` struct, catalog `Vec<SettingDef>` with 7 known settings, `fn catalog() -> &'static [SettingDef]`
- `claude_version_core/src/config_resolve.rs` — new module: `Layer` enum (Env/Project/User/Default/Absent), `ResolvedValue` struct, `fn resolve(key, home_dir, cwd, catalog) -> ResolvedValue`, `fn resolve_all(home_dir, cwd, catalog) -> Vec<(String, ResolvedValue)>`
- `claude_version_core/src/lib.rs` — add `pub mod config_catalog; pub mod config_resolve;`
- `claude_version/src/commands.rs` — new `handle_config()` handler routing show-all / get / set / unset modes
- `claude_version/src/lib.rs` — register `.config` command and 4 new parameters (`scope::`, `unset::`, and extend `key::` / `value::` to `.config`)
- Integration tests in `claude_version/tests/integration/config_commands_test.rs` — 17 tests covering 013_config.md + 006_config_command.md FTs
- Unit tests in `claude_version_core/tests/` — 6 AT tests covering 002_config_resolution.md

## Out of Scope

- No changes to `settings_io.rs`, `adapter.rs`, `output.rs`, `render.rs`
- No removal of `.settings.*` commands (deprecation only — they remain functional)
- `claude_profile`, `claude_runner`, and other crates — hard repo/workspace boundary
- Persistent config file for cm itself (config_param.md documents cm has no config file)
- Multi-scope read for project config ancestor walk across symlinks (standard parent walk only)

## Work Procedure

### Phase 1 — config_catalog.rs (claude_version_core)

1. Create `claude_version_core/src/config_catalog.rs`:
   - Define `pub struct SettingDef { pub key: &'static str, pub env_var: Option<&'static str>, pub default: Option<&'static str> }`
   - Define `pub fn catalog() -> &'static [SettingDef]` returning 7 entries per `docs/algorithm/002_config_resolution.md` § Catalog
   - Add `#[inline]` on all pub fns; derive nothing (no Clone/Debug needed for static data)

2. Add `pub mod config_catalog;` to `claude_version_core/src/lib.rs`

3. Run Level 1 — must compile with no warnings.

### Phase 2 — config_resolve.rs (claude_version_core)

4. Create `claude_version_core/src/config_resolve.rs`:
   - `pub enum Layer { Env, Project, User, Default, Absent }` with `Display` impl for source annotation text
   - `pub struct ResolvedValue { pub value: Option<String>, pub source: Layer }`
   - `pub fn resolve(key: &str, home_dir: &Path, cwd: &Path, catalog: &[SettingDef]) -> ResolvedValue` — 4 steps per algorithm doc
   - `pub fn resolve_all(home_dir: &Path, cwd: &Path, catalog: &[SettingDef]) -> Vec<(String, ResolvedValue)>` — union all keys, sorted
   - Project config search: walk parent dirs from `cwd` up to filesystem root looking for `.claude/settings.json`; stop at first `.claude/settings.json` found, or when crossing a git repository root (detected by presence of `.git` entry), or at filesystem root — whichever comes first (per `docs/algorithm/002_config_resolution.md` Step 2)
   - Reuse `claude_version_core::settings_io::read_all_settings()` for file reads (the function already exists in `claude_version_core/src/settings_io.rs`; `read_all_settings()` takes a `&Path` and returns a parsed map)

5. Add `pub mod config_resolve;` to `claude_version_core/src/lib.rs`

6. Write unit tests in `claude_version_core/tests/` covering AT-01 through AT-06 (002_config_resolution.md)

7. Run Level 1 in `claude_version_core`.

### Phase 3 — .config command handler (claude_version)

8. In `claude_version/src/commands.rs`, add `pub fn handle_config(params, credential_store, verbose, format, dry, key, value, scope, unset)`:
   - Mode dispatch table per feature doc 006 § Design
   - Show-all: call `claude_version_core::config_resolve::resolve_all()`; render with source annotation per verbosity
   - Get: call `claude_version_core::config_resolve::resolve()`; render value + source; exit 0 (even if absent)
   - Set: delegate to `settings_io::set_setting()` from **`claude_version/src/settings_io.rs`** (the same `set_setting` already used by `.settings.set` in `commands.rs`); use target scope path; respect `dry::1`
   - Unset: read file via `claude_version/src/settings_io.rs::read_all_settings()`, remove key, write atomically; respect `dry::1`
   - Resolve target scope path: `scope::user` → `<home>/.claude/settings.json`; `scope::project` → `<cwd>/.claude/settings.json` (create dir + file if absent)
   - JSON format: include `source` field per key in the output object
   - Invalid param combinations: exit 1 with specific message per command doc invalid-combinations table
   - Pitfall: both `claude_version/src/settings_io.rs` and `claude_version_core/src/settings_io.rs` exist and have the same functions. The handler (`commands.rs`) uses the local `settings_io.rs` in `claude_version/src/`. The resolver (`config_resolve.rs` in `_core`) uses the one in `claude_version_core/src/`. Both have the same API — no cross-crate confusion needed.

9. Register `.config` in `claude_version/src/lib.rs`:
   - Add command `".config"` with parameter list: `key::` (optional String), `value::` (optional String), `scope::` (String, default `"user"`), `unset::` (bool, default false), `format::`, `v::`, `dry::` — exactly how other commands are registered in `lib.rs`
   - The existing `key::` and `value::` parameters in the registry are shared across commands: check how `lib.rs` binds parameters to commands (via a parameter-to-command allowlist or similar). Add `".config"` to the allowed-commands list for `key::` and `value::` parameters in the same way `.settings.get` and `.settings.set` are bound. Inspect the existing registration pattern before writing any code.
   - Add `scope::` (new, String, default `"user"`) and `unset::` (new, bool, default false) as new parameters in the registry

10. Write integration tests in `claude_version/tests/integration/config_commands_test.rs`:
    - 17 IT tests from 013_config.md (show-all, get, set user/project, unset, format::json, env override, arbitrary key, catalog default, dry-run, invalid combinations, HOME unset)
    - 12 FT tests from 006_config_command.md
    - Use isolated HOME temp dir for all tests
    - Update `claude_version/tests/integration/readme.md` — add row for `config_commands_test.rs`

11. Write unit tests in `claude_version_core/tests/` — 6 AT tests from 002_config_resolution.md (AT-01 through AT-06); update `claude_version_core/tests/readme.md` — add row for the resolution test file.

12. Update `claude_version_core/src/lib.rs` doc comment to include the two new modules (`config_catalog`, `config_resolve`) in the module listing.

13. Run Level 3 in `claude_version`. All tests pass, Clippy clean.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `cm .config` (no params, user settings has `theme=dark`) | show-all mode | stdout contains `theme: dark (user)` and catalog defaults with `(default)` annotation |
| `cm .config key::model` with `CLAUDE_MODEL=claude-opus-4-6` | env layer override | stdout shows `claude-opus-4-6 (env)` |
| `cm .config key::model` no env, no user config | catalog default | stdout shows `claude-sonnet-4-6 (default)` |
| `cm .config key::model` no env, user config has `model=claude-haiku-4-5-20251001` | user layer | stdout shows `claude-haiku-4-5-20251001 (user)` |
| `cm .config key::theme value::dark` | set user scope | `~/.claude/settings.json` contains `"theme": "dark"` atomically; exit 0 |
| `cm .config key::theme value::dark scope::project` | set project scope | `{cwd}/.claude/settings.json` contains `"theme": "dark"`; user file unchanged |
| `cm .config key::theme unset::1` (theme exists in user settings) | unset mode | `theme` key removed; other keys preserved; exit 0 |
| `cm .config key::theme value::dark dry::1` | dry-run | no file change; stdout shows preview; exit 0 |
| `cm .config format::json` | JSON show-all | valid JSON with `source` field per key; exit 0 |
| `cm .config value::v` (no key) | invalid combination | exit 1; `key:: is required when value:: is provided` |
| `cm .config key::k value::v unset::1` | invalid combination | exit 1; mutually exclusive |
| HOME unset + `cm .config key::k` | HOME unset | exit 2 |
| `cm .config key::myCustomKey value::myVal` | arbitrary key set | `settings.json` contains `"myCustomKey": "myVal"`; exit 0 |
| `cm .config key::myCustomKey unset::1` (key exists) | arbitrary key unset | key removed from settings.json; exit 0 |
| `cm .config key::autoUpdates value::false` | type inference | `settings.json` contains `"autoUpdates": false` (JSON bool, not string); exit 0 |
| `cm .config` (HOME set, no config files, no env) | show-all absent keys | keys with no value show `(absent)` annotation; catalog keys show `(default)` annotation |

## Acceptance Criteria

- AC-1: `cm .config` with no key/value params prints all resolved settings with source annotations (env/project/user/default/absent).
- AC-2: `cm .config key::<name>` prints the effective value with correct source layer for each of the 4 resolution layers.
- AC-3: `cm .config key::<name> value::<val>` writes atomically to user settings (`~/.claude/settings.json`); exit 0.
- AC-4: `cm .config key::<name> value::<val> scope::project` writes to project-level config (`{cwd}/.claude/settings.json`); user file unchanged.
- AC-5: `cm .config key::<name> unset::1` removes the key from settings; exit 0.
- AC-6: `cm .config format::json` produces valid JSON output with `source` field per key.
- AC-7: Invalid param combinations (`value::` without `key::`, `value::` + `unset::1`) exit 1 with specific error message.
- AC-8: All 17 integration tests in `tests/docs/cli/command/013_config.md` pass.
- AC-9: All 12 FT cases in `tests/docs/feature/006_config_command.md` pass.
- AC-10: All 6 AT tests in `tests/docs/algorithm/002_config_resolution.md` pass.
- AC-11: Level 3 verification passes (nextest + doc tests + Clippy, zero warnings).

## Affected Entities

None (no doc entity directories introduced by this task — implementation only).

## Related Documentation

- `docs/feature/006_config_command.md` — primary feature spec, AC-01 through AC-12
- `docs/algorithm/002_config_resolution.md` — 4-layer resolution algorithm and catalog
- `docs/cli/command/config.md` — CLI reference for `.config`
- `docs/cli/param/11_scope.md` — `scope::` parameter
- `docs/cli/param/12_unset.md` — `unset::` parameter
- `docs/cli/type/06_config_scope.md` — `ConfigScope` type
- `docs/cli/type/07_config_key.md` — `ConfigKey` type
- `docs/cli/param_group/04_config_identity.md` — Config Identity parameter group
- `docs/feature/003_settings_management.md` — deprecated `.settings.*` commands
- `tests/docs/feature/006_config_command.md` — FT-01 through FT-12 test surface
- `tests/docs/algorithm/002_config_resolution.md` — AT-01 through AT-06 test surface
- `tests/docs/cli/command/013_config.md` — IT-1 through IT-17 integration test surface
- `task/decisions.md` — Q-01 (unified command decision), Q-02 (resolution chain decision)

## History

- **[2026-06-09]** `CREATED` — Implement `.config` command with 4-layer resolution in claude_version and claude_version_core.
- **[2026-06-09]** `VERIFY ROUND 1 FAIL` — Implementation Readiness FAIL. Fixed: (1) git-boundary stop condition added to Phase 2 project config walk; (2) settings_io ambiguity resolved — handler uses claude_version/src/settings_io.rs, resolver uses claude_version_core/src/settings_io.rs; (3) param binding guidance added for key::/value:: extension; (4) missing steps added (readme updates, lib.rs doc update); (5) test matrix rows strengthened (arbitrary key unset, type inference observable, show-all absent keys). Re-verification scheduled.
- **[2026-06-09]** `VERIFIED` — Round 2 MAAV all 4 dimensions PASS; task is 🎯 (Verified) and ready to claim.

## Verification Record

Round 2 — 2026-06-09 — All 4 MAAV dimensions PASS.

| Dimension | Agent | Verdict | Notes |
|-----------|-------|---------|-------|
| Scope Coherence | Independent subagent | PASS | Two advisory findings (settings_io pre-condition, param binding underspec) — both addressed in Round 1 fixes |
| MOST Goal Quality | Independent subagent | PASS | All four MOST dimensions pass; minor test-surface indirection noted (not a defect) |
| Value / YAGNI | Independent subagent | PASS | Concrete committed need; no speculative work; one advisory (AC-12 catalog count 6 vs algorithm doc 7 — algorithm doc is authoritative) |
| Implementation Readiness | Independent subagent (Round 2) | PASS | All 6 questions pass after fixes; three minor observations (git boundary detail, unset/project scope test row, unset handler approach) — non-blocking |

