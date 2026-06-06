# Test: `.credentials.status`

Integration test planning for the `.credentials.status` command. See [command/namespace.md](../../../../docs/cli/command/002_credentials.md#command--10-credentialsstatus) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | No clp credential store — exits 0, default output | Account-Store Independence |
| IT-2 | Default output with `.claude.json` — all 6 default-on fields shown | Field Presence (default) |
| IT-3 | `format::json` — returns parseable JSON with all 12 fields | Output Format |
| IT-4 | Missing `.credentials.json` — exits non-zero with actionable error | Error Handling |
| IT-5 | Default output without `.claude.json` — email and account show N/A | Missing Optional File |
| IT-6 | All default-on fields suppressed — only token line shown | Field Presence (suppress) |
| IT-7 | `file::1 saved::1` — File and Saved lines appended | Field Presence (opt-in) |
| IT-8 | Output is stable across repeated invocations | Stability |
| IT-9 | `uuid::1` shows `ID:` line from live `~/.claude.json` `taggedId` | Extended Fields (opt-in) |
| IT-10 | `capabilities::1` shows `Capabilities:` as comma-separated list from live `~/.claude.json` | Extended Fields (opt-in) |
| IT-11 | `org_uuid::1` shows `Org ID:` from `{active_account}.json` in credential store | Org Identity (opt-in) |
| IT-12 | `org_name::1` shows `Org:` from `{active_account}.json` in credential store | Org Identity (opt-in) |
| IT-13 | `uuid::`, `capabilities::`, `org_uuid::`, `org_name::` all absent by default | Extended Fields / Default |
| IT-14 | `format::json` includes `tagged_id`, `capabilities`, `organization_uuid`, `organization_name` keys | Extended Fields / JSON |

### Test Coverage Summary

- Account-Store Independence: 1 test
- Field Presence (default): 1 test
- Output Format: 1 test
- Error Handling: 1 test
- Missing Optional File: 1 test
- Field Presence (suppress): 1 test
- Field Presence (opt-in): 1 test
- Stability: 1 test
- Extended Fields (opt-in): 2 tests (IT-9, IT-10)
- Org Identity (opt-in): 2 tests (IT-11, IT-12)
- Extended Fields / Default: 1 test (IT-13)
- Extended Fields / JSON: 1 test (IT-14)

**Total:** 14 integration tests

---

### IT-1: No clp credential store — exits 0, default output

- **Given:** Claude Code's `~/.claude/.credentials.json` present (subscriptionType="pro", rateLimitTier="standard", expiresAt=far future). No `clp` credential store created.
- **When:** `clp .credentials.status`
- **Then:** Stdout contains subscription type ("pro") and a token state classification ("valid"), `Account: N/A`, exit 0.; default fields visible including N/A account
- **Exit:** 0
- **Source:** [FR-17](../../../../docs/feature/012_live_credentials_status.md)

---

### IT-2: Default output with `.claude.json` — all 6 default-on fields shown

- **Given:** Claude Code's `~/.claude/.credentials.json` present (subscriptionType="pro", rateLimitTier="standard", expiresAt=far future). Claude Code's `~/.claude.json` present (emailAddress="user@example.com"). No `clp` credential store.
- **When:** `clp .credentials.status`
- **Then:** Stdout contains all 6 default-on fields (Account, Sub, Tier, Token, Expires, Email), exit 0.; all 6 default-on fields present in output
- **Exit:** 0
- **Source:** [FR-17](../../../../docs/feature/012_live_credentials_status.md)

---

### IT-3: `format::json` — returns parseable JSON with all 16 fields

- **Given:** Claude Code's `~/.claude/.credentials.json` present (subscriptionType="pro", rateLimitTier="standard", expiresAt=far future). Claude Code's `~/.claude.json` present (emailAddress="user@example.com").
- **When:** `clp .credentials.status format::json`
- **Then:** Valid JSON object on stdout containing all 16 keys: subscription, tier, token, expires_in_secs, email, account, file, saved, display_name, role, billing, model, tagged_id, capabilities, organization_uuid, organization_name; exit 0.
- **Exit:** 0
- **Source:** [FR-17](../../../../docs/feature/012_live_credentials_status.md)

---

### IT-4: Missing `.credentials.json` — exits non-zero with actionable error

- **Given:** Claude Code's `~/.claude/` directory exists but `.credentials.json` is not present.
- **When:** `clp .credentials.status`
- **Then:** Error message on stderr referencing "credential", exit non-zero.; Exit non-zero; stderr references the missing credential file
- **Exit:** 2
- **Source:** [FR-17](../../../../docs/feature/012_live_credentials_status.md)

---

### IT-5: Default output without `.claude.json` — email and account show N/A

- **Given:** Claude Code's `~/.claude/.credentials.json` present (subscriptionType="pro"). No `~/.claude.json`. No `clp` credential store (no per-machine active marker).
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
- **Source:** [command/002_credentials.md — .credentials.status](../../../../docs/cli/command/002_credentials.md#command--10-credentialsstatus)

---

### IT-9: `uuid::1` shows `ID:` line from live `~/.claude.json`

- **Given:** `~/.claude/.credentials.json` present. `~/.claude.json` present and contains `{"oauthAccount":{"taggedId":"user_abc123"}}`.
- **When:** `clp .credentials.status uuid::1`
- **Then:** Stdout contains `ID:      user_abc123`.
- **Exit:** 0
- **Source:** [021_extended_snapshot_fields.md AC-01](../../../../docs/feature/021_extended_snapshot_fields.md)

---

### IT-10: `capabilities::1` shows `Capabilities:` as comma-separated list

- **Given:** `~/.claude/.credentials.json` present. `~/.claude.json` present and contains `{"oauthAccount":{"capabilities":["claude_code","pro"]}}`.
- **When:** `clp .credentials.status capabilities::1`
- **Then:** Stdout contains `Capabilities: claude_code, pro`.
- **Exit:** 0
- **Source:** [021_extended_snapshot_fields.md AC-02](../../../../docs/feature/021_extended_snapshot_fields.md)

---

### IT-11: `org_uuid::1` shows `Org ID:` from `{active_account}.json`

- **Given:** `~/.claude/.credentials.json` present. Credential store has per-machine active marker pointing to `work@acme.com` and `{credential_store}/work@acme.com.json` containing `{"organization_uuid":"org-xyz-789","organization_name":"Acme Corp"}`.
- **When:** `clp .credentials.status org_uuid::1`
- **Then:** Stdout contains `Org ID:  org-xyz-789`.
- **Exit:** 0
- **Source:** [022_org_identity_snapshot.md AC-07](../../../../docs/feature/022_org_identity_snapshot.md)

---

### IT-12: `org_name::1` shows `Org:` from `{active_account}.json`

- **Given:** Same setup as IT-11 (active account with org identity fields in `{active_account}.json` in credential store).
- **When:** `clp .credentials.status org_name::1`
- **Then:** Stdout contains `Org:     Acme Corp`.
- **Exit:** 0
- **Source:** [022_org_identity_snapshot.md AC-08](../../../../docs/feature/022_org_identity_snapshot.md)

---

### IT-13: Extended params absent by default

- **Given:** `~/.claude/.credentials.json` present. `~/.claude.json` contains taggedId and capabilities. Credential store has per-machine active marker with `{active_account}.json` containing org fields.
- **When:** `clp .credentials.status` (no extended params)
- **Then:** Stdout does NOT contain `ID:`, `Capabilities:`, `Org ID:`, or `Org:` lines. Only default-on fields shown.
- **Exit:** 0
- **Source:** [021_extended_snapshot_fields.md AC-05](../../../../docs/feature/021_extended_snapshot_fields.md), [022_org_identity_snapshot.md](../../../../docs/feature/022_org_identity_snapshot.md) Design §New field-presence params

---

### IT-14: `format::json` includes extended and org fields

- **Given:** `~/.claude/.credentials.json` present. `~/.claude.json` contains `taggedId="user_abc"` and `capabilities=["claude_code"]`. Credential store has active account with `{active_account}.json` containing `organization_uuid="org-xyz"` and `organization_name="Acme"`.
- **When:** `clp .credentials.status format::json`
- **Then:** Valid JSON object containing `tagged_id`, `capabilities`, `organization_uuid`, `organization_name` keys; `tagged_id` = `"user_abc"`, `capabilities` = `["claude_code"]`, `organization_uuid` = `"org-xyz"`, `organization_name` = `"Acme"`.
- **Exit:** 0
- **Source:** [021_extended_snapshot_fields.md AC-06](../../../../docs/feature/021_extended_snapshot_fields.md), [022_org_identity_snapshot.md AC-09](../../../../docs/feature/022_org_identity_snapshot.md)
