# Parameter :: `capabilities::`

Edge case tests for the `capabilities::` parameter. Tests validate boolean enforcement, default behavior, and product capabilities list control from `oauthAccount.capabilities`. Used by `.credentials.status` (live `~/.claude.json`) and `.accounts` (saved `{name}.json` snapshot).

**Source:** [params.md#parameter--29-capabilities](../../../../docs/cli/param/029_capabilities.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `capabilities::1` shows `Capabilities:` line as comma-separated list | Field Control |
| EC-2 | `capabilities::2` rejected (out of range) | Boundary Values |
| EC-3 | `capabilities::yes` rejected (type validation) | Type Validation |
| EC-4 | Default value is `0` (absent by default, opt-in) | Default |
| EC-5 | `capabilities::0` explicit disable accepted — field absent | Field Control |
| EC-6 | `format::json` always emits `capabilities` key regardless of `capabilities::0` | Interaction |
| EC-7 | Empty capabilities array → `Capabilities: N/A` | Edge Case |
| EC-8 | Missing capabilities field in `.claude.json` → `Capabilities: N/A` | Edge Case |

## Test Coverage Summary

- Field Control: 2 tests (EC-1, EC-5)
- Boundary Values: 1 test (EC-2)
- Type Validation: 1 test (EC-3)
- Default: 1 test (EC-4)
- Interaction: 1 test (EC-6)
- Edge Case: 2 tests (EC-7, EC-8)

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-1 (opt-in enabled) ↔ EC-4 (absent by default)

## Test Cases
---

### EC-1: `capabilities::1` — Capabilities: field included as comma-separated list

- **Given:** `~/.claude.json` contains `{"oauthAccount":{"capabilities":["claude_code","pro"]}}`
- **When:** `clp .credentials.status capabilities::1`
- **Then:** Output contains `Capabilities:` line with value `claude_code, pro`
- **Exit:** 0
- **Source:** [params.md#parameter--29-capabilities](../../../../docs/cli/param/029_capabilities.md)
---

### EC-2: `capabilities::2` rejected

- **Given:** clean environment with valid credentials
- **When:** `clp .credentials.status capabilities::2`
- **Then:** Exit 1 with error referencing `capabilities::`; must be 0 or 1
- **Exit:** 1
- **Source:** [params.md#parameter--29-capabilities](../../../../docs/cli/param/029_capabilities.md)
---

### EC-3: `capabilities::yes` rejected

- **Given:** clean environment with valid credentials
- **When:** `clp .credentials.status capabilities::yes`
- **Then:** Exit 1 with type validation error referencing `capabilities::`
- **Exit:** 1
- **Source:** [params.md#parameter--29-capabilities](../../../../docs/cli/param/029_capabilities.md)
---

### EC-4: Default value (absent by default, opt-in)

- **Given:** `~/.claude.json` contains `{"oauthAccount":{"capabilities":["claude_code","pro"]}}`
- **When:** `clp .credentials.status` (no `capabilities::` param)
- **Then:** `Capabilities:` line absent from output; capabilities list not exposed unless opted in
- **Exit:** 0
- **Source:** [params.md#parameter--29-capabilities](../../../../docs/cli/param/029_capabilities.md)
---

### EC-5: `capabilities::0` explicit disable accepted — field absent

- **Given:** `~/.claude.json` contains `{"oauthAccount":{"capabilities":["claude_code","pro"]}}`
- **When:** `clp .credentials.status capabilities::0`
- **Then:** `Capabilities:` line absent from output; explicit 0 same as default-off
- **Exit:** 0
- **Source:** [params.md#parameter--29-capabilities](../../../../docs/cli/param/029_capabilities.md)
---

### EC-6: `format::json` always emits `capabilities` key

- **Given:** `~/.claude.json` contains `{"oauthAccount":{"capabilities":["claude_code"]}}`
- **When:** `clp .credentials.status format::json capabilities::0`
- **Then:** JSON output contains `"capabilities"` key with value `["claude_code"]` regardless of `capabilities::0`
- **Exit:** 0
- **Source:** [params.md#parameter--29-capabilities](../../../../docs/cli/param/029_capabilities.md)
---

### EC-7: Empty capabilities array → `Capabilities: N/A`

- **Given:** `~/.claude.json` contains `{"oauthAccount":{"capabilities":[]}}`
- **When:** `clp .credentials.status capabilities::1`
- **Then:** Output contains `Capabilities: N/A`; empty array treated same as absent
- **Exit:** 0
- **Source:** [params.md#parameter--29-capabilities](../../../../docs/cli/param/029_capabilities.md)
---

### EC-8: Missing capabilities field → `Capabilities: N/A`

- **Given:** `~/.claude.json` present but `oauthAccount.capabilities` absent
- **When:** `clp .credentials.status capabilities::1`
- **Then:** Output contains `Capabilities: N/A`
- **Exit:** 0
- **Source:** [params.md#parameter--29-capabilities](../../../../docs/cli/param/029_capabilities.md)
