# Parameter Group :: Settings Identity

Interaction tests for Group 3 (Settings Identity): `key::` and `value::`. Tests validate required pairing, read vs. write semantics, and error behavior when one is missing.

**Source:** [parameter_groups.md#group--3-settings-identity](../../../../docs/cli/parameter_groups.md#group--3-settings-identity)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `key::` + `value::` together → settings written | Happy Path |
| EC-2 | `key::` without `value::` → exit 1 on `.settings.set` | Missing Value |
| EC-3 | `key::` alone → value read on `.settings.get` | Read Mode |
| EC-4 | Both present on `.settings.get` → `value::` ignored (read-only) | Interaction |

## Test Coverage Summary

- Happy Path: 1 test (EC-1)
- Missing Value: 1 test (EC-2)
- Read Mode: 1 test (EC-3)
- Interaction: 1 test (EC-4)

**Total:** 4 edge cases

## Test Cases
---

### EC-1: `key::` + `value::` → settings written:

- **Given:** clean environment
- **When:** `cm .settings.set key::theme value::dark`
- **Then:** Setting `theme=dark` written; exit 0
- **Exit:** 0
- **Source:** [parameter_groups.md#group--3-settings-identity](../../../../docs/cli/parameter_groups.md#group--3-settings-identity)
---

### EC-2: `key::` without `value::` on `.settings.set` → exit 1:

- **Given:** clean environment
- **When:** `cm .settings.set key::theme`
- **Then:** Exit 1; error indicating `value::` is required for `.settings.set`
- **Exit:** 1
- **Source:** [parameter_groups.md#group--3-settings-identity](../../../../docs/cli/parameter_groups.md#group--3-settings-identity)
---

### EC-3: `key::` alone on `.settings.get` → value read:

- **Given:** clean environment with `theme` key set
- **When:** `cm .settings.get key::theme`
- **Then:** Current value of `theme` printed; exit 0
- **Exit:** 0
- **Source:** [parameter_groups.md#group--3-settings-identity](../../../../docs/cli/parameter_groups.md#group--3-settings-identity)
---

### EC-4: Both on `.settings.get` → `value::` ignored:

- **Given:** clean environment
- **When:** `cm .settings.get key::theme value::dark`
- **Then:** Current setting value read and printed; `value::dark` has no effect on get
- **Exit:** 0
- **Source:** [parameter_groups.md#group--3-settings-identity](../../../../docs/cli/parameter_groups.md#group--3-settings-identity)
