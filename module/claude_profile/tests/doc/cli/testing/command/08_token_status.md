# Test: `.token.status`

Integration test planning for the `.token.status` command. See [commands.md](../../../../../docs/cli/commands.md#command--7-tokenstatus) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Valid token shows "valid" with remaining time | Basic Invocation |
| IT-2 | Expired token shows "expired" | Status Classification |
| IT-3 | ExpiringSoon token shows "expiring soon" with remaining time | Status Classification |
| IT-4 | Custom `threshold::1800` changes classification boundary | Threshold Override |
| IT-5 | `threshold::0` never classifies as ExpiringSoon | Threshold Edge |
| IT-6 | `format::json` returns JSON object with status and expires_in_secs | Output Format |
| IT-7 | `v::0` shows bare status word only | Verbosity |
| IT-8 | Missing credentials file exits 2 | Error Handling |
| IT-9 | Credentials file with unparseable `expiresAt` exits 2 | Error Handling |
| IT-10 | `v::2` shows raw expiresAt timestamp and threshold value | Verbosity |

### Test Coverage Summary

- Basic Invocation: 1 test
- Status Classification: 2 tests
- Threshold Override: 1 test
- Threshold Edge: 1 test
- Output Format: 1 test
- Verbosity: 2 tests
- Error Handling: 2 tests

**Total:** 10 integration tests

---

### IT-1: Valid token shows "valid" with remaining time

**Goal:** Verify that a token with `expiresAt` well beyond the threshold displays "valid" status with human-readable remaining time.
**Setup:** `~/.claude/.credentials.json` contains an `expiresAt` timestamp set to current time + 7200 seconds (2 hours from now). Default threshold is 3600 seconds.
**Command:** `clp .token.status`
**Expected Output:** Output on stdout matching pattern `valid — *remaining` (e.g., `valid — 1h59m remaining`), exit 0.
**Verification:**
- Stdout contains the word `valid`
- Stdout contains a human-readable duration (e.g., `1h59m remaining` or `119m remaining`)
- Stdout does not contain `expired` or `expiring soon`
- Exit code is 0
**Pass Criteria:** Exit 0; output contains "valid" and remaining time.
**Source:** [commands.md — .token.status](../../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-2: Expired token shows "expired"

**Goal:** Verify that a token with `expiresAt` in the past is classified as "expired".
**Setup:** `~/.claude/.credentials.json` contains an `expiresAt` timestamp set to current time - 3600 seconds (1 hour ago).
**Command:** `clp .token.status`
**Expected Output:** Output on stdout containing `expired`, exit 0.
**Verification:**
- Stdout contains the word `expired`
- Stdout does not contain `valid` as a standalone status (may appear as substring in `expired`)
- Stdout does not contain `expiring soon`
- Exit code is 0
**Pass Criteria:** Exit 0; output contains "expired".
**Source:** [commands.md — .token.status](../../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-3: ExpiringSoon token shows "expiring soon" with remaining time

**Goal:** Verify that a token within the threshold window is classified as "expiring soon" with remaining time shown.
**Setup:** `~/.claude/.credentials.json` contains an `expiresAt` timestamp set to current time + 1800 seconds (30 minutes from now). Default threshold is 3600 seconds, so 1800 < 3600 triggers ExpiringSoon.
**Command:** `clp .token.status`
**Expected Output:** Output on stdout matching pattern `expiring soon — *remaining` (e.g., `expiring soon — 29m remaining`), exit 0.
**Verification:**
- Stdout contains the phrase `expiring soon`
- Stdout contains a human-readable duration
- Exit code is 0
**Pass Criteria:** Exit 0; output contains "expiring soon" and remaining time.
**Source:** [commands.md — .token.status](../../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-4: Custom threshold changes classification boundary

**Goal:** Verify that `threshold::1800` narrows the ExpiringSoon window so a token at 2500 seconds remaining is classified as "valid" instead of "expiring soon".
**Setup:** `~/.claude/.credentials.json` contains an `expiresAt` timestamp set to current time + 2500 seconds. With default threshold (3600), this would be "expiring soon". With threshold 1800, it should be "valid".
**Command:** `clp .token.status threshold::1800`
**Expected Output:** Output on stdout containing `valid`, exit 0.
**Verification:**
- Stdout contains the word `valid`
- Stdout does not contain `expiring soon`
- Exit code is 0
- Running the same credentials without `threshold::1800` (default 3600) would produce `expiring soon`
**Pass Criteria:** Exit 0; output contains "valid" under custom threshold.
**Source:** [commands.md — .token.status](../../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-5: Threshold zero never classifies as ExpiringSoon

**Goal:** Verify that `threshold::0` eliminates the ExpiringSoon classification entirely: tokens are either "valid" or "expired", never "expiring soon".
**Setup:** `~/.claude/.credentials.json` contains an `expiresAt` timestamp set to current time + 60 seconds (1 minute remaining). With default threshold (3600), this would be "expiring soon".
**Command:** `clp .token.status threshold::0`
**Expected Output:** Output on stdout containing `valid`, exit 0.
**Verification:**
- Stdout contains the word `valid`
- Stdout does not contain `expiring soon`
- Exit code is 0
**Pass Criteria:** Exit 0; output contains "valid" even with very little time remaining.
**Source:** [commands.md — .token.status](../../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-6: JSON format returns structured object

**Goal:** Verify that `format::json` produces a valid JSON object containing `status` and `expires_in_secs` fields.
**Setup:** `~/.claude/.credentials.json` contains an `expiresAt` timestamp set to current time + 7200 seconds.
**Command:** `clp .token.status format::json`
**Expected Output:** Valid JSON on stdout (e.g., `{"status":"valid","expires_in_secs":7199}`), exit 0.
**Verification:**
- Stdout is valid JSON (parseable without error)
- Parsed JSON contains key `status` with string value (`valid`, `expired`, or `expiring_soon`)
- Parsed JSON contains key `expires_in_secs` with integer value
- `expires_in_secs` value is approximately 7200 (within reasonable tolerance for execution time)
- Exit code is 0
**Pass Criteria:** Exit 0; output is valid JSON with `status` and `expires_in_secs` keys.
**Source:** [commands.md — .token.status](../../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-7: Quiet verbosity shows bare status word only

**Goal:** Verify that `v::0` strips all decoration and shows only the bare status word without remaining time.
**Setup:** `~/.claude/.credentials.json` contains an `expiresAt` timestamp set to current time + 7200 seconds.
**Command:** `clp .token.status v::0`
**Expected Output:** `valid` on stdout (bare word, no dash, no remaining time), exit 0.
**Verification:**
- Stdout is exactly one word: `valid` (or `expired` / `expiring soon` depending on token state)
- Stdout does not contain `remaining`
- Stdout does not contain `—`
- Exit code is 0
**Pass Criteria:** Exit 0; output is bare status word with no additional text.
**Source:** [commands.md — .token.status](../../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-8: Missing credentials file exits 2

**Goal:** Verify that when `~/.claude/.credentials.json` does not exist, the command exits 2 with an error.
**Setup:** Ensure `~/.claude/.credentials.json` does not exist (rename or remove it). Account store state is irrelevant.
**Command:** `clp .token.status`
**Expected Output:** Error message on stderr indicating credentials file is unreadable or missing, exit 2.
**Verification:**
- Exit code is 2
- Stderr contains an error message (not empty)
- No output on stdout
**Pass Criteria:** Exit 2; stderr contains error about missing credentials.
**Source:** [commands.md — .token.status](../../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-9: Unparseable `expiresAt` exits 2

**Goal:** Verify that when `~/.claude/.credentials.json` exists but contains an `expiresAt` value that cannot be parsed as a timestamp, the command exits 2.
**Setup:** `~/.claude/.credentials.json` contains valid JSON but with `"expiresAt": "not-a-timestamp"` (a non-numeric, non-ISO string).
**Command:** `clp .token.status`
**Expected Output:** Error message on stderr indicating `expiresAt` is unparseable, exit 2.
**Verification:**
- Exit code is 2
- Stderr contains an error message referencing the parse failure
- No misleading status output on stdout
**Pass Criteria:** Exit 2; stderr contains error about unparseable expiresAt.
**Source:** [commands.md — .token.status](../../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-10: Verbose output shows raw timestamp and threshold

**Goal:** Verify that `v::2` includes the raw `expiresAt` timestamp value and the effective threshold value in the output.
**Setup:** `~/.claude/.credentials.json` contains an `expiresAt` timestamp (e.g., `"expiresAt": "2026-03-22T14:00:00Z"` or a millisecond epoch value).
**Command:** `clp .token.status v::2`
**Expected Output:** Output on stdout containing the status classification, remaining time, raw `expiresAt` value, and the threshold value (default `3600`), exit 0.
**Verification:**
- Stdout contains the raw `expiresAt` value from the credentials file
- Stdout contains the threshold value (`3600` by default)
- Stdout still contains the status classification word
- Exit code is 0
**Pass Criteria:** Exit 0; output includes raw timestamp, threshold value, and status.
**Source:** [commands.md — .token.status](../../../../../docs/cli/commands.md#command--7-tokenstatus)
