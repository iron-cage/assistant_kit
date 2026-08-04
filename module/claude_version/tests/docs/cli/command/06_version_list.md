# Test: `.version.list`

### Scope

- **Purpose**: Integration test cases for the `.version.list` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for alias listing (`mode::aliases`, default) and release-history listing (`mode::history`).
- **In Scope**: `mode::` switch, alias resolution, release-history retrieval, `count::` interaction, verbosity levels, output formats, network/HOME environmental failures.
- **Out of Scope**: Parameter edge cases (→ `../param/`), group interactions (→ `../param_group/`).

Integration test planning for the `.version.list` command. See [command/version.md](../../../../docs/cli/command/version.md) for specification. Absorbs the former `.version.history` command's test surface (`12_version_history.md`, deleted) since that command was merged into `.version.list` as `mode::history` — see `docs/cli/command/version.md` Command :: 12 retirement note.

## Test Factor Analysis

### Factor 1: `mode::` (String, optional, default "aliases")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default value `aliases`, compile-time alias table | Default behavior |
| `aliases` | Explicit alias listing (same as absent) | Explicit valid |
| `history` | Release-history listing from GitHub Releases API | Alternate valid |
| `bogus` | Unrecognized value | Invalid: exit 1 |
| `` (empty) | Empty string value | Invalid: exit 1 |
| `Aliases` / `History` | Wrong case (case-sensitive) | Invalid: exit 1 |

### Factor 2: `count::` (Integer, optional, default 10)

Meaningful only under `mode::history`; accepted but has no effect under `mode::aliases`.

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default value 10 (`mode::history` only) | Default behavior |
| 0 | Empty output, no entries | Boundary: minimum |
| 1 | Single entry | Boundary: min useful |
| 3 | Small subset | Nominal |
| 100 | Exceeds available releases | Boundary: max (capped by API data) |
| -1 | Negative integer | Invalid: exit 1 (u64 adapter parse failure) |
| `abc` | Non-integer string | Invalid: type mismatch |

Boundary set: 0, 1, 10 (default), 66 (current release count), 100 (API limit).

### Factor 3: `v::` / verbosity under `mode::aliases` (Integer, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default value 1, alias names with descriptions | Default behavior |
| 0 | Names only, no descriptions | Minimum output |
| 1 | Names + pinned semver in parens + description | Nominal |
| 3 | Out-of-range integer | Invalid: exit 1 |

### Factor 4: `v::` / verbosity under `mode::history` (Integer, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default value 1, labeled output | Default behavior |
| 0 | Bare: `{version}  {date}` only | Minimum output |
| 1 | Version + date + summary per line | Nominal |
| 2 | Full changelog with `##` headers | Maximum detail |
| `abc` | Non-integer string | Invalid: type mismatch |

### Factor 5: `format::` (String, optional, default "text")

Same two valid values and validation rules apply under both modes; JSON field shapes differ per mode (see Coverage Verification).

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default text output | Default behavior |
| `text` | Explicit text output | Explicit valid |
| `json` | JSON array (alias objects under `mode::aliases`; release objects under `mode::history`) | Alternate valid |
| `xml` | Unrecognized value | Invalid: exit 1 |
| `JSON` | Wrong case (case-sensitive) | Invalid: exit 1 |
| (empty) | Empty string value | Invalid: exit 1 |

### Factor 6: Network availability (Environmental, `mode::history` only)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| available | curl succeeds, data returned | Happy path |
| unavailable | curl fails, no data | Fallback: exit 0 (compiled-in snapshot, stderr advisory) |

### Factor 7: HOME environment (Environmental, `mode::history` only)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| set | Normal HOME, cache path accessible | Happy path |
| empty | HOME unset or empty | Failure: exit 2 |

### Factor 8: Unknown parameters

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| none | No unknown params | Happy path |
| present | e.g. `bogus::x` | Invalid: exit 1 |

### Factor 9: Parameter syntax

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| correct | `param::value` style | Happy path |
| flag | `--param` style | Invalid: exit 1 |

**Note:** Under `mode::aliases` (default), `.version.list` reads only compile-time constants — no external dependencies, every invocation exits 0 in a valid environment, no runtime failures exist. Under `mode::history`, `$HOME` (Factor 7) is the only remaining exit-2 condition; network unavailability (Factor 6) no longer fails — it falls back to a compiled-in snapshot with a stderr advisory.

