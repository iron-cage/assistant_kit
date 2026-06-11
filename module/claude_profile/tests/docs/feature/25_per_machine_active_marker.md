# Test: Feature 025 — Per-Machine Active Marker

Feature behavioral requirement test cases for `docs/feature/025_per_machine_active_marker.md`. Each FT case maps to one acceptance criterion. Prefix resolution edge cases are in [cli/command/001_account.md](../cli/command/01_account.md) and [feature/015_name_shortcut_syntax.md](../../../../docs/feature/015_name_shortcut_syntax.md).

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
| FT-09 | `.account.save` (no `name::`) — `oauthAccount.emailAddress` absent, falls back to `_active` marker (BUG-209 regression) | AC-08 (002) | Integration (BUG-209) |
| FT-10 | `.account.save` (no `name::`) — `oauthAccount.emailAddress` present, overrides stale `_active` marker (BUG-212) | AC-08 (002) | Integration (BUG-212) |
| FT-11 | `other_machines_active()` returns other machines' account names, excludes own marker | AC-05 | Unit |
| FT-12 | `other_machines_active()` returns empty HashSet when only own marker or empty store | AC-05 | Unit |

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
| FT-09 | .account.save falls back to `_active` marker when `oauthAccount.emailAddress` absent | AC-08 (002) | Name Resolution |
| FT-10 | .account.save uses `oauthAccount.emailAddress` when present, ignores stale `_active` marker | AC-08 (002) | Name Resolution |
| FT-11 | other_machines_active returns other machines' names, excludes own marker | AC-05 | Unit |
| FT-12 | other_machines_active returns empty HashSet when only own marker or empty store | AC-05 | Unit |

**Total:** 12 FT cases

---

### FT-01: `.account.use` writes `_active_{hostname}_{user}`, not `_active`

- **Given:** A fresh credential store with one saved account `alice@home.com`. No `_active` file present.
- **When:** `clp .account.use alice@home.com`
- **Then:** The credential store contains a file named `_active_{hostname}_{user}` (as returned by `active_marker_filename()`) whose content is `alice@home.com`. No file named `_active` (bare) is created.
- **Exit:** 0
- **Source fn:** `aw07_switch_updates_active_marker` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/025_per_machine_active_marker.md AC-01](../../../../docs/feature/025_per_machine_active_marker.md)

---

### FT-02: `.account.save` writes per-machine marker

- **Given:** A running `~/.claude/credentials.json` with `alice@home.com` credentials.
- **When:** `clp .account.save name::alice@home.com`
- **Then:** The credential store contains `_active_{hostname}_{user}` = `"alice@home.com"`. No bare `_active` file is created. The fix in `save()` uses `active_marker_filename()` (not the old hard-coded `"_active"`).
- **Exit:** 0
- **Source fn:** `as16_save_writes_active_marker` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/025_per_machine_active_marker.md AC-01](../../../../docs/feature/025_per_machine_active_marker.md)

---

### FT-03: `active_marker_filename()` returns `_active_<hostname>_<user>` format

- **Given:** Test environment with `HOSTNAME` and `USER` env vars set (inherited from shell).
- **When:** `active_marker_filename()` is called.
- **Then:** The returned string starts with `_active_` and contains at least one `_` after the prefix (i.e., `_active_{hostname}_{user}`). Tests that call `active_marker_filename()` to locate the marker file implicitly validate that the function returns the correct filename for the current machine.
- **Exit:** n/a (unit test, implicit)
- **Note:** Validated by any test that writes a marker via `save()`/`switch_account()` and then reads it back using `active_marker_filename()` to locate the file. FT-01 and FT-02 both demonstrate this. Dedicated unit tests: `switch_account_updates_active_marker` and `list_marks_active_account_via_active_marker` in `tests/account_tests.rs`.
- **Source fn:** `switch_account_updates_active_marker` (in `tests/account_tests.rs`)
- **Source:** [feature/025_per_machine_active_marker.md AC-02](../../../../docs/feature/025_per_machine_active_marker.md)

---

