# Test: `.credentials.status`

Integration test planning for the `.credentials.status` command. See [commands.md](../../../../docs/cli/commands.md#command--11-credentialsstatus) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | No clp credential store — exits 0, default output | Account-Store Independence |
| IT-2 | Default output with `.claude.json` — all 6 default-on fields shown | Field Presence (default) |
| IT-3 | `format::json` — returns parseable JSON with all 8 fields | Output Format |
| IT-4 | Missing `.credentials.json` — exits non-zero with actionable error | Error Handling |
| IT-5 | Default output without `.claude.json` — email and account show N/A | Missing Optional File |
| IT-6 | All default-on fields suppressed — only token line shown | Field Presence (suppress) |
| IT-7 | `file::1 saved::1` — File and Saved lines appended | Field Presence (opt-in) |
| IT-8 | Output is stable across repeated invocations | Stability |

### Test Coverage Summary

- Account-Store Independence: 1 test
- Field Presence (default): 1 test
- Output Format: 1 test
- Error Handling: 1 test
- Missing Optional File: 1 test
- Field Presence (suppress): 1 test
- Field Presence (opt-in): 1 test

**Total:** 8 integration tests

---

### IT-1: No clp credential store — exits 0, default output

- **Given:** Claude Code's `~/.claude/.credentials.json` present (subscriptionType="pro", rateLimitTier="standard", expiresAt=far future). No `clp` credential store created.
- **When:** `clp .credentials.status`
- **Then:** Stdout contains subscription type ("pro") and a token state classification ("valid"), `Account: N/A`, exit 0.; default fields visible including N/A account
- **Exit:** 0
- **Source:** [FR-17](../../../../docs/feature/012_live_credentials_status.md)

---

### IT-2: Default output with `.claude.json` — all 6 default-on fields shown

- **Given:** Claude Code's `~/.claude/.credentials.json` present (subscriptionType="pro", rateLimitTier="standard", expiresAt=far future). Claude Code's `~/.claude/.claude.json` present (emailAddress="user@example.com"). No `clp` credential store.
- **When:** `clp .credentials.status`
- **Then:** Stdout contains all 6 default-on fields (Account, Sub, Tier, Token, Expires, Email), exit 0.; all 6 default-on fields present in output
- **Exit:** 0
- **Source:** [FR-17](../../../../docs/feature/012_live_credentials_status.md)

---

### IT-3: `format::json` — returns parseable JSON with all 8 fields

- **Given:** Claude Code's `~/.claude/.credentials.json` present (subscriptionType="pro", rateLimitTier="standard", expiresAt=far future). Claude Code's `~/.claude/.claude.json` present (emailAddress="user@example.com").
- **When:** `clp .credentials.status format::json`
- **Then:** Valid JSON object on stdout containing all 8 keys, exit 0.; output is valid JSON with all 8 required fields (including opt-in `file` and `saved`)
- **Exit:** 0
- **Source:** [FR-17](../../../../docs/feature/012_live_credentials_status.md)

---

### IT-4: Missing `.credentials.json` — exits non-zero with actionable error

- **Given:** Claude Code's `~/.claude/` directory exists but `.credentials.json` is not present.
- **When:** `clp .credentials.status`
- **Then:** Error message on stderr referencing "credential", exit non-zero.; Exit non-zero; stderr references the missing credential file
- **Exit:** 0
- **Source:** [FR-17](../../../../docs/feature/012_live_credentials_status.md)

---

### IT-5: Default output without `.claude.json` — email and account show N/A

- **Given:** Claude Code's `~/.claude/.credentials.json` present (subscriptionType="pro"). No `~/.claude/.claude.json`. No `clp` credential store (no `_active` marker).
- **When:** `clp .credentials.status`
- **Then:** Stdout shows "N/A" for email and account fields, exit 0.; N/A displayed for missing email and account
- **Exit:** 0
- **Source:** [FR-17](../../../../docs/feature/012_live_credentials_status.md)

---

### IT-6: All default-on fields suppressed — only token line shown

- **Given:** Claude Code's `~/.claude/.credentials.json` present (subscriptionType="pro", rateLimitTier="standard", expiresAt=far future).
- **When:** `clp .credentials.status account::0 sub::0 tier::0 expires::0 email::0`
- **Then:** Stdout contains only the Token line, exit 0.; only Token: line in output, all other default-on lines suppressed
- **Exit:** 0
- **Source:** [FR-17](../../../../docs/feature/012_live_credentials_status.md)

---

### IT-7: `file::1 saved::1` — File and Saved lines appended

- **Given:** Claude Code's `~/.claude/.credentials.json` present (subscriptionType="pro"). `clp` credential store may or may not exist.
- **When:** `clp .credentials.status file::1 saved::1`
- **Then:** Stdout contains the default-on fields plus File and Saved lines, exit 0.; File: and Saved: lines present in output alongside default-on fields
- **Exit:** 0
- **Source:** [FR-17](../../../../docs/feature/012_live_credentials_status.md)

---

### IT-8: Output is stable across repeated invocations

- **Given:** Claude Code's `~/.claude/.credentials.json` present with static credentials
- **When:** `clp .credentials.status` (run 3 times)
- **Then:** All 3 stdout captures are byte-identical
- **Exit:** 0
- **Source:** [commands.md — .credentials.status](../../../../docs/cli/commands.md#command--10-credentialsstatus)
