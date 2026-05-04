# Test: `.version.install`

Integration test planning for the `.version.install` command. See [commands.md](../../../../docs/cli/commands.md) for specification.

## Test Factor Analysis

### Factor 1: `version::` (String, optional, default "stable")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Defaults to `stable` | Default behavior |
| `stable` | Named alias (pinned) | Valid alias |
| `month` | Named alias (pinned) | Valid alias |
| `latest` | Named alias (dynamic) | Valid alias (special: no lock) |
| `1.2.3` | Valid semver | Valid semver |
| `2.1.50` | Valid semver (older) | Valid semver |
| `0.0.0` | Minimal semver | Boundary: minimum valid |
| `STABLE` | Wrong-case alias | Invalid: exit 1 |
| (empty) | Empty string | Invalid: exit 1 |
| `1.2` | Two-part semver | Invalid: exit 1 |
| `01.02.03` | Leading zeros | Invalid: exit 1 |
| `x` | Unknown alias | Invalid: exit 1 |

Boundary set: `0.0.0`, `latest`, two-part, leading-zeros.

### Factor 2: `dry::` (Boolean, optional, default 0)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: no dry-run, real install | Default behavior |
| 0 | Explicit: real install | Explicit false |
| 1 | Preview mode; no side effects | Explicit true |
| 2 | Out-of-range boolean | Invalid: exit 1 |

### Factor 3: `force::` (Boolean, optional, default 0)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: idempotency guard active | Default behavior |
| 0 | Explicit: guard active | Explicit false |
| 1 | Bypass idempotency check | Explicit true |

### Factor 4: Interaction: `dry::1` vs `force::1`

| Combination | Behavior | Expected |
|-------------|----------|---------|
| `dry::1 force::1` | dry wins, no install | Preview only |
| `force::1` alone | bypass guard, real install | Real install |

### Factor 5: Lock actions for version type

| Version type | Lock behavior | Description |
|-------------|---------------|-------------|
| pinned alias / semver | autoUpdates=false, DISABLE_AUTOUPDATER=1, chmod 555, purge stale binaries, store preferredVersionSpec/Resolved | 5-layer lock |
| `latest` | autoUpdates=true, remove DISABLE_AUTOUPDATER, chmod 755 | Unlock |

### Factor 6: Preference storage

| Scenario | Expected |
|----------|---------|
| Successful install | `preferredVersionSpec` + `preferredVersionResolved` written |
| `dry::1` | Preference keys NOT written |
| Idempotent skip | Preference still written |

### Factor 7: Unknown parameters

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| none | No unknown params | Happy path |
| present | e.g. `bogus::x` | Invalid: exit 1 |

### Factor 8: `verbosity::` / `v::` (Integer, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: labeled output | Default behavior |
| 0 | Bare/compact output | Compact |
| 1 | Labeled output | Labeled |

### Factor 9: `format::` (String, optional, default "text")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: text output | Default behavior |
| `text` | Human-readable text | Valid |
| `json` | Machine-readable JSON | Valid |
| `JSON` | Wrong case | Invalid: exit 1 |

---

## Test Matrix

