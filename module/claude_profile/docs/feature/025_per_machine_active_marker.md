# Feature: Per-Machine Active Marker

### Scope

- **Purpose**: Eliminate git churn on shared credential stores by making the active-account marker file specific to each machine and user.
- **Responsibility**: Documents the `active_marker_filename()` API and the `_active_{hostname}_{user}` naming convention.
- **In Scope**: Marker filename derivation; `.gitignore` exclusion of `_active_*`.
- **Out of Scope**: Switching logic (→ 004_account_use.md); prefix resolution in general (→ 015_name_shortcut_syntax.md).

### Design

#### Per-machine marker

Previously `switch_account()` and `save()` wrote a single shared `_active` file to the credential store. In a multi-machine setup where the credential store is version-controlled, every account switch on any machine produced a git modification — causing noise and implicit conflicts.

The new design writes `_active_{hostname}_{user}` instead:

- `hostname`: `$HOSTNAME` env var; falls back to `/etc/hostname`; falls back to `"local"`.
- `user`: `$USER` env var; falls back to `$USERNAME`; falls back to `"user"`.
- Both components are sanitized: only alphanumeric, `-`, and `.` are kept; all other characters become `_`.

Example: machine `w003` logged in as `user1` → marker file is `_active_w003_user1`.

The `.gitignore` pattern `_active_*` excludes all per-machine marker files from version control. Each machine is independent — switching on `w003` never affects `w004`.

`active_marker_filename()` is the single source of truth for the marker filename. All reads and writes go through this function; no caller hard-codes `"_active"`.

**No migration**: If an old `_active` file exists, it is silently ignored. Run `.account.use` once on each machine after upgrading to populate the machine-specific marker.

### Acceptance Criteria

- **AC-01**: After `.account.use name::alice@home.com`, the credential store contains `_active_{hostname}_{user}` containing `alice@home.com`; no file named `_active` is created.
- **AC-02**: `active_marker_filename()` returns `_active_<hostname>_<user>` where hostname and user reflect the running machine.
- **AC-03**: Two machines sharing the same credential store directory can each have their own active account without affecting each other.
- **AC-04**: `_active_*` is listed in `.gitignore` at the repository root.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `module/claude_profile_core/src/account.rs` | `active_marker_filename()` — derives per-machine marker name; `read_active_marker()`, `switch_account()`, `save()`, `delete()` — use it |
| source | `src/commands/credentials.rs`, `src/commands/account_ops.rs` | `resolve_account_name()` — exact-local-part match priority; `account_save_routine()` — reads `oauthAccount.emailAddress` from `~/.claude.json` as primary name inference source when `name::` is omitted; falls back to `_active` marker (BUG-212 fix, TSK-215) |
| source | `src/usage/refresh.rs`, `src/usage/touch.rs` | `apply_refresh` and `apply_touch` snapshot/restore the `_active` marker around per-account processing; snapshot+restore removed by BUG-211 fix (`save()` now writes conditionally via `update_marker=false` — see AC-15 in [002_account_save.md](002_account_save.md)); reads removed in Phases 3/4 of TSK-214 |
| config | `.gitignore` | `_active_*` pattern excludes per-machine markers from version control |
| doc | [002_account_save.md](002_account_save.md) | Name resolution: `account_save_routine()` uses `oauthAccount.emailAddress` as primary, `_active` marker as fallback when `name::` is omitted (AC-08, AC-16, BUG-209, BUG-212) |
| doc | [004_account_use.md](004_account_use.md) | Base switch behavior; design step 4 updated |
| doc | [015_name_shortcut_syntax.md](015_name_shortcut_syntax.md) | Prefix resolution; AC-11 added for exact-local-part match |
| doc | [invariant/005_atomic_switching.md](../invariant/005_atomic_switching.md) | Atomicity invariant; `_active` marker note updated |
| test | `tests/cli/account_mutations_test.rs` (aw16, aw17) | aw16: exact-local-part wins over prefix; aw17: no exact match falls through to ambiguous |
