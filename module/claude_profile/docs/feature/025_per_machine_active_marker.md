# Feature: Per-Machine Active Marker

### Scope

- **Purpose**: Eliminate git churn on shared credential stores by making the active-account marker file specific to each machine and user.
- **Responsibility**: Documents the `active_marker_filename()` API and the `_active_{hostname}_{user}` naming convention.
- **In Scope**: Marker filename derivation; `.gitignore` exclusion of `_active_*`; `other_machines_active()` API for reading all non-own `_active_*` markers.
- **Out of Scope**: Switching logic (→ 004_account_use.md); prefix resolution in general (→ 015_name_shortcut_syntax.md).

### Design

#### Per-machine marker

Previously `switch_account()` and `save()` wrote a single shared `_active` file to the credential store. In a multi-machine setup where the credential store is version-controlled, every account switch on any machine produced a git modification — causing noise and implicit conflicts.

The new design writes `_active_{hostname}_{user}` instead. Filename derivation, sanitization rules, and `.gitignore` convention are documented in [schema/005_active_marker.md](../schema/005_active_marker.md).

`active_marker_filename()` is the single source of truth for the marker filename. All reads and writes go through this function; no caller hard-codes `"_active"`.

**No migration**: If an old `_active` file exists, it is silently ignored. Run `.account.use` once on each machine after upgrading to populate the machine-specific marker.

### Acceptance Criteria

- **AC-01**: After `.account.use name::alice@home.com`, the credential store contains `_active_{hostname}_{user}` containing `alice@home.com`; no file named `_active` is created.
- **AC-02**: `active_marker_filename()` returns `_active_<hostname>_<user>` where hostname and user reflect the running machine.
- **AC-03**: Two machines sharing the same credential store directory can each have their own active account without affecting each other.
- **AC-04**: `_active_*` is listed in `.gitignore` at the repository root.
- **AC-05**: `other_machines_active(credential_store)` returns a `HashSet<String>` containing account names found in every `_active_*` file in the credential store EXCEPT the current machine's own marker (as returned by `active_marker_filename()`). Each file's content is trimmed; empty strings after trimming are excluded. Missing or unreadable files are silently skipped.

### Features

| File | Relationship |
|------|--------------|
| [002_account_save.md](002_account_save.md) | Name resolution: `account_save_routine()` uses `oauthAccount.emailAddress` as primary, `_active_{hostname}_{user}` marker as fallback when `name::` is omitted (AC-08, AC-16, BUG-209, BUG-212) |
| [004_account_use.md](004_account_use.md) | Base switch behavior; design step 4 updated |
| [015_name_shortcut_syntax.md](015_name_shortcut_syntax.md) | Prefix resolution; AC-11 added for exact-local-part match |
| [029_account_host_metadata.md](029_account_host_metadata.md) | `resolve_hostname()` fallback chain shared with host auto-capture |
| [032_account_assign.md](032_account_assign.md) | Marker-only write for any `USER@MACHINE` pair via `active::USER@MACHINE name::X` (Feature 064); contrast point for `.account.use` full credential rotation |
| [036_account_ownership.md](036_account_ownership.md) | `current_identity()` uses the same `resolve_hostname()` fallback chain to form `$USER@<hostname>` |
| [009_token_usage.md](009_token_usage.md) | Sessions table (AC-33, AC-34): `.usage` renders all `_active_*` markers as `{user}@{host} → account` table after the footer; controlled by `who::` parameter |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/005_atomic_switching.md](../invariant/005_atomic_switching.md) | Atomicity invariant; `_active` marker note updated |

### Sources

| File | Relationship |
|------|--------------|
| `module/claude_profile_core/src/account.rs` | `active_marker_filename()` — derives per-machine marker name; `read_active_marker()`, `switch_account()`, `save()`, `delete()` — use it; `other_machines_active(store)` — reads all `_active_*` except own marker, returns `HashSet<String>` of account names |
| `src/commands/shared.rs`, `src/commands/account_ops.rs` | `resolve_account_name()` — exact-local-part match priority (defined in `shared.rs`); `account_save_routine()` — reads `oauthAccount.emailAddress` from `~/.claude.json` as primary name inference source when `name::` is omitted; falls back to `_active_{hostname}_{user}` marker (BUG-212 fix, TSK-215) |
| `src/commands/accounts.rs` | `accounts_routine()` `active::` path — writes or clears `_active_{machine}_{user}` for any `USER@MACHINE` pair without credential rotation (Feature 064: replaced former `assign::1` + `for::` params) |
| `src/usage/refresh.rs`, `src/usage/touch.rs` | `apply_refresh` and `apply_touch` snapshot/restore the `_active` marker around per-account processing; snapshot+restore removed by BUG-211 fix (`save()` now writes conditionally via `update_marker=false` — see AC-15 in [002_account_save.md](002_account_save.md)); reads removed in Phases 3/4 of TSK-214 |
| `.gitignore` | `_active_*` pattern excludes per-machine markers from version control |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/account_mutations_test.rs` (aw16, aw17) | aw16: exact-local-part wins over prefix; aw17: no exact match falls through to ambiguous |
| `module/claude_profile_core/tests/account_test.rs` | FT-11: other_machines_active returns others' names; FT-12: returns empty HashSet when only own marker or empty store |

### Schema

| File | Relationship |
|------|-------------|
| [schema/005_active_marker.md](../schema/005_active_marker.md) | Marker filename derivation, content format, and `.gitignore` convention — extracted from this feature |
| [pitfall/004_account_identity_pitfalls.md](../pitfall/004_account_identity_pitfalls.md) | BUG-308 (test fixture collision), BUG-212 (stale marker as name source) |