### Positive Tests (dry-run mode — no network needed)

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-1 | `dry::1` → `[dry-run]` prefix, exit 0 | P | 0 | F2=1 | [mutation_commands_test.rs] |
| TC-301 | `version::stable dry::1` → preview shows `stable` | P | 0 | F1=stable, F2=1 | [mutation_commands_test.rs] |
| TC-302 | `version::1.2.3 dry::1` → preview shows exact version | P | 0 | F1=semver, F2=1 | [mutation_commands_test.rs] |
| IT-2 | `dry::1 force::1` → dry wins | P | 0 | F2=1, F3=1, F4 | [mutation_commands_test.rs] |
| TC-308 | Absent `version::` with `dry::1` → uses `stable` | P | 0 | F1=absent, F2=1 | [mutation_commands_test.rs] |
| TC-309 | `version::month dry::1` → resolves to pinned semver (2.1.74) | P | 0 | F1=month, F2=1 | [mutation_commands_test.rs] |
| TC-350 | `version::latest dry::1` → autoUpdates=true in preview | P | 0 | F1=latest, F2=1, F5=unlock | [mutation_commands_test.rs] |
| TC-351 | `version::stable dry::1` → autoUpdates=false in preview | P | 0 | F1=stable, F2=1, F5=lock | [mutation_commands_test.rs] |
| TC-352 | `version::2.1.50 dry::1` → autoUpdates=false in preview | P | 0 | F1=semver, F2=1, F5=lock | [mutation_commands_test.rs] |
| TC-353 | `version::latest dry::1` → previews unlock actions | P | 0 | F1=latest, F2=1, F5=unlock | [mutation_commands_test.rs] |
| TC-355 | `version::0.0.0 dry::1` → single-zero parts valid | P | 0 | F1=0.0.0, F2=1 | [mutation_commands_test.rs] |
| TC-356 | `dry::1` output mentions preferred version storage | P | 0 | F2=1, F6 | [mutation_commands_test.rs] |
| TC-357 | `dry::1` does NOT write preference keys | P | 0 | F2=1, F6=no-write | [mutation_commands_test.rs] |
| TC-358 | Idempotent skip still stores preference | P | 0 | F6=idempotent | [mutation_commands_test.rs] |
| TC-359 | `version::stable dry::1` → output includes purge line | P | 0 | F2=1, F5=layer4 | [mutation_commands_test.rs] |
| TC-360 | `version::latest dry::1` → output does NOT contain "purge" | P | 0 | F1=latest, F2=1, F5=unlock | [mutation_commands_test.rs] |
| IT-6 | `dry::1 format::json` → JSON object output, exit 0 | P | 0 | F2=1, F9=json | [mutation_commands_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-3 | `version::STABLE` → wrong case, exit 1 | N | 1 | F1=STABLE | [mutation_commands_test.rs] |
| TC-305 | `version::` (empty) → exit 1 | N | 1 | F1=empty | [mutation_commands_test.rs] |
| TC-306 | `version::1.2` → two-part semver, exit 1 | N | 1 | F1=1.2 | [mutation_commands_test.rs] |
| TC-307 | `version::x` → unknown alias, exit 1 | N | 1 | F1=x | [mutation_commands_test.rs] |
| IT-4 | `version::01.02.03` → leading zeros, exit 1 | N | 1 | F1=leading-zeros | [mutation_commands_test.rs] |
| IT-5 | `bogus::x` → unknown param, exit 1 | N | 1 | F7=present | new |
| IT-7 | `format::JSON` (uppercase) → exit 1 | N | 1 | F9=JSON | [mutation_commands_test.rs] |
| IT-8 | `dry::2` → out-of-range boolean, exit 1 | N | 1 | F2=2 | [mutation_commands_test.rs] |

### Summary

- **Total:** 25 tests (17 positive, 8 negative)
- **Negative ratio:** 32.0% — supplemented by cross-cutting TC-242 to TC-244, TC-261
- **Combined with cross-cutting:** 11/26 = 42.3% ✅
- **TC range:** IT-1 to IT-7, IT-5 to IT-8

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (dry-run or real) | IT-1 through IT-2, TC-308, TC-309, TC-350 to TC-358 |
| 1 | Invalid arguments | IT-3 through TC-307, IT-4, IT-5, IT-8 |
| 2 | Runtime error (install failure) | Real-install tests only (not in automated suite; covered by TC-444 pattern) |

### Dry-Run Parity Requirement (FR-05)

`[dry-run] would install X (Y)` must exactly mirror actual install message.
IT-1 through IT-2 verify dry-run prefix and content consistency.
TC-357 verifies dry-run has zero side effects on settings.

### Version Lock / Unlock Coverage (FR-15)

| Scenario | Lock Status | Test |
|----------|-------------|------|
| pinned semver (2.1.50) | 5-layer lock | TC-352, TC-359 (layers 1–4), TC-356 (layer 5) |
| stable alias | 5-layer lock | TC-351, TC-359 (layers 1–4), TC-356 (layer 5) |
| latest alias | Remove all locks | TC-350, TC-353 |

### Preference Storage Coverage (FR-17)

| Scenario | Written? | Test |
|----------|----------|------|
| dry::1 | No | TC-357 |
| real install (idempotent skip) | Yes | TC-358 |
| dry::1 (preview mentions storage) | Preview only | TC-356 |

---

## Test Case Details

---

### IT-1: `dry::1` → `[dry-run]` prefix

- **Given:** clean environment
- **When:**
  `cm .version.install dry::1`
  **Expected:** Exit 0; stdout contains `[dry-run]`.
- **Then:** dry-run marker present
- **Exit:** 0

---

### IT-2: `dry::1 force::1` → dry wins

- **Given:** clean environment
- **When:**
  `cm .version.install dry::1 force::1`
  **Expected:** Exit 0; stdout contains `[dry-run]`; no install.
- **Then:** preview mode only
- **Exit:** 0

---

### IT-3: `version::STABLE` → exit 1

- **Given:** clean environment
- **When:**
  `cm .version.install version::STABLE`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-4: `version::01.02.03` → leading zeros rejected

- **Given:** clean environment
- **When:**
  `cm .version.install version::01.02.03`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-5: `bogus::x` → unknown parameter, exit 1

- **Given:** clean environment
- **When:**
  `cm .version.install bogus::x`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-6: `dry::1 format::json` → JSON object output

- **Given:** clean environment
- **When:**
  `cm .version.install dry::1 format::json`
  **Expected:** Exit 0; stdout starts with `{`.
- **Then:** JSON object output
- **Exit:** 0

---

### IT-7: `format::JSON` (uppercase) → exit 1

- **Given:** clean environment
- **When:**
  `cm .version.install format::JSON`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-8: `dry::2` → out-of-range boolean, exit 1

- **Given:** clean environment
- **When:**
  `cm .version.install dry::2`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1
