# Test: `.token.status`

Integration test planning for the `.token.status` command. See [commands.md](../../../../docs/cli/commands.md#command--7-tokenstatus) for specification.

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

- **Given:** `~/.claude/.credentials.json` contains an `expiresAt` timestamp set to current time + 7200 seconds (2 hours from now). Default threshold is 3600 seconds.
- **When:** `clp .token.status`
- **Then:** Output on stdout matching pattern `valid — *remaining` (e.g., `valid — 1h59m remaining`), exit 0.; output contains "valid" and remaining time
- **Exit:** 0
- **Source:** [commands.md — .token.status](../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-2: Expired token shows "expired"

- **Given:** `~/.claude/.credentials.json` contains an `expiresAt` timestamp set to current time - 3600 seconds (1 hour ago).
- **When:** `clp .token.status`
- **Then:** Output on stdout containing `expired`, exit 0.; output contains "expired"
- **Exit:** 0
- **Source:** [commands.md — .token.status](../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-3: ExpiringSoon token shows "expiring soon" with remaining time

- **Given:** `~/.claude/.credentials.json` contains an `expiresAt` timestamp set to current time + 1800 seconds (30 minutes from now). Default threshold is 3600 seconds, so 1800 < 3600 triggers ExpiringSoon.
- **When:** `clp .token.status`
- **Then:** Output on stdout matching pattern `expiring soon — *remaining` (e.g., `expiring soon — 29m remaining`), exit 0.; output contains "expiring soon" and remaining time
- **Exit:** 0
- **Source:** [commands.md — .token.status](../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-4: Custom threshold changes classification boundary

- **Given:** `~/.claude/.credentials.json` contains an `expiresAt` timestamp set to current time + 2500 seconds. With default threshold (3600), this would be "expiring soon". With threshold 1800, it should be "valid".
- **When:** `clp .token.status threshold::1800`
- **Then:** Output on stdout containing `valid`, exit 0.; output contains "valid" under custom threshold
- **Exit:** 0
- **Source:** [commands.md — .token.status](../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-5: Threshold zero never classifies as ExpiringSoon

- **Given:** `~/.claude/.credentials.json` contains an `expiresAt` timestamp set to current time + 60 seconds (1 minute remaining). With default threshold (3600), this would be "expiring soon".
- **When:** `clp .token.status threshold::0`
- **Then:** Output on stdout containing `valid`, exit 0.; output contains "valid" even with very little time remaining
- **Exit:** 0
- **Source:** [commands.md — .token.status](../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-6: JSON format returns structured object

- **Given:** `~/.claude/.credentials.json` contains an `expiresAt` timestamp set to current time + 7200 seconds.
- **When:** `clp .token.status format::json`
- **Then:** Valid JSON on stdout (e.g., `{"status":"valid","expires_in_secs":7199}`), exit 0.; output is valid JSON with `status` and `expires_in_secs` keys
- **Exit:** 0
- **Source:** [commands.md — .token.status](../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-7: Quiet verbosity shows bare status word only

- **Given:** `~/.claude/.credentials.json` contains an `expiresAt` timestamp set to current time + 7200 seconds.
- **When:** `clp .token.status v::0`
- **Then:** `valid` on stdout (bare word, no dash, no remaining time), exit 0.; output is bare status word with no additional text
- **Exit:** 0
- **Source:** [commands.md — .token.status](../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-8: Missing credentials file exits 2

- **Given:** Ensure `~/.claude/.credentials.json` does not exist (rename or remove it). Account store state is irrelevant.
- **When:** `clp .token.status`
- **Then:** Error message on stderr indicating credentials file is unreadable or missing, exit 2.; stderr contains error about missing credentials
- **Exit:** 2
- **Source:** [commands.md — .token.status](../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-9: Unparseable `expiresAt` exits 2

- **Given:** `~/.claude/.credentials.json` contains valid JSON but with `"expiresAt": "not-a-timestamp"` (a non-numeric, non-ISO string).
- **When:** `clp .token.status`
- **Then:** Error message on stderr indicating `expiresAt` is unparseable, exit 2.; stderr contains error about unparseable expiresAt
- **Exit:** 2
- **Source:** [commands.md — .token.status](../../../../docs/cli/commands.md#command--7-tokenstatus)

---

### IT-10: Verbose output shows raw timestamp and threshold

- **Given:** `~/.claude/.credentials.json` contains an `expiresAt` timestamp (e.g., `"expiresAt": "2026-03-22T14:00:00Z"` or a millisecond epoch value).
- **When:** `clp .token.status v::2`
- **Then:** Output on stdout containing the status classification, remaining time, raw `expiresAt` value, and the threshold value (default `3600`), exit 0.; output includes raw timestamp, threshold value, and status
- **Exit:** 0
- **Source:** [commands.md — .token.status](../../../../docs/cli/commands.md#command--7-tokenstatus)
