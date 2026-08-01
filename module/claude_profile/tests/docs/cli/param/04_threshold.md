# Test: `threshold::`

Edge case coverage for the `threshold::` parameter. See [params.md](../../../../docs/cli/param/003_threshold.md) and [types.md](../../../../docs/cli/type/003_warning_threshold.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `threshold::3600` matches default behavior (60 min) | Default Equivalence |
| EC-2 | `threshold::0` disables ExpiringSoon — only Valid or Expired | Boundary Min |
| EC-3 | `threshold::1800` narrows window to 30 minutes | Custom Value |
| EC-4 | `threshold::7200` widens window to 2 hours | Custom Value |
| EC-5 | `threshold::abc` exits 1 (non-integer) | Invalid Type |
| EC-6 | `threshold::86400` (24 hours) — most tokens classified as ExpiringSoon | Large Value |
| EC-7 | Omitted `threshold::` defaults to `3600` | Default |
| EC-8 | Duplicate `threshold::0 threshold::7200` — last wins | Last Wins |

### Test Coverage Summary

- Default Equivalence: 1 test
- Boundary Min: 1 test
- Custom Value: 2 tests
- Invalid Type: 1 test
- Large Value: 1 test
- Default: 1 test
- Last Wins: 1 test

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

---

### EC-1: Default Equivalence

- **Given:** Active credentials exist at `~/.claude/.credentials.json` with a token that has more than 60 minutes remaining.
- **When:** `clp .credentials.status threshold::3600`
- **Then:** `Token:` line shows `valid`. Exit 0. Output identical to `clp .credentials.status` (no threshold parameter).; explicit `threshold::3600` output identical to default threshold output
- **Exit:** 0
- **Source:** [params.md -- threshold::](../../../../docs/cli/param/003_threshold.md)

---

### EC-2: Boundary Min

- **Given:** Active credentials exist at `~/.claude/.credentials.json` with a token that has between 1 and 59 minutes remaining (would normally be ExpiringSoon).
- **When:** `clp .credentials.status threshold::0`
- **Then:** `Token:` line shows `valid` (not `expiring in Xm`) despite having less than 60 minutes remaining. Exit 0.; token classified as `valid` because ExpiringSoon is disabled at threshold 0
- **Exit:** 0
- **Source:** [types.md -- WarningThreshold](../../../../docs/cli/type/003_warning_threshold.md)

---

### EC-3: Custom Value — Narrow Window

- **Given:** Active credentials exist at `~/.claude/.credentials.json` with a token that has approximately 45 minutes remaining.
- **When:** `clp .credentials.status threshold::1800`
- **Then:** `Token:` line shows `valid` (45 minutes > 30 minute threshold). Exit 0.; 30-minute threshold classifies a 45-minute token as `valid`
- **Exit:** 0
- **Source:** [params.md -- threshold::](../../../../docs/cli/param/003_threshold.md)

---

### EC-4: Custom Value — Wide Window

- **Given:** Active credentials exist at `~/.claude/.credentials.json` with a token that has approximately 90 minutes remaining.
- **When:** `clp .credentials.status threshold::7200`
- **Then:** `Token:` line shows `expiring in 90m` (90 minutes < 2 hour threshold). Exit 0.; 2-hour threshold classifies a 90-minute token as `expiring in Xm`
- **Exit:** 0
- **Source:** [params.md -- threshold::](../../../../docs/cli/param/003_threshold.md)

---

### EC-5: Invalid Type

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .credentials.status threshold::abc`
- **Then:** Error message containing `invalid threshold 'abc'` with exit 1.; non-integer threshold value rejected with descriptive error
- **Exit:** 1
- **Source:** [types.md -- WarningThreshold](../../../../docs/cli/type/003_warning_threshold.md)

---

### EC-6: Large Value

- **Given:** Active credentials exist at `~/.claude/.credentials.json` with a token that has several hours remaining (e.g., 4 hours).
- **When:** `clp .credentials.status threshold::86400`
- **Then:** `Token:` line shows `expiring in 240m` because remaining time (4 hours) is less than 24 hours. Exit 0.; 24-hour threshold classifies most tokens as `expiring in Xm`
- **Exit:** 0
- **Source:** [params.md -- threshold::](../../../../docs/cli/param/003_threshold.md)

---

### EC-7: Default

- **Given:** Active credentials exist at `~/.claude/.credentials.json` with a token that has approximately 45 minutes remaining.
- **When:** `clp .credentials.status`
- **Then:** `Token:` line shows `expiring in 45m` (45 minutes < 60 minute default threshold). Exit 0.; default threshold is 3600, classifying a 45-minute token as `expiring in Xm`
- **Exit:** 0
- **Source:** [params.md -- threshold::](../../../../docs/cli/param/003_threshold.md)

---

### EC-8: Last Wins

- **Given:** Active credentials exist at `~/.claude/.credentials.json` with a token that has approximately 90 minutes remaining.
- **When:** `clp .credentials.status threshold::0 threshold::7200`
- **Then:** `Token:` line shows `expiring in 90m` (matching `threshold::7200` behavior, not `threshold::0`). Exit 0.; last `threshold::` value (7200) takes effect, producing `expiring in Xm`
- **Exit:** 0
- **Source:** [params.md -- threshold::](../../../../docs/cli/param/003_threshold.md)
