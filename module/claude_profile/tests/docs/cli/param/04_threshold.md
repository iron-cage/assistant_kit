# Test: `threshold::`

Edge case coverage for the `threshold::` parameter. See [params.md](../../../../docs/cli/params.md#parameter--4-threshold) and [types.md](../../../../docs/cli/types.md#type--4-warningthreshold) for specification.

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
- **When:** `clp .token.status threshold::3600`
- **Then:** Token classified as `valid` with remaining time shown. Exit 0. Output identical to `clp .token.status` (no threshold parameter).; explicit `threshold::3600` output identical to default threshold output
- **Exit:** 0
- **Source:** [params.md -- threshold::](../../../../docs/cli/params.md#parameter--4-threshold)

---

### EC-2: Boundary Min

- **Given:** Active credentials exist at `~/.claude/.credentials.json` with a token that has between 1 and 59 minutes remaining (would normally be ExpiringSoon).
- **When:** `clp .token.status threshold::0`
- **Then:** Token classified as `valid` (not `expiring soon`) despite having less than 60 minutes remaining. Exit 0.; token classified as `valid` because ExpiringSoon is disabled at threshold 0
- **Exit:** 0
- **Source:** [types.md -- WarningThreshold](../../../../docs/cli/types.md#type--4-warningthreshold)

---

### EC-3: Custom Value — Narrow Window

- **Given:** Active credentials exist at `~/.claude/.credentials.json` with a token that has approximately 45 minutes remaining.
- **When:** `clp .token.status threshold::1800`
- **Then:** Token classified as `valid` (45 minutes > 30 minute threshold). Exit 0.; 30-minute threshold classifies a 45-minute token as `valid`
- **Exit:** 0
- **Source:** [params.md -- threshold::](../../../../docs/cli/params.md#parameter--4-threshold)

---

### EC-4: Custom Value — Wide Window

- **Given:** Active credentials exist at `~/.claude/.credentials.json` with a token that has approximately 90 minutes remaining.
- **When:** `clp .token.status threshold::7200`
- **Then:** Token classified as `expiring soon` (90 minutes < 2 hour threshold). Exit 0.; 2-hour threshold classifies a 90-minute token as `expiring soon`
- **Exit:** 0
- **Source:** [params.md -- threshold::](../../../../docs/cli/params.md#parameter--4-threshold)

---

### EC-5: Invalid Type

- **Given:** Active credentials exist at `~/.claude/.credentials.json`.
- **When:** `clp .token.status threshold::abc`
- **Then:** Error message containing `invalid threshold 'abc'` with exit 1.; non-integer threshold value rejected with descriptive error
- **Exit:** 1
- **Source:** [types.md -- WarningThreshold](../../../../docs/cli/types.md#type--4-warningthreshold)

---

### EC-6: Large Value

- **Given:** Active credentials exist at `~/.claude/.credentials.json` with a token that has several hours remaining (e.g., 4 hours).
- **When:** `clp .token.status threshold::86400`
- **Then:** Token classified as `expiring soon` because remaining time (4 hours) is less than 24 hours. Exit 0.; 24-hour threshold classifies most tokens as `expiring soon`
- **Exit:** 0
- **Source:** [params.md -- threshold::](../../../../docs/cli/params.md#parameter--4-threshold)

---

### EC-7: Default

- **Given:** Active credentials exist at `~/.claude/.credentials.json` with a token that has approximately 45 minutes remaining.
- **When:** `clp .token.status`
- **Then:** Token classified as `expiring soon` (45 minutes < 60 minute default threshold). Exit 0.; default threshold is 3600, classifying a 45-minute token as `expiring soon`
- **Exit:** 0
- **Source:** [params.md -- threshold::](../../../../docs/cli/params.md#parameter--4-threshold)

---

### EC-8: Last Wins

- **Given:** Active credentials exist at `~/.claude/.credentials.json` with a token that has approximately 90 minutes remaining.
- **When:** `clp .token.status threshold::0 threshold::7200`
- **Then:** Token classified as `expiring soon` (matching `threshold::7200` behavior, not `threshold::0`). Exit 0.; last `threshold::` value (7200) takes effect, producing `expiring soon`
- **Exit:** 0
- **Source:** [params.md -- threshold::](../../../../docs/cli/params.md#parameter--4-threshold)