---

## Test Matrix

### Positive Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-1 | `.version.list` exits 0 (`mode::aliases` implicit) | P | 0 | F1=absent, F3=absent | [read_version_test.rs] |
| IT-9 | Output includes "stable" alias | P | 0 | F1=absent | [read_version_test.rs] |
| IT-10 | Output includes "latest" alias | P | 0 | F1=absent | [read_version_test.rs] |
| IT-2 | `v::0` → names only, no descriptions | P | 0 | F3=0 | [read_version_test.rs] |
| IT-11 | `v::1` → aliases with descriptions | P | 0 | F3=1 | [read_version_test.rs] |
| IT-3 | Output is deterministic on two calls | P | 0 | F1=absent | [read_version_test.rs] |
| IT-12 | `format::json` → valid JSON array or object | P | 0 | F5=json | [read_version_test.rs] |
| IT-13 | Output includes "month" alias | P | 0 | F1=absent | [read_version_test.rs] |
| IT-14 | `v::1` shows pinned versions in parens `(vX.Y.Z)` | P | 0 | F3=1 | [read_version_test.rs] |
| IT-15 | `format::json` has `"value"` field | P | 0 | F5=json | [read_version_test.rs] |
| IT-7 | `format::json` → valid JSON output | P | 0 | F5=json | [read_version_test.rs] |
| IT-8 | Output is stable across repeated invocations | P | 0 | F1=absent | [read_version_test.rs] |
| IT-16 | `mode::history` defaults → exits 0 (live fetch or compiled-in fallback) | P | 0 | F1=history, F2=absent, F4=absent, F5=absent, F6=any | [read_version_test.rs] |
| IT-17 | `mode::history count::3` → output has ≤3 version entries | P | 0 | F1=history, F2=3 | [read_version_test.rs] |
| IT-18 | `mode::history count::0` → empty output, no version lines | P | 0 | F1=history, F2=0 | [read_version_test.rs] |
| IT-19 | `mode::history v::0` → bare `{version}  {date}` lines, no summaries | P | 0 | F1=history, F4=0 | [read_version_test.rs] |
| IT-20 | `mode::history v::1` explicit → version + date + summary per line | P | 0 | F1=history, F4=1 | [read_version_test.rs] |
| IT-21 | `mode::history v::2` → full changelog with `##` headers per version | P | 0 | F1=history, F4=2 | [read_version_test.rs] |
| IT-22 | `mode::history format::json` → valid JSON array with version/date/summary fields | P | 0 | F1=history, F5=json | [read_version_test.rs] |
| IT-23 | `mode::history count::1 format::json` → JSON array with exactly 1 element | P | 0 | F1=history, F2=1, F5=json | [read_version_test.rs] |
| IT-24 | `mode::history count::1 v::0` → exactly 1 bare line | P | 0 | F1=history, F2=1, F4=0 | [read_version_test.rs] |
| IT-25 | `mode::history count::1 v::2` → single full changelog block | P | 0 | F1=history, F2=1, F4=2 | [read_version_test.rs] |
| IT-26 | `mode::history` default count ≤10 entries (verify default value) | P | 0 | F1=history, F2=absent | [read_version_test.rs] |
| IT-27 | `mode::history count::100` → all available releases, capped by data | P | 0 | F1=history, F2=100 | [read_version_test.rs] |
| IT-28 | Idempotency: two consecutive `mode::history` calls produce identical output | P | 0 | F1=history, F2=1 | [read_version_test.rs] |
| IT-29 | Param order: `mode::history count::3 v::0` = `v::0 mode::history count::3` | P | 0 | F1=history, F2=3, F4=0 | [read_version_test.rs] |
| IT-30 | `mode::history count::0 format::json` → empty JSON array `[]` | P | 0 | F1=history, F2=0, F5=json | [read_version_test.rs] |
| IT-41 | UTF-8 non-ASCII chars in release body preserved intact (`mode::history`) | P | 0 | F1=history, F5=absent, F4=2 | [read_version_test.rs] |
| IT-42 | `mode::aliases` explicit == default (absent) output, byte-identical | P | 0 | F1=aliases | [read_version_test.rs] |
| IT-46 | `count::` accepted under `mode::aliases`, has no effect on output | P | 0 | F1=aliases, F2=5 | [read_version_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-4 | `bogus::x` → exit 1, unknown parameter | N | 1 | F8=present | [read_version_test.rs] |
| IT-5 | `format::xml` → exit 1, unknown format | N | 1 | F5=xml | [read_version_test.rs] |
| IT-6 | `v::3` → exit 1, out of range (`mode::aliases`) | N | 1 | F3=3 | [read_version_test.rs] |
| IT-31 | `mode::history format::xml` → exit 1, unknown format | N | 1 | F1=history, F5=xml | [read_version_test.rs] |
| IT-32 | `mode::history format::JSON` (uppercase) → exit 1, case-sensitive | N | 1 | F1=history, F5=JSON | [read_version_test.rs] |
| IT-33 | `mode::history format::` (empty value) → exit 1 | N | 1 | F1=history, F5="" | [read_version_test.rs] |
| IT-34 | `mode::history` unknown param `bogus::x` → exit 1 | N | 1 | F1=history, F8=present | [read_version_test.rs] |
| IT-35 | `mode::history`, network unavailable → exit 0 via compiled-in fallback, stderr carries advisory | P | 0 | F1=history, F6=unavailable | manual |
| IT-36 | `mode::history`, HOME empty → exit 2 | N | 2 | F1=history, F7=empty | [read_version_test.rs] |
| IT-37 | `mode::history count::-1` (negative) → parse error → exit 1 | N | 1 | F1=history, F2=-1 | [read_version_test.rs] |
| IT-38 | `mode::history v::abc` → exit 1, type mismatch for Integer | N | 1 | F1=history, F4=invalid | [read_version_test.rs] |
| IT-39 | `mode::history count::abc` → exit 1, type mismatch for Integer | N | 1 | F1=history, F2=invalid | [read_version_test.rs] |
| IT-40 | `mode::history --verbose` flag-style → exit 1 | N | 1 | F1=history, F9=flag-style | [read_version_test.rs] |
| IT-43 | `mode::bogus` → exit 1, unrecognized mode | N | 1 | F1=bogus | [read_version_test.rs] |
| IT-44 | `mode::` (empty value) → exit 1 | N | 1 | F1="" | [read_version_test.rs] |
| IT-45 | `mode::History` (wrong case) → exit 1, case-sensitive | N | 1 | F1=History | [read_version_test.rs] |

### Summary

- **Total:** 46 tests (30 positive, 16 negative)
- **Negative ratio:** 34.8% command-local; supplemented by cross-cutting format/verbosity edge cases in `read_status_test.rs` and `tests/docs/cli/param/05_format.md` ✅
- **IT range:** IT-1 to IT-46 (IT-1–IT-15: `mode::aliases`, unchanged from pre-merge; IT-16–IT-41: absorbed from the retired `12_version_history.md`, renumbered, each command literal now carries explicit `mode::history`; IT-42–IT-46: new, covering the `mode::` switch itself and its `count::` interaction)

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success — `mode::aliases` always; `mode::history` with or without output (live fetch or compiled-in fallback) | IT-1 through IT-3, IT-7 through IT-30, IT-35, IT-41, IT-42, IT-46 |
| 1 | Invalid arguments (mode, format, type, unknown param, syntax, out-of-range) | IT-4 through IT-6, IT-31 through IT-34, IT-37 through IT-40, IT-43 through IT-45 |
| 2 | Operational failure (HOME unset) — `mode::history` only | IT-36 |

### Alias Completeness (`mode::aliases`)

All three aliases must appear in output: `stable` (IT-9), `latest` (IT-10), `month` (IT-13).
Pinned values for `stable` and `month` must appear in `v::1` output (IT-14).

### JSON Field Requirements

`format::json` under `mode::aliases` must include at minimum: `"name"` or `"alias"`, and `"value"` (pinned semver or null). IT-12 verifies array structure. IT-15 verifies `"value"` field presence.

`format::json` under `mode::history` must include at minimum: `version`, `date`, `summary` fields per element. IT-22 verifies array structure with required fields.

### Empty vs Error Distinction (`mode::history`)

| State | Exit | Output | Tests |
|-------|------|--------|-------|
| Valid non-empty | 0 | Version entries on stdout | IT-16, IT-17, IT-20, etc. |
| Valid empty | 0 | Empty stdout (text) or `[]` (json) | IT-18 (text), IT-30 (json) |
| Fallback (network down) | 0 | Compiled-in snapshot on stdout, advisory warning on stderr | IT-35 |
| Error | 2 | Error message on stderr | IT-36 |

### Pairwise Coverage: `count::` x `v::` (`mode::history`)

| count \ v | absent | 0 | 1 | 2 |
|-----------|--------|---|---|---|
| absent | IT-16 | IT-19 | IT-20 | IT-21 |
| 0 | IT-18 | pruned | pruned | pruned |
| 1 | — | IT-24 | (IT-23 implicit) | IT-25 |
| 3 | IT-17 | IT-29 | — | — |
| 100 | IT-27 | pruned | pruned | pruned |
| -1 | IT-37 (exit 1) | — | — | — |

**Pruned with justification:**
- (0, 0/1/2): `count::0` produces zero entries; verbosity formats entries but there are none. Output is always empty regardless of verbosity.
- (100, 0/1/2): `count::100` exercises the data-volume boundary; verbosity formatting is an independent dimension already covered by (absent, 0/1/2).
- (-1, v/format): IT-37 exits 1 at adapter parse (u64 rejects negative); no handler output to cover.

### Pairwise Coverage: `count::` x `format::` (`mode::history`)

| count \ format | absent | json |
|----------------|--------|------|
| absent | IT-16 | IT-22 |
| 0 | IT-18 | IT-30 |
| 1 | — | IT-23 |
| 3 | IT-17 | pruned |
| 100 | IT-27 | pruned |

**Pruned:** (3, json), (100, json) — JSON array structure independent of entry count; covered by (absent, json) + (1, json) + (0, json).

### Pairwise Coverage: `v::` x `format::` (`mode::history`)

| v \ format | absent | json |
|------------|--------|------|
| absent | IT-16 | IT-22 |
| 0 | IT-19 | pruned |
| 1 | IT-20 | pruned |
| 2 | IT-21 | pruned |

**Pruned:** (0/1/2, json) — `format::json` produces identical JSON array regardless of verbosity level. JSON output ignores verbosity by design (consistent with `mode::aliases` sibling behavior).

### Error Path Completeness (`mode::history`)

| Error Source | Error Message Pattern | Exit | Test(s) |
|-------------|----------------------|------|---------|
| `OutputOptions::from_cmd` | "unknown format '{other}': expected text or json" | 1 | IT-31, IT-32, IT-33 |
| `require_claude_paths` | "HOME environment variable not set" | 2 | IT-36 |
| Unilang adapter | Type mismatch for Integer param | 1 | IT-37, IT-38, IT-39 |
| Unilang adapter | Unknown parameter rejected | 1 | IT-34 |
| Unilang adapter | Flag-style syntax rejected | 1 | IT-40 |
| `ListMode` adapter | "unknown mode '{raw}': expected aliases or history" | 1 | IT-43, IT-45 |

### Mode Switch Coverage

| Mode value | Behavior | Tests |
|------------|----------|-------|
| absent | Defaults to `aliases` | IT-1 |
| `aliases` | Explicit, byte-identical to absent | IT-42 |
| `history` | Release-history listing | IT-16 through IT-41 |
| `bogus` / empty / wrong-case | Rejected, exit 1 | IT-43, IT-44, IT-45 |

`count::` cross-mode interaction: accepted under both modes (never an "unknown parameter" error), but only affects output under `mode::history` (IT-46 confirms no effect under `mode::aliases`).

---

## Test Case Details

---

### IT-1: `.version.list` exits 0

- **Given:** clean environment
- **When:** `clv .version.list`
- **Then:** exit 0; stdout contains 3 alias lines (`stable`, `latest`, `month`)
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-2: `v::0` → names only

- **Given:** clean environment
- **When:** `clv .version.list v::0`
- **Then:** exit 0; each output line contains only an alias name; no ` — ` description separator present
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-3: Deterministic output on two calls

- **Given:** clean environment
- **When:** `clv .version.list` (run twice in succession)
- **Then:** both stdout captures are byte-identical; output order and content do not change between runs
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-4: `bogus::x` → exit 1

- **Given:** clean environment
- **When:** `clv .version.list bogus::x`
- **Then:** exit 1; stderr or stdout mentions unknown parameter
- **Exit:** 1
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-5: `format::xml` → exit 1

- **Given:** clean environment
- **When:** `clv .version.list format::xml`
- **Then:** exit 1; error message references format or valid values
- **Exit:** 1
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-6: `v::3` → exit 1

- **Given:** clean environment
- **When:** `clv .version.list v::3`
- **Then:** exit 1; error references out-of-range verbosity value
- **Exit:** 1
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-7: `format::json` → valid JSON output

- **Given:** clean environment
- **When:** `clv .version.list format::json`
- **Then:** stdout is valid JSON containing version alias entries
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-8: Output is stable across repeated invocations

- **Given:** clean environment
- **When:** `clv .version.list` (run 3 times)
- **Then:** All 3 stdout captures are byte-identical
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-9: Output includes "stable" alias

- **Given:** clean environment
- **When:** `clv .version.list`
- **Then:** exit 0; stdout contains the string `stable`
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-10: Output includes "latest" alias

- **Given:** clean environment
- **When:** `clv .version.list`
- **Then:** exit 0; stdout contains the string `latest`
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-11: `v::1` → aliases with descriptions

- **Given:** clean environment
- **When:** `clv .version.list v::1`
- **Then:** exit 0; each alias line includes a description separator (` — ` or equivalent); at least one non-empty description present
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-12: `format::json` → valid JSON array or object

- **Given:** clean environment
- **When:** `clv .version.list format::json`
- **Then:** exit 0; stdout is valid JSON (parseable); top-level structure is an array or object containing alias entries
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-13: Output includes "month" alias

- **Given:** clean environment
- **When:** `clv .version.list`
- **Then:** exit 0; stdout contains the string `month`
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-14: `v::1` shows pinned versions in parens `(vX.Y.Z)`

- **Given:** clean environment; `stable` and `month` aliases have pinned semver values
- **When:** `clv .version.list v::1`
- **Then:** exit 0; output for `stable` and/or `month` contains a parenthesized version string matching `(v\d+\.\d+\.\d+)`
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-15: `format::json` has `"value"` field

- **Given:** clean environment
- **When:** `clv .version.list format::json`
- **Then:** exit 0; parsed JSON contains at least one entry with a `"value"` key (pinned semver string or null)
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-16: `mode::history` default invocation exits 0

- **Given:** HOME set (network state irrelevant — falls back to the compiled-in snapshot if the live fetch and cache both fail).
- **When:** `clv .version.list mode::history`
- **Then:** exit 0 with version history on stdout, from the live GitHub Releases API when reachable or the compiled-in `VERSION_HISTORY` snapshot otherwise
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-17: `mode::history count::3` limits output to 3 entries

- **Given:** Network available.
- **When:** `clv .version.list mode::history count::3`
- **Then:** exit 0; at most 3 version entries in output
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-18: `mode::history count::0` produces empty output

- **Given:** Network available.
- **When:** `clv .version.list mode::history count::0`
- **Then:** exit 0; stdout is empty
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-19: `mode::history v::0` produces bare version+date lines

- **Given:** Network available.
- **When:** `clv .version.list mode::history v::0 count::3`
- **Then:** exit 0; lines match `{semver}  {YYYY-MM-DD}` pattern, no summaries or labels
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-20: `mode::history v::1` shows version + date + summary

- **Given:** Network available.
- **When:** `clv .version.list mode::history v::1 count::3`
- **Then:** exit 0; each line has version, date, and a changelog summary
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-21: `mode::history v::2` shows full changelog with `##` headers

- **Given:** Network available.
- **When:** `clv .version.list mode::history v::2 count::2`
- **Then:** exit 0; output contains `##` markdown headers and `- ` bullet lines
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-22: `mode::history format::json` produces valid JSON array

- **Given:** Network available.
- **When:** `clv .version.list mode::history format::json count::3`
- **Then:** exit 0; JSON array where each element has `version`, `date`, `summary` fields
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-23: `mode::history count::1 format::json` produces single-element array

- **Given:** Network available.
- **When:** `clv .version.list mode::history count::1 format::json`
- **Then:** exit 0; JSON array with exactly 1 object element
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-24: `mode::history count::1 v::0` produces single bare line

- **Given:** Network available.
- **When:** `clv .version.list mode::history count::1 v::0`
- **Then:** exit 0; exactly 1 line of bare `{version}  {date}`
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-25: `mode::history count::1 v::2` produces single changelog block

- **Given:** Network available.
- **When:** `clv .version.list mode::history count::1 v::2`
- **Then:** exit 0; one `##` header block with changelog bullets for a single release
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-26: `mode::history` default count is 10

- **Given:** Network available; API has 66+ releases.
- **When:** `clv .version.list mode::history`
- **Then:** exit 0; at most 10 version entries in output
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-27: `mode::history count::100` returns all available releases

- **Given:** Network available.
- **When:** `clv .version.list mode::history count::100 v::0`
- **Then:** exit 0; all available releases (currently ~66), each on its own line
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-28: Idempotency — two `mode::history` calls produce identical output

- **Given:** Network available.
- **When:** `clv .version.list mode::history count::1` (run twice)
- **Then:** exit codes equal; both outputs byte-identical
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-29: Parameter order independence (`mode::history`)

- **Given:** Network available.
- **When:** run with both `mode::history count::3 v::0` and `v::0 mode::history count::3`
- **Then:** exit codes equal; outputs equal regardless of parameter order
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-30: `mode::history count::0 format::json` produces empty array

- **Given:** Network available.
- **When:** `clv .version.list mode::history count::0 format::json`
- **Then:** exit 0; output is `[]` (empty JSON array)
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-31: `mode::history format::xml` exits 1

- **Given:** clean environment
- **When:** `clv .version.list mode::history format::xml`
- **Then:** exit 1; stderr mentions unknown format
- **Exit:** 1
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-32: `mode::history format::JSON` (uppercase) exits 1

- **Given:** clean environment
- **When:** `clv .version.list mode::history format::JSON`
- **Then:** exit 1; same error as unknown format (case-sensitive)
- **Exit:** 1
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-33: `mode::history format::` (empty value) exits 1

- **Given:** clean environment
- **When:** `clv .version.list mode::history format::`
- **Then:** exit 1
- **Exit:** 1
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-34: `mode::history` unknown parameter exits 1

- **Given:** clean environment
- **When:** `clv .version.list mode::history bogus::x`
- **Then:** exit 1; error about unknown parameter
- **Exit:** 1
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-35: `mode::history`, network unavailable falls back to compiled-in snapshot

- **Given:** Network unavailable or curl unreachable; HOME set.
- **When:** `clv .version.list mode::history`
- **Then:** exit 0; stdout carries the compiled-in `VERSION_HISTORY` snapshot; stderr carries an advisory warning that the live fetch failed
- **Exit:** 0
- **Note:** Manual verification test — cannot be reliably triggered in standard CI
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-36: `mode::history`, HOME empty exits 2

- **Given:** Override HOME to empty string.
- **When:** `clv .version.list mode::history` with `HOME=""`
- **Then:** exit 2; error about HOME
- **Exit:** 2
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-37: `mode::history count::-1` (negative) exits 1

- **Given:** clean environment
- **When:** `clv .version.list mode::history count::-1`
- **Then:** exit 1; adapter rejects negative as type error
- **Exit:** 1
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-38: `mode::history v::abc` (non-integer) exits 1

- **Given:** clean environment
- **When:** `clv .version.list mode::history v::abc`
- **Then:** exit 1; type mismatch error
- **Exit:** 1
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-39: `mode::history count::abc` (non-integer) exits 1

- **Given:** clean environment
- **When:** `clv .version.list mode::history count::abc`
- **Then:** exit 1; type mismatch error
- **Exit:** 1
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-40: `mode::history --verbose` flag-style exits 1

- **Given:** clean environment
- **When:** `clv .version.list mode::history --verbose`
- **Then:** exit 1; error mentions `param::value` syntax
- **Exit:** 1
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-41: UTF-8 non-ASCII body characters preserved (`mode::history`)

- **Given:** `HOME=<tmp>`; write `version_history_cache.json` to `<tmp>/.claude/.transient/` containing a release body with raw em-dash (U+2014) and right-quote (U+2019) bytes (not `\uXXXX` escapes).
- **When:** `clv .version.list mode::history v::2 count::1`
- **Then:** exit 0; stdout contains the em-dash and right-quote characters intact; stdout does not contain garbled U+00E2 (`â`)
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-42: `mode::aliases` explicit is byte-identical to default

- **Given:** clean environment
- **When:** `clv .version.list mode::aliases` and `clv .version.list` (run both)
- **Then:** exit 0 for both; stdout byte-identical between the two invocations
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-43: `mode::bogus` exits 1

- **Given:** clean environment
- **When:** `clv .version.list mode::bogus`
- **Then:** exit 1; error mentions unknown mode, expected `aliases` or `history`
- **Exit:** 1
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-44: `mode::` (empty value) exits 1

- **Given:** clean environment
- **When:** `clv .version.list mode::`
- **Then:** exit 1; error mentions mode:: value
- **Exit:** 1
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-45: `mode::History` (wrong case) exits 1

- **Given:** clean environment
- **When:** `clv .version.list mode::History`
- **Then:** exit 1; same error as unknown mode; `mode::` is case-sensitive
- **Exit:** 1
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

### IT-46: `count::` under `mode::aliases` is accepted but has no effect

- **Given:** clean environment
- **When:** `clv .version.list count::5` and `clv .version.list` (run both; `mode::` absent = `aliases` in both)
- **Then:** exit 0 for both; stdout byte-identical between the two invocations (count:: does not truncate alias output, does not error as unknown parameter)
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md)

