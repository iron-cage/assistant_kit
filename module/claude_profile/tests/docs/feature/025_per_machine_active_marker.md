# Test: Feature 025 — Per-Machine Active Marker

### Scope

- **Purpose**: Test cases for per-machine active marker filenames and isolation.
- **Source**: `docs/feature/025_per_machine_active_marker.md`
- **Covers**: AC-01 through AC-05 (FT-06..FT-10, FT-13 cross-reference ACs in Features 002, 009, 015)

Feature behavioral requirement test cases for `docs/feature/025_per_machine_active_marker.md`. Each FT case maps to one acceptance criterion. Prefix resolution edge cases are in [cli/command/001_account.md](../../../docs/cli/command/001_account.md) and [feature/015_name_shortcut_syntax.md](../../../docs/feature/015_name_shortcut_syntax.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `.account.use` writes `_active_{hostname}_{user}`, not `_active` | AC-01 | Integration |
| FT-02 | `.account.save` writes per-machine marker (save/use symmetry) | AC-01 | Integration |
| FT-03 | `active_marker_filename()` returns `_active_<hostname>_<user>` format | AC-02 | Unit (implicit) |
| FT-04 | Two machines share a credential store without overwriting each other's marker | AC-03 | Design invariant |
| FT-05 | `_active_*` is excluded from version control via `.gitignore` | AC-04 | Static config |
| FT-06 | `clp .account.use i1` resolves exact local-part match unambiguously | AC-11 (015) | Integration (→ 015) |
| FT-07 | `clp .account.use a` exits 1 when no exact local-part match, two prefix hits | AC-06 (015) | Integration (→ 015) |
| FT-08 | `clp .account.use i1` exits 1 when no `i1@` account and `i11@`/`i12@` both match | AC-06, AC-11 (015) | Integration (→ 015) |
| FT-09 | `.account.save` (no `name::`) — stale top-level `emailAddress` ignored (BUG-209 regression) | AC-08 (002) | Integration (BUG-209) |
| FT-10 | `.account.save` (no `name::`) — `oauthAccount.emailAddress` present, overrides stale `_active` marker (BUG-212) | AC-08 (002) | Integration (BUG-212) |
| FT-11 | `other_machines_active()` returns other machines' account names, excludes own marker | AC-05 | Unit |
| FT-12 | `other_machines_active()` returns empty HashSet when only own marker or empty store | AC-05 | Unit |
| FT-13 | `.usage` sessions table renders `_active_*` markers as `{user}@{host}` → account rows | AC-33 (009) | Integration (→ 009) |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | .account.use writes per-machine marker | AC-01 | Marker |
| FT-02 | .account.save writes per-machine marker | AC-01 | Marker |
| FT-03 | active_marker_filename format starts with `_active_` | AC-02 | Unit |
| FT-04 | Machine independence — distinct filenames guarantee isolation | AC-03 | Design |
| FT-05 | .gitignore excludes `_active_*` | AC-04 | Static Config |
| FT-06 | Exact local-part match resolves unambiguously | AC-11 (015) | Prefix Resolution |
| FT-07 | Ambiguous prefix with no exact match exits 1 | AC-06 (015) | Prefix Resolution |
| FT-08 | Prefix `i1` exits 1 when only `i11@`/`i12@` exist (no exact match) | AC-06, AC-11 (015) | Prefix Resolution |
| FT-09 | .account.save ignores stale top-level `emailAddress` (BUG-209) | AC-08 (002) | Name Resolution |
| FT-10 | .account.save uses `oauthAccount.emailAddress` when present, ignores stale `_active` marker | AC-08 (002) | Name Resolution |
| FT-11 | other_machines_active returns other machines' names, excludes own marker | AC-05 | Unit |
| FT-12 | other_machines_active returns empty HashSet when only own marker or empty store | AC-05 | Unit |
| FT-13 | `.usage` sessions table renders `_active_*` markers as session identity → account rows | AC-33 (009) | Integration (→ 009) |

**Total:** 13 FT cases

---

### FT-01: `.account.use` writes `_active_{hostname}_{user}`, not `_active`

- **Given:** A fresh credential store with one saved account `alice@home.com`. No `_active` file present.
- **When:** `clp .account.use alice@home.com`
- **Then:** The credential store contains a file named `_active_{hostname}_{user}` (as returned by `active_marker_filename()`) whose content is `alice@home.com`. No file named `_active` (bare) is created.
- **Exit:** 0
- **Source fn:** `aw07_switch_updates_active_marker` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/025_per_machine_active_marker.md AC-01](../../../docs/feature/025_per_machine_active_marker.md)

