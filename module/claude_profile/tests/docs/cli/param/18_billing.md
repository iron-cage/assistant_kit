# Parameter :: `billing::`

Edge case tests for the `billing::` parameter. Tests validate boolean enforcement, default behavior, and billing type field control from `oauthAccount`. Used by `.credentials.status` (live `~/.claude.json`) and `.accounts` (saved `{name}.claude.json` snapshot).

**Source:** [params.md#parameter--18-billing](../../../../docs/cli/params.md#parameter--18-billing)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `billing::1` shows `Billing:` line with billingType value | Field Control |
| EC-2 | `billing::2` rejected (out of range) | Boundary Values |
| EC-3 | `billing::yes` rejected (type validation) | Type Validation |
| EC-4 | Default value is `0` (absent by default, opt-in) | Default |
| EC-5 | `billing::0` explicit disable accepted — field absent | Field Control |
| EC-6 | `format::json` always emits `billing` key regardless of `billing::0` | Interaction |

## Test Coverage Summary

- Field Control: 2 tests (EC-1, EC-5)
- Boundary Values: 1 test (EC-2)
- Type Validation: 1 test (EC-3)
- Default: 1 test (EC-4)
- Interaction: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (opt-in enabled) ↔ EC-4 (absent by default)

## Test Cases
---

### EC-1: `billing::1` — Billing: field included in output

- **Given:** `~/.claude.json` contains `oauthAccount.billingType = "stripe_subscription"`
- **When:** `clp .credentials.status billing::1`
- **Then:** Output contains `Billing:` line with value `stripe_subscription`
- **Exit:** 0
- **Source:** [params.md#parameter--18-billing](../../../../docs/cli/params.md#parameter--18-billing)
---

### EC-2: `billing::2` rejected

- **Given:** clean environment with valid credentials
- **When:** `clp .credentials.status billing::2`
- **Then:** Exit 1 with error referencing `billing::`; must be 0 or 1
- **Exit:** 1
- **Source:** [params.md#parameter--18-billing](../../../../docs/cli/params.md#parameter--18-billing)
---

### EC-3: `billing::yes` rejected

- **Given:** clean environment with valid credentials
- **When:** `clp .credentials.status billing::yes`
- **Then:** Exit 1 with type validation error referencing `billing::`
- **Exit:** 1
- **Source:** [params.md#parameter--18-billing](../../../../docs/cli/params.md#parameter--18-billing)
---

### EC-4: Default value (absent by default, opt-in)

- **Given:** `~/.claude.json` contains `oauthAccount.billingType = "stripe_subscription"`
- **When:** `clp .credentials.status` (no `billing::` param)
- **Then:** `Billing:` line absent from output; billingType not exposed unless opted in
- **Exit:** 0
- **Source:** [params.md#parameter--18-billing](../../../../docs/cli/params.md#parameter--18-billing)
---

### EC-5: `billing::0` explicit disable accepted — field absent

- **Given:** `~/.claude.json` contains `oauthAccount.billingType = "stripe_subscription"`
- **When:** `clp .credentials.status billing::0`
- **Then:** `Billing:` line absent from output; explicit 0 same as default-off
- **Exit:** 0
- **Source:** [params.md#parameter--18-billing](../../../../docs/cli/params.md#parameter--18-billing)
---

### EC-6: `format::json` always emits `billing` key

- **Given:** `~/.claude.json` contains `oauthAccount.billingType = "stripe_subscription"`
- **When:** `clp .credentials.status format::json billing::0`
- **Then:** JSON output contains `"billing"` key with value `"stripe_subscription"` regardless of `billing::0`
- **Exit:** 0
- **Source:** [params.md#parameter--18-billing](../../../../docs/cli/params.md#parameter--18-billing)