---

## Test Implementation Strategy

### Network-Conditional Helper

```rust
fn skip_if_no_network( out : &std::process::Output ) -> bool
{
  if out.status.code() == Some( 2 )
  {
    let err = String::from_utf8_lossy( &out.stderr );
    if err.contains( "failed to fetch" ) { return true; }
  }
  false
}
```

Tests requiring successful network fetch:
1. Run the command
2. If `skip_if_no_network()` returns true → return (test passes vacuously)
3. Otherwise assert exit 0 and verify output format

### Test Isolation Patterns

| Pattern | Tests | Method |
|---------|-------|--------|
| No isolation needed | IT-1 through IT-6 (`mode::aliases`), IT-31 through IT-34, IT-37 through IT-40, IT-43 through IT-45 | Fails at adapter/validation, or reads compile-time data only |
| Network-conditional | IT-16 through IT-30, IT-41 | `skip_if_no_network()` helper |
| HOME isolation | IT-36, IT-41 | `run_cm_with_env(&[...], &[("HOME", "<tmp>")])` |
| Manual verification | IT-35 | Cannot reliably trigger in CI |

### Categories Summary

| Category | Tests | Count |
|----------|-------|-------|
| Basic Invocation (`mode::aliases`) | IT-1 | 1 |
| Content Presence (`mode::aliases`) | IT-9, IT-10, IT-13 | 3 |
| Verbosity (`mode::aliases`) | IT-2, IT-11, IT-14 | 3 |
| Stability (`mode::aliases`) | IT-3, IT-8 | 2 |
| Format (`mode::aliases`) | IT-7, IT-12, IT-15 | 3 |
| Param Validation (`mode::aliases`) | IT-4 | 1 |
| Format Validation (`mode::aliases`) | IT-5 | 1 |
| Verbosity Validation (`mode::aliases`) | IT-6 | 1 |
| Basic Invocation (`mode::history`) | IT-16 | 1 |
| Count Param | IT-17 | 1 |
| Count Boundary | IT-18, IT-27 | 2 |
| Verbosity (`mode::history`) | IT-19, IT-20, IT-21 | 3 |
| Format (`mode::history`) | IT-22 | 1 |
| Pairwise: count x format | IT-23, IT-30 | 2 |
| Pairwise: count x verbosity | IT-24, IT-25 | 2 |
| Default Behavior (`mode::history`) | IT-26 | 1 |
| Stability (`mode::history`) | IT-28 | 1 |
| Commutativity (`mode::history`) | IT-29 | 1 |
| Format Validation (`mode::history`) | IT-31, IT-32, IT-33 | 3 |
| Param Validation (`mode::history`) | IT-34 | 1 |
| Fallback Handling | IT-35 | 1 |
| Environment | IT-36 | 1 |
| Type Validation | IT-37, IT-38, IT-39 | 3 |
| Syntax Validation | IT-40 | 1 |
| Bug Fix | IT-41 | 1 |
| Mode Switch | IT-42 | 1 |
| Mode Validation | IT-43, IT-44, IT-45 | 3 |
| Cross-Mode Interaction | IT-46 | 1 |