---

### FT-02: `.account.save` writes per-machine marker

- **Given:** A running `~/.claude/.credentials.json` with generic credentials (not yet tied to any saved account name).
- **When:** `clp .account.save name::work@acme.com`
- **Then:** The credential store contains `_active_{hostname}_{user}` = `"work@acme.com"`. No bare `_active` file is created. The fix in `save()` uses `active_marker_filename()` (not the old hard-coded `"_active"`).
- **Exit:** 0
- **Source fn:** `as16_save_writes_active_marker` (in `account_mutations_test_b.rs`)
- **Source:** [feature/025_per_machine_active_marker.md AC-01](../../../docs/feature/025_per_machine_active_marker.md)

---

### FT-03: `active_marker_filename()` returns `_active_<hostname>_<user>` format

- **Given:** Test environment with `HOSTNAME` and `USER` env vars set (inherited from shell).
- **When:** `active_marker_filename()` is called.
- **Then:** The returned string starts with `_active_` and contains at least one `_` after the prefix (i.e., `_active_{hostname}_{user}`). Tests that call `active_marker_filename()` to locate the marker file implicitly validate that the function returns the correct filename for the current machine.
- **Exit:** n/a (unit test, implicit)
- **Note:** Validated by any test that writes a marker via `save()`/`switch_account()` and then reads it back using `active_marker_filename()` to locate the file. FT-01 and FT-02 both demonstrate this. Dedicated unit tests: `switch_account_updates_active_marker` and `list_marks_active_account_via_active_marker` in `tests/account_tests.rs`.
- **Source fn:** `switch_account_updates_active_marker` (in `tests/account_tests.rs`)
- **Source:** [feature/025_per_machine_active_marker.md AC-02](../../../docs/feature/025_per_machine_active_marker.md)

---

### FT-04: Two machines share a credential store without overwriting each other's marker

- **Given:** A credential store shared between machine A (`_active_A_devuser`) and machine B (`_active_B_devuser`).
- **When:** Machine A runs `clp .account.use account-a@example.com` while machine B has `_active_B_devuser` = `account-b@example.com`.
- **Then:** Machine A writes `_active_A_devuser`; machine B's `_active_B_devuser` is untouched. Each machine reads its own marker independently.
- **Note:** Design invariant guaranteed by distinct filenames (`HOSTNAME` + `USER` combination). No isolated test required; independence follows architecturally from non-overlapping filename keys. Both FT-01 and FT-02 implicitly rely on this property via TempDir HOME isolation.
- **Source fn:** (design invariant — no dedicated test)
- **Source:** [feature/025_per_machine_active_marker.md AC-03](../../../docs/feature/025_per_machine_active_marker.md)

---

### FT-05: `_active_*` is excluded from version control via `.gitignore`

- **Given:** The repository `.gitignore` at `dev/.gitignore`.
- **When:** The file is inspected.
- **Then:** It contains the pattern `_active_*`, excluding all per-machine marker files from version control.
- **Note:** Verified by static inspection of `dev/.gitignore` line 31: `_active_*`. Updated as part of Feature 025 implementation.
- **Source fn:** (static config — no dedicated test)
- **Source:** [feature/025_per_machine_active_marker.md AC-04](../../../docs/feature/025_per_machine_active_marker.md)

---

### FT-06: `clp .account.use i1` resolves exact local-part match unambiguously

