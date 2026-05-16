# Parameter Group :: System Prompt

Interaction test planning for the System Prompt parameter group. See [param_group.md](../../../../docs/cli/param_group.md#group--3-system-prompt) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `--system-prompt` alone → forwarded; `--append-system-prompt` absent | Single Flag |
| CC-2 | `--append-system-prompt` alone → forwarded; `--system-prompt` absent | Single Flag |
| CC-3 | Both together → both forwarded in parse order | Combined |
| CC-4 | Neither present → claude uses default system prompt (no system-prompt flag injected) | Default Behavior |

## Test Coverage Summary

- Single Flag: 2 tests
- Combined: 1 test
- Default Behavior: 1 test

**Total:** 4 tests

## Test Cases
---

### CC-1: `--system-prompt` alone → forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --system-prompt "Be concise." "test"`
- **Then:** Contains `--system-prompt`; does NOT contain `--append-system-prompt`.; only system-prompt present
- **Exit:** 0
- **Source:** [param_group.md — System Prompt](../../../../docs/cli/param_group.md#group--3-system-prompt)

---

### CC-2: `--append-system-prompt` alone → forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --append-system-prompt "Always JSON." "test"`
- **Then:** Contains `--append-system-prompt`; does NOT contain `--system-prompt`.; only append-system-prompt present
- **Exit:** 0
- **Source:** [param_group.md — System Prompt](../../../../docs/cli/param_group.md#group--3-system-prompt)

---

### CC-3: Both together → both forwarded in parse order

- **Given:** clean environment
- **When:** `clr --dry-run --system-prompt "Base." --append-system-prompt "Extra." "test"`
- **Then:** Contains both `--system-prompt` and `--append-system-prompt`.; both flags present in output
- **Exit:** 0
- **Source:** [param_group.md — System Prompt](../../../../docs/cli/param_group.md#group--3-system-prompt)

---

### CC-4: Neither present → no system-prompt flag injected

- **Given:** clean environment
- **When:** `clr --dry-run "test"`
- **Then:** Does NOT contain `--system-prompt` or `--append-system-prompt`.; no system-prompt flags injected by default
- **Exit:** 0
- **Source:** [--system-prompt default: absent](../../../../docs/cli/param/15_system_prompt.md)
