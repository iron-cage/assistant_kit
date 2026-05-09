# Parameter :: `model::`

Edge case tests for the `model::` parameter. Tests validate boolean enforcement, default behavior, and active model field control from settings. Used by `.credentials.status` (live `~/.claude/settings.json`) and `.accounts` (saved `{name}.settings.json` snapshot).

**Source:** [params.md#parameter--19-model](../../../../docs/cli/params.md#parameter--19-model)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `model::1` shows `Model:` line with model value from settings.json | Field Control |
| EC-2 | `model::2` rejected (out of range) | Boundary Values |
| EC-3 | `model::yes` rejected (type validation) | Type Validation |
| EC-4 | Default value is `0` (absent by default, opt-in) | Default |
| EC-5 | `model::0` explicit disable accepted — field absent | Field Control |
| EC-6 | `format::json` always emits `model` key regardless of `model::0` | Interaction |

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

### EC-1: `model::1` — Model: field included in output

- **Given:** `~/.claude/settings.json` contains `"model": "sonnet"`
- **When:** `clp .credentials.status model::1`
- **Then:** Output contains `Model:` line with value `sonnet`
- **Exit:** 0
- **Source:** [params.md#parameter--19-model](../../../../docs/cli/params.md#parameter--19-model)
---

### EC-2: `model::2` rejected

- **Given:** clean environment with valid credentials
- **When:** `clp .credentials.status model::2`
- **Then:** Exit 1 with error referencing `model::`; must be 0 or 1
- **Exit:** 1
- **Source:** [params.md#parameter--19-model](../../../../docs/cli/params.md#parameter--19-model)
---

### EC-3: `model::yes` rejected

- **Given:** clean environment with valid credentials
- **When:** `clp .credentials.status model::yes`
- **Then:** Exit 1 with type validation error referencing `model::`
- **Exit:** 1
- **Source:** [params.md#parameter--19-model](../../../../docs/cli/params.md#parameter--19-model)
---

### EC-4: Default value (absent by default, opt-in)

- **Given:** `~/.claude/settings.json` contains `"model": "sonnet"`
- **When:** `clp .credentials.status` (no `model::` param)
- **Then:** `Model:` line absent from output; active model not exposed unless opted in
- **Exit:** 0
- **Source:** [params.md#parameter--19-model](../../../../docs/cli/params.md#parameter--19-model)
---

### EC-5: `model::0` explicit disable accepted — field absent

- **Given:** `~/.claude/settings.json` contains `"model": "sonnet"`
- **When:** `clp .credentials.status model::0`
- **Then:** `Model:` line absent from output; explicit 0 same as default-off
- **Exit:** 0
- **Source:** [params.md#parameter--19-model](../../../../docs/cli/params.md#parameter--19-model)
---

### EC-6: `format::json` always emits `model` key

- **Given:** `~/.claude/settings.json` contains `"model": "sonnet"`
- **When:** `clp .credentials.status format::json model::0`
- **Then:** JSON output contains `"model"` key with value `"sonnet"` regardless of `model::0`
- **Exit:** 0
- **Source:** [params.md#parameter--19-model](../../../../docs/cli/params.md#parameter--19-model)