- **Given:** Three saved accounts: `i1@example.com`, `i11@example.com`, `i12@example.com`. Prefix `i1` matches all three via `starts_with`, but `i1@example.com` has local part equal to `i1` exactly.
- **When:** `clp .account.use i1`
- **Then:** Exits 0. Active marker contains `i1@example.com`. The exact-local-part check resolves `i1@example.com` before reaching the prefix scan — no ambiguity error.
- **Exit:** 0
- **Source fn:** `aw16_exact_local_part_wins_over_ambiguous_prefix` (in `account_mutations_test_b.rs`)
- **Source:** [feature/015_name_shortcut_syntax.md AC-11](../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-07: `clp .account.use a` exits 1 when no exact local-part match, two prefix hits

- **Given:** Two saved accounts: `alice@example.com` and `amy@example.com`. Prefix `a` matches both via `starts_with`; neither has local part `a` exactly.
- **When:** `clp .account.use a`
- **Then:** Exits 1. Stderr contains "ambiguous". No account switch occurs. The exact-local-part check finds no match, falling through to prefix scan which reports ambiguity.
- **Exit:** 1
- **Source fn:** `aw15_use_prefix_ambiguous_exits_1` (in `account_mutations_test_b.rs`)
- **Source:** [feature/015_name_shortcut_syntax.md AC-06](../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-08: `clp .account.use i1` exits 1 when only `i11@`/`i12@` exist (no exact match)

- **Given:** Two saved accounts: `i11@example.com` and `i12@example.com`. No `i1@example.com` account exists. Prefix `i1` matches both via `starts_with`; neither has local part exactly `i1`.
- **When:** `clp .account.use i1`
- **Then:** Exits 1. Stderr contains "ambiguous". The exact-local-part check finds no match (no account with local part `i1`), falls through to prefix scan, which finds two matches and reports ambiguity.
- **Exit:** 1
- **Source fn:** `aw17_use_prefix_ambiguous_no_exact_local_part_exits_1` (in `account_mutations_test_b.rs`)
- **Source:** [feature/015_name_shortcut_syntax.md AC-06, AC-11](../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-09: `.account.save` (no `name::`) — stale top-level `emailAddress` is never read

- **Given:** `~/.claude.json` contains BOTH a stale top-level `emailAddress = "a@test.com"` (never updated by `switch_account()`) AND `oauthAccount.emailAddress = "b@test.com"` (kept in sync). The per-machine active marker (`_active_{hostname}_{user}`) also contains `"b@test.com"` (set by a prior `.account.use b@test.com`) — agreeing with `oauthAccount.emailAddress`, so this fixture does not itself discriminate between the two sources.
- **When:** `clp .account.save` (no `name::` argument)
- **Then:** Exits 0. Stdout contains `"b@test.com"` and does NOT contain `"a@test.com"` — the stale top-level `emailAddress` field is never read. The per-machine active marker still reads `b@test.com` after save.
- **Exit:** 0
- **Source fn:** `mre_bug_209_account_save_uses_active_marker_not_stale_email` (in `account_relogin_test_b.rs`)
- **Note:** Tests the BUG-209 fix: top-level `emailAddress` is stale and never read. The primary-vs-fallback precedence between `oauthAccount.emailAddress` and the `_active` marker (added later by BUG-212, and part of the general two-level inference read in `account_ops.rs`) is exercised where the two sources actually disagree by FT-10.
- **Source:** [feature/002_account_save.md AC-08](../../../docs/feature/002_account_save.md)

---

### FT-10: `.account.save` (no `name::`) — `oauthAccount.emailAddress` overrides stale `_active` marker (BUG-212)

- **Given:** `~/.claude/.credentials.json` exists with live credentials. `~/.claude.json` contains `{"oauthAccount":{"emailAddress":"i5@example.com"}}` (fresh — written by external OAuth login). The per-machine active marker (`_active_{hostname}_{user}`) contains `"i2@example.com"` (stale — last written by a prior clp session). No `name::` argument is passed.
- **When:** `clp .account.save` (no `name::` argument)
- **Then:** Exits 0. Output reads `saved current credentials as 'i5@example.com'`. `{credential_store}/i5@example.com.credentials.json` created. `{credential_store}/i2@example.com.credentials.json` NOT created. The `_active` marker is not consulted when `oauthAccount.emailAddress` provides a non-empty value.
- **Exit:** 0
- **Source fn:** `mre_bug_212_account_save_stale_marker_uses_oauth_email` (in `account_relogin_test_b.rs`)
- **Note:** BUG-212 regression guard. `oauthAccount.emailAddress` is written by both clp ops and external OAuth login; `_active` is written only by clp ops — external login leaves it stale. Primary over fallback precedence is the two-level inference introduced by TSK-215.
- **Source:** [feature/002_account_save.md AC-08, AC-16](../../../docs/feature/002_account_save.md)

---

### FT-11: `other_machines_active()` returns other machines' account names, excludes own marker

- **Given:** A credential store (TempDir) containing three `_active_*` files: the current machine's own marker (as returned by `active_marker_filename()`) containing `"own@test.com"`, a second file `_active_machine2_devuser` containing `"alice@test.com"`, and a third file `_active_machine3_devuser2` containing `"bob@test.com"`.
- **When:** `other_machines_active(&store_path)` is called.
- **Then:** Returns a `HashSet<String>` containing exactly `{"alice@test.com", "bob@test.com"}`. The own marker's content (`"own@test.com"`) is NOT present in the result. The set has exactly 2 elements.
- **Note:** File names for the other machines must differ from `active_marker_filename()` — use hard-coded names like `_active_machine2_devuser` to guarantee they differ from the current machine's marker regardless of environment.
- **Source fn:** `test_ft11_025_other_machines_active_returns_others` (in `claude_profile_core/tests/account_test.rs`)
- **Source:** [feature/025_per_machine_active_marker.md AC-05](../../../docs/feature/025_per_machine_active_marker.md)

---

### FT-12: `other_machines_active()` returns empty HashSet when only own marker or empty store

- **Given (Case A):** A credential store containing only the current machine's own marker (`active_marker_filename()`). **Given (Case B):** An empty credential store directory with no `_active_*` files.
- **When:** `other_machines_active(&store_path)` is called in each case.
- **Then:** Returns an empty `HashSet<String>` in both cases.
- **Note:** Case A verifies the own-marker exclusion filter. Case B verifies graceful empty-directory handling. Both are covered in the same test function.
- **Source fn:** `test_ft12_025_other_machines_active_empty_when_only_own` (in `claude_profile_core/tests/account_test.rs`)
- **Source:** [feature/025_per_machine_active_marker.md AC-05](../../../docs/feature/025_per_machine_active_marker.md)

---

### FT-13: `.usage` sessions table renders `_active_*` markers as session identity → account rows

- **Given:** A credential store with 3 `_active_*` marker files: `_active_testhost1_tst1` containing `"alice@test.com"`, `_active_testhost2_tst2` containing `"bob@test.com"`, and the current machine's own marker (as returned by `active_marker_filename()`) containing `"own@test.com"`. The `accounts` list passed to the renderer holds one unrelated synthetic `AccountQuota` (`mk_aq_ok(10.0)`) — the sessions table is driven entirely by reading marker files from the store path, not by the `accounts` list content.
- **When:** `render_text(&accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols, None, None, Some(spath), None, false)` is called directly (not via CLI) with `who=None` — auto-shows because marker_count=3 > 1.
- **Then:** Output contains "Sessions" (table header appears). Each `_active_*` marker is rendered as a row: `Session` = `{user}@{host}` parsed from filename `_active_{host}_{user}` (`_active_testhost1_tst1` → `"tst1@testhost1"`, `_active_testhost2_tst2` → `"tst2@testhost2"`), and `Account` = file content (`"alice@test.com"`, `"bob@test.com"`). The own session's account cell shows `"own@test.com ✓"`.
- **Exit:** N/A (direct `render_text` function call — no CLI, no exit code)
- **Note:** Cross-feature integration: this test validates Feature 025's `_active_*` marker data as consumed by Feature 009's sessions table (AC-33). The data source (marker files under the store path) is Feature 025's responsibility; the rendering is Feature 009's. BUG-308 fix: synthetic hostnames (`testhost1`/`testhost2`) replaced the original hardcoded `_active_devbox_devuser`/`_active_buildbox_devuser2` names, which could collide with `active_marker_filename()` on a machine actually named `devbox`/`devuser`.
- **Source fn:** `ft13_025_sessions_table_parses_marker_identity_from_filename` (in `tests/usage/render_tests_b.rs`)
- **Source:** [feature/009_token_usage.md AC-33](../../../docs/feature/009_token_usage.md), [feature/025_per_machine_active_marker.md AC-05](../../../docs/feature/025_per_machine_active_marker.md)