### FT-04: Two machines share a credential store without overwriting each other's marker

- **Given:** A credential store shared between machine A (`_active_A_user1`) and machine B (`_active_B_user1`).
- **When:** Machine A runs `clp .account.use account-a@example.com` while machine B has `_active_B_user1` = `account-b@example.com`.
- **Then:** Machine A writes `_active_A_user1`; machine B's `_active_B_user1` is untouched. Each machine reads its own marker independently.
- **Note:** Design invariant guaranteed by distinct filenames (`HOSTNAME` + `USER` combination). No isolated test required; independence follows architecturally from non-overlapping filename keys. Both FT-01 and FT-02 implicitly rely on this property via TempDir HOME isolation.
- **Source fn:** (design invariant — no dedicated test)
- **Source:** [feature/025_per_machine_active_marker.md AC-03](../../../../docs/feature/025_per_machine_active_marker.md)

---

### FT-05: `_active_*` is excluded from version control via `.gitignore`

- **Given:** The repository `.gitignore` at `dev/.gitignore`.
- **When:** The file is inspected.
- **Then:** It contains the pattern `_active_*`, excluding all per-machine marker files from version control.
- **Note:** Verified by static inspection of `dev/.gitignore` line 31: `_active_*`. Updated as part of Feature 025 implementation.
- **Source fn:** (static config — no dedicated test)
- **Source:** [feature/025_per_machine_active_marker.md AC-04](../../../../docs/feature/025_per_machine_active_marker.md)

---

### FT-06: `clp .account.use i1` resolves exact local-part match unambiguously

