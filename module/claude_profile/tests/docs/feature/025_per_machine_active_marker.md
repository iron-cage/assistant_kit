# Test: Feature 025 — Per-Machine Active Marker

Feature behavioral requirement test cases for `docs/feature/025_per_machine_active_marker.md`. Each FT case maps to one acceptance criterion. Prefix resolution edge cases are in [cli/command/001_account.md](../cli/command/001_account.md) and [feature/015_name_shortcut_syntax.md](../../../../docs/feature/015_name_shortcut_syntax.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `.account.use` writes `_active_{hostname}_{user}`, not `_active` | AC-01 | Integration |
| FT-02 | `.account.save` writes per-machine marker (save/use symmetry) | AC-01 | Integration |
| FT-03 | `active_marker_filename()` returns `_active_<hostname>_<user>` format | AC-02 | Unit (implicit) |
| FT-04 | Two machines share a credential store without overwriting each other's marker | AC-03 | Design invariant |
| FT-05 | `_active_*` is excluded from version control via `.gitignore` | AC-04 | Static config |
| FT-06 | `clp .account.use i1` resolves exact local-part match unambiguously | AC-05 | Integration |
| FT-07 | `clp .account.use a` exits 1 when no exact local-part match, two prefix hits | AC-06 | Integration |
| FT-08 | `clp .account.use i1` exits 1 when no `i1@` account and `i11@`/`i12@` both match | AC-07 | Integration (⏳) |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | .account.use writes per-machine marker | AC-01 | Marker |
| FT-02 | .account.save writes per-machine marker | AC-01 | Marker |
| FT-03 | active_marker_filename format starts with `_active_` | AC-02 | Unit |
| FT-04 | Machine independence — distinct filenames guarantee isolation | AC-03 | Design |
| FT-05 | .gitignore excludes `_active_*` | AC-04 | Static Config |
| FT-06 | Exact local-part match resolves unambiguously | AC-05 | Prefix Resolution |
| FT-07 | Ambiguous prefix with no exact match exits 1 | AC-06 | Prefix Resolution |
| FT-08 | Prefix `i1` exits 1 when only `i11@`/`i12@` exist (no exact match) | AC-07 | Prefix Resolution (⏳) |

**Total:** 8 FT cases

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
- **Source:** [feature/025_per_machine_active_marker.md AC-05](../../../../docs/feature/025_per_machine_active_marker.md)

---

### FT-07: `clp .account.use a` exits 1 when no exact local-part match, two prefix hits

- **Given:** Two saved accounts: `alice@example.com` and `amy@example.com`. Prefix `a` matches both via `starts_with`; neither has local part `a` exactly.
- **When:** `clp .account.use a`
- **Then:** Exits 1. Stderr contains "ambiguous". No account switch occurs. The exact-local-part check finds no match, falling through to prefix scan which reports ambiguity.
- **Exit:** 1
- **Source fn:** `aw15_use_prefix_ambiguous_exits_1` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/025_per_machine_active_marker.md AC-06](../../../../docs/feature/025_per_machine_active_marker.md)

---

### FT-08: `clp .account.use i1` exits 1 when only `i11@`/`i12@` exist (no exact match)

- **Given:** Two saved accounts: `i11@wbox.pro` and `i12@wbox.pro`. No `i1@wbox.pro` account exists. Prefix `i1` matches both via `starts_with`; neither has local part exactly `i1`.
- **When:** `clp .account.use i1`
- **Then:** Exits 1. Stderr contains "ambiguous". The exact-local-part check finds no match (no account with local part `i1`), falls through to prefix scan, which finds two matches and reports ambiguity.
- **Exit:** 1
- **Source fn:** ⏳ TBD — no dedicated test found; this scenario distinguishes AC-07 from AC-06 (prefix looks like exact match but is not)
- **Source:** [feature/025_per_machine_active_marker.md AC-07](../../../../docs/feature/025_per_machine_active_marker.md)