---

### Source Functions

| Function | File |
|----------|------|
| `tc115_version_list_exits_0` | `tests/cli/read_version_test.rs` |
| `tc116_version_list_includes_stable` | `tests/cli/read_version_test.rs` |
| `tc117_version_list_includes_latest` | `tests/cli/read_version_test.rs` |
| `tc118_version_list_v0_names_only` | `tests/cli/read_version_test.rs` |
| `tc119_version_list_v1_has_descriptions` | `tests/cli/read_version_test.rs` |
| `tc120_version_list_is_idempotent` | `tests/cli/read_version_test.rs` |
| `tc121_version_list_format_json_array` | `tests/cli/read_version_test.rs` |
| `tc122_version_list_includes_month` | `tests/cli/read_version_test.rs` |
| `tc123_version_list_v1_shows_pinned_versions` | `tests/cli/read_version_test.rs` |
| `tc124_version_list_json_has_value_field` | `tests/cli/read_version_test.rs` |
| `it04_version_list_bogus_param_exits_1` | `tests/cli/read_version_test.rs` |
| `it05_version_list_format_xml_exits_1` | `tests/cli/read_version_test.rs` |
| `it06_version_list_v3_exits_1` | `tests/cli/read_version_test.rs` |
| `it07_version_list_format_json_valid` | `tests/cli/read_version_test.rs` |
| `it08_version_list_output_stable` | `tests/cli/read_version_test.rs` |
| `it16_version_list_mode_history_defaults_exit_0` | `tests/cli/read_version_test.rs` |
| `it17_version_list_mode_history_count_3` | `tests/cli/read_version_test.rs` |
| `it18_version_list_mode_history_count_0_empty` | `tests/cli/read_version_test.rs` |
| `it19_version_list_mode_history_v0_bare` | `tests/cli/read_version_test.rs` |
| `it20_version_list_mode_history_v1_with_summary` | `tests/cli/read_version_test.rs` |
| `it21_version_list_mode_history_v2_full_changelog` | `tests/cli/read_version_test.rs` |
| `it22_version_list_mode_history_format_json` | `tests/cli/read_version_test.rs` |
| `it23_version_list_mode_history_count_1_json` | `tests/cli/read_version_test.rs` |
| `it24_version_list_mode_history_count_1_v0` | `tests/cli/read_version_test.rs` |
| `it25_version_list_mode_history_count_1_v2` | `tests/cli/read_version_test.rs` |
| `it26_version_list_mode_history_default_count_le_10` | `tests/cli/read_version_test.rs` |
| `it27_version_list_mode_history_count_100_all` | `tests/cli/read_version_test.rs` |
| `it28_version_list_mode_history_idempotent` | `tests/cli/read_version_test.rs` |
| `it29_version_list_mode_history_param_order` | `tests/cli/read_version_test.rs` |
| `it30_version_list_mode_history_count_0_json_empty_array` | `tests/cli/read_version_test.rs` |
| `it31_version_list_mode_history_format_xml_exits_1` | `tests/cli/read_version_test.rs` |
| `it32_version_list_mode_history_format_uppercase_exits_1` | `tests/cli/read_version_test.rs` |
| `it33_version_list_mode_history_format_empty_exits_1` | `tests/cli/read_version_test.rs` |
| `it34_version_list_mode_history_unknown_param_exits_1` | `tests/cli/read_version_test.rs` |
| IT-35 (manual — no automated function) | — |
| `it36_version_list_mode_history_no_home_exits_2` | `tests/cli/read_version_test.rs` |
| `it37_version_list_mode_history_negative_count_exits_1` | `tests/cli/read_version_test.rs` |
| `it38_version_list_mode_history_v_abc_exits_1` | `tests/cli/read_version_test.rs` |
| `it39_version_list_mode_history_count_abc_exits_1` | `tests/cli/read_version_test.rs` |
| `it40_version_list_mode_history_flag_style_exits_1` | `tests/cli/read_version_test.rs` |
| `it41_version_list_mode_history_utf8_body_preserved` | `tests/cli/read_version_test.rs` |
| `it42_version_list_mode_aliases_matches_default` | `tests/cli/read_version_test.rs` |
| `it43_version_list_mode_bogus_exits_1` | `tests/cli/read_version_test.rs` |
| `it44_version_list_mode_empty_exits_1` | `tests/cli/read_version_test.rs` |
| `it45_version_list_mode_wrong_case_exits_1` | `tests/cli/read_version_test.rs` |
| `it46_version_list_count_inert_under_aliases` | `tests/cli/read_version_test.rs` |