- **Given:** Three saved accounts: `i1@wbox.pro`, `i11@wbox.pro`, `i12@wbox.pro`. Prefix `i1` matches all three via `starts_with`, but `i1@wbox.pro` has local part equal to `i1` exactly.
- **When:** `clp .account.use i1`
- **Then:** Exits 0. Active marker contains `i1@wbox.pro`. The exact-local-part check resolves `i1@wbox.pro` before reaching the prefix scan — no ambiguity error.
- **Exit:** 0
- **Source fn:** `aw16_exact_local_part_wins_over_ambiguous_prefix` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/015_name_shortcut_syntax.md AC-11](../../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-07: `clp .account.use a` exits 1 when no exact local-part match, two prefix hits

- **Given:** Two saved accounts: `alice@example.com` and `amy@example.com`. Prefix `a` matches both via `starts_with`; neither has local part `a` exactly.
- **When:** `clp .account.use a`
- **Then:** Exits 1. Stderr contains "ambiguous". No account switch occurs. The exact-local-part check finds no match, falling through to prefix scan which reports ambiguity.
- **Exit:** 1
- **Source fn:** `aw15_use_prefix_ambiguous_exits_1` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/015_name_shortcut_syntax.md AC-06](../../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-08: `clp .account.use i1` exits 1 when only `i11@`/`i12@` exist (no exact match)

- **Given:** Two saved accounts: `i11@wbox.pro` and `i12@wbox.pro`. No `i1@wbox.pro` account exists. Prefix `i1` matches both via `starts_with`; neither has local part exactly `i1`.
- **When:** `clp .account.use i1`
- **Then:** Exits 1. Stderr contains "ambiguous". The exact-local-part check finds no match (no account with local part `i1`), falls through to prefix scan, which finds two matches and reports ambiguity.
- **Exit:** 1
- **Source fn:** `aw17_use_prefix_ambiguous_no_exact_local_part_exits_1` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/015_name_shortcut_syntax.md AC-06, AC-11](../../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-09: `.account.save` (no `name::`) — fallback to `_active` marker when `oauthAccount.emailAddress` absent

- **Given:** Two saved accounts: `a@test.com` and `b@test.com`. `~/.claude.json` has top-level `emailAddress = "a@test.com"` (stale — no `oauthAccount.emailAddress` field present). The per-machine active marker (`_active_{hostname}_{user}`) contains `"b@test.com"` (set by a prior `.account.use b@test.com`).
- **When:** `clp .account.save` (no `name::` argument)
- **Then:** Exits 0. Output reads `saved current credentials as 'b@test.com'`. The per-machine active marker still reads `b@test.com`. The two-level inference: (1) `oauthAccount.emailAddress` is absent from the JSON → None; (2) fallback to `_active` marker → `b@test.com`. Top-level `emailAddress` is never read.
- **Exit:** 0
- **Source fn:** `mre_bug_209_account_save_uses_active_marker_not_stale_email` (in `tests/cli/account_mutations_test.rs`)
- **Note:** Tests the fallback path. Primary path (`oauthAccount.emailAddress` present, overrides stale marker) is covered by FT-10 (BUG-212).
- **Source:** [feature/002_account_save.md AC-08](../../../../docs/feature/002_account_save.md)

---

### FT-10: `.account.save` (no `name::`) — `oauthAccount.emailAddress` overrides stale `_active` marker (BUG-212)

- **Given:** `~/.claude/.credentials.json` exists with live credentials. `~/.claude.json` contains `{"oauthAccount":{"emailAddress":"i5@wbox.pro"}}` (fresh — written by external OAuth login). The per-machine active marker (`_active_{hostname}_{user}`) contains `"i2@wbox.pro"` (stale — last written by a prior clp session). No `name::` argument is passed.
- **When:** `clp .account.save` (no `name::` argument)
- **Then:** Exits 0. Output reads `saved current credentials as 'i5@wbox.pro'`. `{credential_store}/i5@wbox.pro.credentials.json` created. `{credential_store}/i2@wbox.pro.credentials.json` NOT created. The `_active` marker is not consulted when `oauthAccount.emailAddress` provides a non-empty value.
- **Exit:** 0
- **Source fn:** `mre_bug_212_account_save_stale_marker_uses_oauth_email` (in `tests/cli/account_mutations_test.rs`)
- **Note:** BUG-212 regression guard. `oauthAccount.emailAddress` is written by both clp ops and external OAuth login; `_active` is written only by clp ops — external login leaves it stale. Primary over fallback precedence is the two-level inference introduced by TSK-215.
- **Source:** [feature/002_account_save.md AC-08, AC-16](../../../../docs/feature/002_account_save.md)

---

### FT-11: `other_machines_active()` returns other machines' account names, excludes own marker

- **Given:** A credential store (TempDir) containing three `_active_*` files: the current machine's own marker (as returned by `active_marker_filename()`) containing `"own@test.com"`, a second file `_active_machine2_user1` containing `"alice@test.com"`, and a third file `_active_machine3_user2` containing `"bob@test.com"`.
- **When:** `other_machines_active(&store_path)` is called.
- **Then:** Returns a `HashSet<String>` containing exactly `{"alice@test.com", "bob@test.com"}`. The own marker's content (`"own@test.com"`) is NOT present in the result. The set has exactly 2 elements.
- **Note:** File names for the other machines must differ from `active_marker_filename()` — use hard-coded names like `_active_machine2_user1` to guarantee they differ from the current machine's marker regardless of environment.
- **Source fn:** `test_ft11_025_other_machines_active_returns_others` (in `claude_profile_core/tests/account_test.rs`)
- **Source:** [feature/025_per_machine_active_marker.md AC-05](../../../../docs/feature/025_per_machine_active_marker.md)

---

### FT-12: `other_machines_active()` returns empty HashSet when only own marker or empty store

- **Given (Case A):** A credential store containing only the current machine's own marker (`active_marker_filename()`). **Given (Case B):** An empty credential store directory with no `_active_*` files.
- **When:** `other_machines_active(&store_path)` is called in each case.
- **Then:** Returns an empty `HashSet<String>` in both cases.
- **Note:** Case A verifies the own-marker exclusion filter. Case B verifies graceful empty-directory handling. Both are covered in the same test function.
- **Source fn:** `test_ft12_025_other_machines_active_empty_when_only_own` (in `claude_profile_core/tests/account_test.rs`)
- **Source:** [feature/025_per_machine_active_marker.md AC-05](../../../../docs/feature/025_per_machine_active_marker.md)
