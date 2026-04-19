# Test: `threshold::`

Edge case coverage for the `threshold::` parameter. See [params.md](../../../../../docs/cli/params.md#parameter--4-threshold) and [types.md](../../../../../docs/cli/types.md#type--4-warningthreshold) for specification.

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

---

### EC-1: Default Equivalence

**Goal:** Confirm that explicitly passing `threshold::3600` produces the same output as the default.
**Setup:** Active credentials exist at `~/.claude/.credentials.json` with a token that has more than 60 minutes remaining.
**Command:** `clp .token.status threshold::3600`
**Expected Output:** Token classified as `valid` with remaining time shown. Exit 0. Output identical to `clp .token.status` (no threshold parameter).
**Verification:**
- Exit code is 0
- Output matches the output of `clp .token.status` exactly
- Token status is `valid` (not `expiring soon`) when token has >60 minutes remaining
**Pass Criteria:** Exit 0; explicit `threshold::3600` output identical to default threshold output.
**Source:** [params.md -- threshold::](../../../../../docs/cli/params.md#parameter--4-threshold)

---

### EC-2: Boundary Min

**Goal:** Confirm that `threshold::0` disables the ExpiringSoon classification entirely.
**Setup:** Active credentials exist at `~/.claude/.credentials.json` with a token that has between 1 and 59 minutes remaining (would normally be ExpiringSoon).
**Command:** `clp .token.status threshold::0`
**Expected Output:** Token classified as `valid` (not `expiring soon`) despite having less than 60 minutes remaining. Exit 0.
**Verification:**
- Exit code is 0
- Output status is `valid`, not `expiring soon`
- With default threshold the same token would be classified as `expiring soon`
**Pass Criteria:** Exit 0; token classified as `valid` because ExpiringSoon is disabled at threshold 0.
**Source:** [types.md -- WarningThreshold](../../../../../docs/cli/types.md#type--4-warningthreshold)

---

### EC-3: Custom Value — Narrow Window

**Goal:** Confirm that `threshold::1800` narrows the ExpiringSoon window to 30 minutes.
**Setup:** Active credentials exist at `~/.claude/.credentials.json` with a token that has approximately 45 minutes remaining.
**Command:** `clp .token.status threshold::1800`
**Expected Output:** Token classified as `valid` (45 minutes > 30 minute threshold). Exit 0.
**Verification:**
- Exit code is 0
- Output status is `valid` for a token with 45 minutes remaining
- With default `threshold::3600`, the same token would be classified as `expiring soon`
**Pass Criteria:** Exit 0; 30-minute threshold classifies a 45-minute token as `valid`.
**Source:** [params.md -- threshold::](../../../../../docs/cli/params.md#parameter--4-threshold)

---

### EC-4: Custom Value — Wide Window

**Goal:** Confirm that `threshold::7200` widens the ExpiringSoon window to 2 hours.
**Setup:** Active credentials exist at `~/.claude/.credentials.json` with a token that has approximately 90 minutes remaining.
**Command:** `clp .token.status threshold::7200`
**Expected Output:** Token classified as `expiring soon` (90 minutes < 2 hour threshold). Exit 0.
**Verification:**
- Exit code is 0
- Output status is `expiring soon` for a token with 90 minutes remaining
- With default `threshold::3600`, the same token would be classified as `valid`
**Pass Criteria:** Exit 0; 2-hour threshold classifies a 90-minute token as `expiring soon`.
**Source:** [params.md -- threshold::](../../../../../docs/cli/params.md#parameter--4-threshold)

---

### EC-5: Invalid Type

**Goal:** Confirm that a non-integer threshold value is rejected.
**Setup:** Active credentials exist at `~/.claude/.credentials.json`.
**Command:** `clp .token.status threshold::abc`
**Expected Output:** Error message containing `invalid threshold 'abc'` with exit 1.
**Verification:**
- Exit code is 1
- Stderr contains `invalid threshold` and references the bad value `abc`
- Stderr mentions `expected seconds as integer`
- No token status produced on stdout
**Pass Criteria:** Exit 1; non-integer threshold value rejected with descriptive error.
**Source:** [types.md -- WarningThreshold](../../../../../docs/cli/types.md#type--4-warningthreshold)

---

### EC-6: Large Value

**Goal:** Confirm that a very large threshold (24 hours) causes most non-expired tokens to be classified as ExpiringSoon.
**Setup:** Active credentials exist at `~/.claude/.credentials.json` with a token that has several hours remaining (e.g., 4 hours).
**Command:** `clp .token.status threshold::86400`
**Expected Output:** Token classified as `expiring soon` because remaining time (4 hours) is less than 24 hours. Exit 0.
**Verification:**
- Exit code is 0
- Output status is `expiring soon`
- Only tokens with more than 24 hours remaining would be classified as `valid` at this threshold
**Pass Criteria:** Exit 0; 24-hour threshold classifies most tokens as `expiring soon`.
**Source:** [params.md -- threshold::](../../../../../docs/cli/params.md#parameter--4-threshold)

---

### EC-7: Default

**Goal:** Confirm that omitting `threshold::` defaults to 3600 seconds (60 minutes).
**Setup:** Active credentials exist at `~/.claude/.credentials.json` with a token that has approximately 45 minutes remaining.
**Command:** `clp .token.status`
**Expected Output:** Token classified as `expiring soon` (45 minutes < 60 minute default threshold). Exit 0.
**Verification:**
- Exit code is 0
- Output status is `expiring soon` for a token with 45 minutes remaining
- Behavior matches `clp .token.status threshold::3600`
**Pass Criteria:** Exit 0; default threshold is 3600, classifying a 45-minute token as `expiring soon`.
**Source:** [params.md -- threshold::](../../../../../docs/cli/params.md#parameter--4-threshold)

---

### EC-8: Last Wins

**Goal:** Confirm that when `threshold::` is specified multiple times, the last occurrence takes precedence.
**Setup:** Active credentials exist at `~/.claude/.credentials.json` with a token that has approximately 90 minutes remaining.
**Command:** `clp .token.status threshold::0 threshold::7200`
**Expected Output:** Token classified as `expiring soon` (matching `threshold::7200` behavior, not `threshold::0`). Exit 0.
**Verification:**
- Exit code is 0
- Output status is `expiring soon` (90 minutes < 7200 seconds = 2 hours)
- If `threshold::0` had won, status would be `valid` instead
**Pass Criteria:** Exit 0; last `threshold::` value (7200) takes effect, producing `expiring soon`.
**Source:** [params.md -- threshold::](../../../../../docs/cli/params.md#parameter--4-threshold)
