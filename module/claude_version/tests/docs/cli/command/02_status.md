# Test: `.status`

### Scope

- **Purpose**: Integration test cases for the `.status` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for status output.
- **In Scope**: Verbosity levels, output formats, PATH/HOME scenarios, degradation semantics, lock-state compliance visibility.
- **Out of Scope**: Parameter edge cases (→ `../param/`), group interactions (→ `../param_group/`).

Integration test planning for the `.status` command. See [command/readme.md](../../../../docs/cli/command/readme.md) for specification.

## Test Factor Analysis

### Factor 1: `v::` / verbosity (Integer, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default value 1, labeled output | Default behavior |
| 0 | Bare 3-line output (version, processes, account) | Minimum output |
| 1 | Labeled lines: `Version: X`, `Processes: N`, `Account: X` | Nominal |
| 2 | Extended detail (same as 1 if no extra data available) | Maximum detail |
| 3 | Out-of-range integer | Invalid: exit 1 |
| `abc` | Non-integer string | Invalid: exit 1 |

Boundary set: 0, 1, 2, 3 (out-of-range).

### Factor 2: `format::` (String, optional, default "text")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default text output | Default behavior |
| `text` | Explicit text output | Explicit valid |
| `json` | JSON object with required keys | Alternate valid |
| `xml` | Unrecognized value | Invalid: exit 1 |
| `JSON` | Wrong case | Invalid: exit 1 |
| (empty) | Empty string value | Invalid: exit 1 |

### Factor 3: PATH / claude availability (Environmental)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| claude found | Version string available | Happy path |
| empty PATH | Version "not found", still exits 0 | Degraded |

### Factor 4: HOME environment (Environmental)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| set | Account info available | Happy path |
| empty | Account shown as "unknown", exits 0 | Degraded |

### Factor 5: Preferred version in settings (State)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | No "Preferred:" line in output | No preference |
| set | "Preferred:" line shown | Preference stored |

### Factor 6: Unknown parameters

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| none | No unknown params | Happy path |
| present | e.g. `bogus::x` | Invalid: exit 1 |

### Factor 7: Lock-state compliance (State, visible at `v::2`+ text or any `format::json`)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| pinned, compliant | All 5 settings keys + `chmod` match pinned expectation | No mismatch |
| pinned, `chmod` drifted | Versions dir mode is `755` instead of the pinned `555` | Mismatch flagged |
| pinned, `autoUpdates` drifted | Value is `true` instead of the pinned `false` | Mismatch flagged |
| pinned, `autoUpdatesChannel` drifted | Value is `beta` instead of the pinned `stable` | Mismatch flagged |
| pinned, `minimumVersion` drifted | Value diverges from the pinned resolved version | Mismatch flagged |
| pinned, `env.DISABLE_AUTOUPDATER` drifted | Value is `0` instead of the pinned `1` | Mismatch flagged |
| pinned, `env.DISABLE_UPDATES` drifted | Value is `0` instead of the pinned `1` | Mismatch flagged |
| unpinned, compliant | No pin keys set, `chmod` is `755` | No mismatch |
| unpinned, versions dir absent | No pin keys set, versions directory was never created (fresh install) | No mismatch (absent ≠ drift) |
| settings.json corrupted | `settings.json` exists but fails to parse (invalid JSON) | All 6 rows `UNVERIFIABLE` (never a false mismatch or false compliant) |
| settings.json permission-denied | `settings.json` exists, valid JSON, but unreadable (mode 000) | All 6 rows `UNVERIFIABLE` (same as corrupted — any non-`NotFound` read error is untrusted) |
| install interrupted (preference recorded, mechanism not yet applied) | `preferredVersionSpec`/`preferredVersionResolved` are set but the lock-mechanism keys and `chmod` are still at unpinned defaults | True mismatch flagged on all 6 rows — never `UNVERIFIABLE` (settings.json is fully readable) and never silently `Compliant` |
| unpinned, `autoUpdates` explicit `"true"` | Value is `true` (not absent) while unpinned | No mismatch — exercises the OR's second disjunct, not just the absent-value case |
| unpinned, `autoUpdates` drifted to `"false"` | Value is `false` while unpinned (expects `true`/absent) | Mismatch flagged |

---

## Test Matrix

### Positive Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-1 | `.status` exits 0 always | P | 0 | F1=absent, F2=absent | [read_status_test.rs] |
| IT-2 | `.status` with empty PATH → "not found", exits 0 | P | 0 | F3=empty PATH | [read_status_test.rs] |
| IT-3 | `.status v::0` → exactly 3 bare lines | P | 0 | F1=0 | [read_status_test.rs] |
| IT-4 | `.status v::1` → labeled Version/Processes/Account lines | P | 0 | F1=1 | [read_status_test.rs] |
| IT-5 | `.status format::json` → valid JSON with required keys | P | 0 | F2=json | [read_status_test.rs] |
| IT-6 | `.status v::0` has fewer/equal lines than `.status v::1` | P | 0 | F1=0 vs 1 | [read_status_test.rs] |
| IT-7 | `.status` HOME not set → account "unknown", no crash | P | 0 | F4=empty | [read_status_test.rs] |
| IT-8 | `.status` with no preference → no "Preferred" line | P | 0 | F5=absent | [read_status_test.rs] |
| IT-9 | `.status` with preference → shows "Preferred" line | P | 0 | F5=set | [read_status_test.rs] |
| IT-13 | `.status v::2` pinned, all compliant → `Lock:` section, no mismatch | P | 0 | F7=pinned compliant | [read_status_test.rs] |
| IT-14 | `.status v::2` pinned, `chmod` drifted → mismatch flagged | P | 0 | F7=chmod drifted | [read_status_test.rs] |
| IT-15 | `.status v::2` pinned, `autoUpdates` drifted → mismatch flagged | P | 0 | F7=autoUpdates drifted | [read_status_test.rs] |
| IT-16 | `.status v::2` unpinned, compliant → `Lock:` section, no mismatch | P | 0 | F7=unpinned compliant | [read_status_test.rs] |
| IT-17 | `.status v::0`/`v::1` output unchanged by the Lock: feature | P | 0 | F1=0,1 | [read_status_test.rs] |
| IT-18 | `.status format::json` pinned, compliant → `"lock"` object present | P | 0 | F2=json, F7=pinned compliant | [read_status_test.rs] |
| IT-20 | `.status v::2` unpinned, versions dir never created (fresh install) → `chmod: absent`, no false mismatch | P | 0 | F7=unpinned, dir absent | [read_status_test.rs] |
| IT-21 | `.status v::2` settings.json corrupted, versions dir genuinely locked (555) → all 6 rows `UNVERIFIABLE`, no false mismatch | P | 0 | F7=settings corrupted | [read_status_test.rs] |
| IT-22 | `.status format::json` settings.json corrupted → `"compliant":null` for every key, never `false` | P | 0 | F2=json, F7=settings corrupted | [read_status_test.rs] |
| IT-23 | `.status v::2` settings.json permission-denied (mode 000), versions dir genuinely locked (555) → all 6 rows `UNVERIFIABLE`, no false mismatch | P | 0 | F7=settings permission-denied | [read_status_test.rs] |
| IT-24 | `.status v::2` pinned, `autoUpdatesChannel` drifted → mismatch flagged | P | 0 | F7=autoUpdatesChannel drifted | [read_status_test.rs] |
| IT-25 | `.status v::2` pinned, `minimumVersion` drifted → mismatch flagged | P | 0 | F7=minimumVersion drifted | [read_status_test.rs] |
| IT-26 | `.status v::2` pinned, `env.DISABLE_AUTOUPDATER` drifted → mismatch flagged | P | 0 | F7=env.DISABLE_AUTOUPDATER drifted | [read_status_test.rs] |
| IT-27 | `.status v::2` pinned, `env.DISABLE_UPDATES` drifted → mismatch flagged | P | 0 | F7=env.DISABLE_UPDATES drifted | [read_status_test.rs] |
| IT-28 | `.status v::2` install interrupted after preference stored but before mechanism applied → all 6 rows report TRUE mismatch, none `UNVERIFIABLE` | P | 0 | F7=install interrupted | [read_status_test.rs] |
| IT-29 | `.status v::2` unpinned, `autoUpdates` explicit `"true"` → compliant, no mismatch | P | 0 | F7=autoUpdates unpinned explicit true | [read_status_test.rs] |
| IT-30 | `.status v::2` unpinned, `autoUpdates` drifted to `"false"` → mismatch flagged | P | 0 | F7=autoUpdates unpinned drifted | [read_status_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-10 | `.status format::xml` → exit 1 | N | 1 | F2=xml | new |
| IT-11 | `.status v::3` → exit 1, out of range | N | 1 | F1=3 | [read_status_test.rs] |
| IT-12 | `.status bogus::x` → exit 1 | N | 1 | F6=present | new |
| IT-19 | `.status v::3` → exit 1 (Task 314 regression check; same requirement as IT-11, verified independently as part of the Lock-state visibility feature's no-regression scope) | N | 1 | F1=3 | [read_status_test.rs] |

### Summary

- **Total:** 30 tests (26 positive, 4 negative)
- **Negative ratio:** 13.3% (command-specific only) — below ≥40% threshold; 4 additional cross-cutting tests in `read_status_test.rs` also apply to `.status` among other commands: 3 negative (`tc242_unknown_format_exits_1`, `tc243_uppercase_format_exits_1`, `tc244_empty_format_exits_1`) and 1 positive (`tc245_last_occurrence_wins_for_verbosity` — exit 0, verifies last-`v::`-wins precedence, not an error case)
- **Existing cross-cutting negatives applying to `.status`:** `tc242` (`format::xml`), `tc243` (`format::JSON`), `tc244` (`format::`)
- **Combined negative count (command-specific + cross-cutting):** 7/34 = 20.6% ❌ (below ≥40% threshold; informational metric only, not a blocking gate for this spec)
- **TC range:** IT-1 to IT-30

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (always — .status never errors) | IT-1 through IT-9, IT-13 through IT-18, IT-20 through IT-30 |
| 1 | Invalid arguments | IT-10 through IT-12, IT-19 |
| 2 | Not applicable (.status always exits 0 for any valid state) | — |

### Degradation Semantics

`.status` exhibits unique behavior: it always exits 0 regardless of environment state.
Missing claude, missing HOME, or missing accounts produce informational "not found"/"unknown" output
rather than exit 2. This is by design (FR-01: status is read-only, never fails). The same principle
extends to the `Lock:` section: a `settings.json` that could not be read — malformed JSON or a
permission error alike — degrades every row to `UNVERIFIABLE` rather than propagating an error or
guessing at a possibly-wrong compliance verdict (IT-21 through IT-23).

### Factor Coverage

| Factor | Positive Coverage | Negative Coverage |
|--------|-------------------|-------------------|
| F1 (v::) | IT-3 (v=0), IT-4 (v=1), IT-1 (absent), IT-17 (v::0/1 unchanged) | IT-11, IT-19 (v::3) |
| F2 (format) | IT-5 (json) | IT-10 (xml) |
| F3 (PATH) | IT-1 (found), IT-2 (empty) | — |
| F4 (HOME) | IT-1 (set), IT-7 (empty) | — |
| F5 (preference) | IT-8 (absent), IT-9 (set) | — |
| F6 (unknown params) | — | IT-12 |
| F7 (lock-state) | IT-13 (pinned compliant), IT-14 (chmod drift), IT-15 (autoUpdates drift), IT-16 (unpinned compliant), IT-18 (json), IT-20 (dir absent), IT-21 (settings corrupted), IT-22 (settings corrupted, json), IT-23 (settings permission-denied), IT-24 (autoUpdatesChannel drift), IT-25 (minimumVersion drift), IT-26 (env.DISABLE_AUTOUPDATER drift), IT-27 (env.DISABLE_UPDATES drift), IT-28 (install interrupted), IT-29 (autoUpdates unpinned explicit true), IT-30 (autoUpdates unpinned drift) | — |

---

## Test Case Details

---

### IT-1: `.status` exits 0 always

- **Given:** clean environment
- **When:** `clv .status`
- **Then:** exit 0; output contains version, processes, and account information

---

### IT-2: Empty PATH → "not found", exits 0

- **Given:** `PATH=""`, `HOME=<tmp>`.
- **When:** `clv .status`
- **Then:** exit 0; output contains "not found" or "unknown"

---

### IT-3: `v::0` → exactly 3 bare lines

- **Given:** `HOME=<tmp>` with empty settings (no preference stored).
- **When:** `clv .status v::0`
- **Then:** exit 0; exactly 3 non-empty lines in stdout

---

### IT-4: `v::1` → labeled lines

- **Given:** clean environment
- **When:** `clv .status v::1`
- **Then:** exit 0; output contains "Version:", "Processes:", "Account:" labels

---

### IT-5: `format::json` → valid JSON

- **Given:** clean environment
- **When:** `clv .status format::json`
- **Then:** exit 0; valid JSON object with `version`, `processes`, `account` keys

---

### IT-6: `v::0` has ≤ lines than `v::1`

- **Given:** clean environment
- **When:** `clv .status v::0` and `clv .status v::1`
- **Then:** exit 0 for both; line count of v::0 output ≤ line count of v::1 output

---

### IT-7: HOME not set → "unknown" account, no crash

- **Given:** `HOME=""`.
- **When:** `clv .status`
- **Then:** exit 0; stdout contains "unknown"

---

### IT-8: No preference → no "Preferred" line

- **Given:** `HOME=<tmp>`; `settings.json` has no `preferredVersionSpec`.
- **When:** `clv .status`
- **Then:** exit 0; output does not contain "Preferred"

---

### IT-9: With preference → shows "Preferred" line

- **Given:** `HOME=<tmp>`; `settings.json` has `preferredVersionSpec = "stable"`.
- **When:** `clv .status`
- **Then:** exit 0; output contains "Preferred"

---

### IT-10: `format::xml` → exit 1

- **Given:** clean environment
- **When:** `clv .status format::xml`
- **Then:** exit 1; stderr mentions unknown format

---

### IT-11: `v::3` → exit 1, out of range

- **Given:** clean environment
- **When:** `clv .status v::3`
- **Then:** exit 1; out-of-range verbosity rejected

---

### IT-12: `bogus::x` → exit 1

- **Given:** clean environment
- **When:** `clv .status bogus::x`
- **Then:** exit 1; stderr mentions unknown parameter

---

### IT-13: Pinned, all compliant → `Lock:` section, no mismatch

- **Given:** `HOME=<tmp>`; `settings.json` has all 5 pin keys (`preferredVersionSpec`, `preferredVersionResolved`, `autoUpdates=false`, `autoUpdatesChannel=stable`, `minimumVersion`, nested `env.DISABLE_AUTOUPDATER=1`/`env.DISABLE_UPDATES=1`); versions dir `chmod 555`.
- **When:** `clv .status v::2`
- **Then:** exit 0; output contains `Lock:` and all 6 keys (5 settings + `chmod`); no `MISMATCH` marker

---

### IT-14: Pinned, `chmod` drifted → mismatch flagged

- **Given:** Same as IT-13 but versions dir is `chmod 755` (should be `555`).
- **When:** `clv .status v::2`
- **Then:** exit 0; the `chmod` row shows actual `755`, expected `555`, and `MISMATCH`

---

### IT-15: Pinned, `autoUpdates` drifted → mismatch flagged

- **Given:** Same as IT-13 but `autoUpdates=true` (should be `false`).
- **When:** `clv .status v::2`
- **Then:** exit 0; the `autoUpdates` row shows `MISMATCH`

---

### IT-16: Unpinned, compliant → `Lock:` section, no mismatch

- **Given:** `HOME=<tmp>` with empty settings (no preference stored); versions dir `chmod 755`.
- **When:** `clv .status v::2`
- **Then:** exit 0; output contains `Lock:`; no `MISMATCH` marker

---

### IT-17: `v::0`/`v::1` output unchanged by the Lock: feature

- **Given:** `HOME=<tmp>` with empty settings.
- **When:** `clv .status v::0` and `clv .status v::1`
- **Then:** exit 0 for both; neither contains `Lock:`; `v::0` still exactly 3 lines; `v::1` still shows `Version:`/`Processes:`/`Account:` labels

---

### IT-18: `format::json` pinned, compliant → `"lock"` object present

- **Given:** Same fixture as IT-13.
- **When:** `clv .status format::json`
- **Then:** exit 0; JSON contains a `"lock"` object with all 6 keys, each entry showing `"compliant":true`

---

### IT-19: `v::3` → exit 1 (Task 314 regression check)

Same requirement as IT-11 (F1's out-of-range boundary), verified independently as part of Task 314's own no-regression scope for the Lock-state visibility feature — the Lock: section is only reachable at `v::2`+, so Task 314 needed its own confirmation that `v::3` still rejects rather than silently falling into the same catch-all branch as `v::2`.

- **Given:** clean environment
- **When:** `clv .status v::3`
- **Then:** exit 1; out-of-range verbosity rejected

---

### IT-20: Unpinned, versions dir absent (fresh install) → no false mismatch

MAAV-found regression case: a genuinely fresh install (nothing ever run through `.version.install`) has no versions directory at all. The `chmod` row's underlying `VersionsDirLockMode::Absent` must be treated as "no reliable signal," not compared against `"755"`/`"555"` as if it were a real-but-wrong value — otherwise every fresh install falsely shows a `chmod MISMATCH`, undermining the entire purpose of the `Lock:` section.

- **Given:** `HOME=<tmp>` with empty settings (no preference stored); the versions directory is never created.
- **When:** `clv .status v::2`
- **Then:** exit 0; the `chmod` row shows `absent (expected: 755)` with no `MISMATCH` marker; no `MISMATCH` appears anywhere in the output

---

### IT-21: settings.json corrupted, versions dir genuinely locked → all rows `UNVERIFIABLE`

MAAV-found regression case (independently rediscovered by two adversarial agents in the same
validation round): `read_preferred_version` silently degrades to "no preference" (`is_pinned =
false`) when `settings.json` cannot be read, via `.ok()` on the underlying read error. Without this
fix, a genuinely-pinned-and-locked install (`chmod 555`) would be compared against the *unpinned*
(`755`) expectation and misreport a false `MISMATCH` on `chmod` — and a false `Compliant` verdict on
the 5 settings-derived rows, since their `None` actual values would coincidentally match the
unpinned expectation. Every row must instead report `UNVERIFIABLE`, with an explanatory note.

- **Given:** `HOME=<tmp>`; `settings.json` contains invalid JSON (`{ not valid json`); versions dir `chmod 555` (genuinely locked).
- **When:** `clv .status v::2`
- **Then:** exit 0; output contains `Lock:` and an explanatory note ("could not be read"); all 6 rows show `UNVERIFIABLE`; no `MISMATCH` appears anywhere

---

### IT-22: `format::json` variant — `"compliant":null` for every key, never `false`

Verifies each of the 6 keys' own JSON entry individually (not merely "somewhere in the blob"), since
a whole-text substring check could pass even if only one of six rows correctly showed `null` while
the rest regressed to `true`/`false`.

- **Given:** Same fixture as IT-21.
- **When:** `clv .status format::json`
- **Then:** exit 0; JSON contains a `"lock"` object; every one of the 6 keys' own entry shows `"compliant":null`

---

### IT-23: settings.json permission-denied (mode 000), versions dir genuinely locked → all rows `UNVERIFIABLE`

MAAV-found regression case, distinct from IT-21: `read_preferred_version` swallows *any* read error
via `.ok()`, not just a JSON parse failure — so a fix that only special-cased
`ErrorKind::InvalidData` would still misreport a false `MISMATCH` when `settings.json` is unreadable
for a different reason (e.g. a permission/ownership/ACL change instead of malformed content). The
detection was broadened to treat any error other than `ErrorKind::NotFound` (file genuinely absent —
a normal, non-corrupted state) as untrustworthy.

- **Given:** `HOME=<tmp>`; `settings.json` is valid JSON, genuinely pinned; file mode `000` (unreadable); versions dir `chmod 555` (genuinely locked).
- **When:** `clv .status v::2`
- **Then:** exit 0; all 6 rows show `UNVERIFIABLE`; no `MISMATCH` appears anywhere

---

### IT-24: `autoUpdatesChannel` drifted → mismatch flagged

MAAV-found coverage gap: prior to this test, `write_pinned_settings` hardcoded `autoUpdatesChannel` to
the always-correct `"stable"`, so no test could distinguish a working comparison from a hardcoded-Compliant
bug in this row specifically (a defect here would have been silently invisible to the whole suite).

- **Given:** `HOME=<tmp>`; pinned install; `autoUpdatesChannel` is `"beta"` instead of the pinned `"stable"`; all other keys compliant.
- **When:** `clv .status v::2`
- **Then:** exit 0; `autoUpdatesChannel` line shows `MISMATCH`

---

### IT-25: `minimumVersion` drifted → mismatch flagged

Same MAAV-found coverage gap as IT-24, for `minimumVersion`.

- **Given:** `HOME=<tmp>`; pinned install; `minimumVersion` is `"2.1.70"` instead of the pinned resolved version `"2.1.78"`; all other keys compliant.
- **When:** `clv .status v::2`
- **Then:** exit 0; `minimumVersion` line shows `MISMATCH`

---

### IT-26: `env.DISABLE_AUTOUPDATER` drifted → mismatch flagged

Same MAAV-found coverage gap as IT-24, for `env.DISABLE_AUTOUPDATER`.

- **Given:** `HOME=<tmp>`; pinned install; `env.DISABLE_AUTOUPDATER` is `"0"` instead of the pinned `"1"`; all other keys compliant.
- **When:** `clv .status v::2`
- **Then:** exit 0; `env.DISABLE_AUTOUPDATER` line shows `MISMATCH`

---

### IT-27: `env.DISABLE_UPDATES` drifted → mismatch flagged

Same MAAV-found coverage gap as IT-24, for `env.DISABLE_UPDATES`.

- **Given:** `HOME=<tmp>`; pinned install; `env.DISABLE_UPDATES` is `"0"` instead of the pinned `"1"`; all other keys compliant.
- **When:** `clv .status v::2`
- **Then:** exit 0; `env.DISABLE_UPDATES` line shows `MISMATCH`

---

### IT-28: Install interrupted after preference stored but before mechanism applied → all rows report a TRUE mismatch

MAAV Fresh Challenger finding (Round 4, B1): `version_install_routine` previously called `perform_install()`
(which ends by applying the lock mechanism — `autoUpdates`/`autoUpdatesChannel`/`minimumVersion`/the two
`env.DISABLE_*` keys/`chmod`) *before* `store_preferred_version()` recorded the pin intent. A crash or kill
in the window between those two calls left the mechanism genuinely applied but `preferredVersionSpec`
unset, so `is_pinned` read `false` and `.status` compared the (now-locked) actual state against the
*unpinned* expectation — reporting a false `MISMATCH` on all 6 rows despite the install having actually
succeeded. The fix reorders the two calls so the preference is recorded first; this test instead reproduces
the *other* half of the interrupted window under the corrected order — preference recorded, mechanism not
yet applied — and confirms that state now correctly reports a **true** mismatch (the user's recorded intent
genuinely isn't enforced yet), not a false one, and never `UNVERIFIABLE` (settings.json is fully valid and
readable throughout). See `Fix(MAAV-found, Task 314 Round 4 Fresh Challenger)` in `commands/version.rs`.

**Scope note (MAAV Round 5, A9):** this test proves only that `status.rs` classifies the given fixture
correctly — the identical fixture is independently reachable via the unrelated idempotency-skip branch in
`version_install_routine` regardless of whether the reorder fix is present, so it does not by itself prove
the fix's call order actually executes. `IT-26` in `04_version_install.md` (`tc533`) supplies that missing
evidence by invoking the real `.version.install` command and observing the reordered write path directly.

- **Given:** `HOME=<tmp>`; `settings.json` contains only `preferredVersionSpec`/`preferredVersionResolved` (valid JSON, fully readable); versions dir `chmod 755` (mechanism not yet applied — still at the unpinned default).
- **When:** `clv .status v::2`
- **Then:** exit 0; no row shows `UNVERIFIABLE`; all 6 rows (`autoUpdates`, `autoUpdatesChannel`, `minimumVersion`, `env.DISABLE_AUTOUPDATER`, `env.DISABLE_UPDATES`, `chmod`) show `MISMATCH`

---

### Source Functions

| Function | File |
|----------|------|
| `tc099_status_exits_0` | `tests/cli/read_status_test.rs` |
| `tc096_status_no_claude_in_path_exits_0` | `tests/cli/read_status_test.rs` |
| `tc097_status_v0_has_3_lines` | `tests/cli/read_status_test.rs` |
| `tc098_status_v1_has_labels` | `tests/cli/read_status_test.rs` |
| `tc100_status_format_json` | `tests/cli/read_status_test.rs` |
| `tc104_status_v0_fewer_lines_than_v1` | `tests/cli/read_status_test.rs` |
| `tc105_status_no_home_shows_unknown_account` | `tests/cli/read_status_test.rs` |
| `tc419_status_no_preference_no_preferred_line` | `tests/cli/read_status_test.rs` |
| `tc420_status_with_preference_shows_preferred` | `tests/cli/read_status_test.rs` |
| `tc255_status_v0_fewer_lines_than_v1` | `tests/cli/cross_cutting_test.rs` |
| `tc515_status_lock_pinned_all_compliant` | `tests/cli/read_status_test.rs` |
| `tc516_status_lock_chmod_drift_flagged` | `tests/cli/read_status_test.rs` |
| `tc517_status_lock_autoupdates_drift_flagged` | `tests/cli/read_status_test.rs` |
| `tc518_status_lock_unpinned_all_compliant` | `tests/cli/read_status_test.rs` |
| `tc519_status_v0_v1_unchanged_by_lock_feature` | `tests/cli/read_status_test.rs` |
| `tc520_status_v3_out_of_range_exits_1` | `tests/cli/read_status_test.rs` |
| `tc521_status_lock_json_object_present` | `tests/cli/read_status_test.rs` |
| `tc522_status_lock_chmod_absent_dir_not_flagged` | `tests/cli/read_status_test.rs` |
| `tc523_status_lock_corrupted_settings_reports_unverifiable` | `tests/cli/read_status_test.rs` |
| `tc524_status_lock_json_corrupted_settings_compliant_null` | `tests/cli/read_status_test.rs` |
| `tc525_status_lock_unreadable_settings_permission_denied_reports_unverifiable` | `tests/cli/read_status_test.rs` |
| `tc526_status_lock_autoupdates_channel_drift_flagged` | `tests/cli/read_status_test.rs` |
| `tc527_status_lock_minimum_version_drift_flagged` | `tests/cli/read_status_test.rs` |
| `tc528_status_lock_disable_autoupdater_drift_flagged` | `tests/cli/read_status_test.rs` |
| `tc529_status_lock_disable_updates_drift_flagged` | `tests/cli/read_status_test.rs` |
| `tc530_status_lock_interrupted_install_reports_true_mismatch` | `tests/cli/read_status_test.rs` |
| `tc242_unknown_format_exits_1` | `tests/cli/read_status_test.rs` |
| `tc243_uppercase_format_exits_1` | `tests/cli/read_status_test.rs` |
| `tc244_empty_format_exits_1` | `tests/cli/read_status_test.rs` |
| `tc245_last_occurrence_wins_for_verbosity` | `tests/cli/read_status_test.rs` |
