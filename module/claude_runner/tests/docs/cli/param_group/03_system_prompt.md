# Test: System Prompt (Group 3)

Interaction test planning for the System Prompt parameter group. See [parameter_groups.md](../../../../docs/cli/parameter_groups.md#group--3-system-prompt) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--system-prompt` alone → forwarded; `--append-system-prompt` absent | Single Flag |
| EC-2 | `--append-system-prompt` alone → forwarded; `--system-prompt` absent | Single Flag |
| EC-3 | Both together → both forwarded in parse order | Combined |
| EC-4 | Neither present → claude uses default system prompt (no system-prompt flag injected) | Default Behavior |

## Test Coverage Summary

- Single Flag: 2 tests
- Combined: 1 test
- Default Behavior: 1 test

**Total:** 4 tests

---

### EC-1: `--system-prompt` alone → forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --system-prompt "Be concise." "test"`
- **Then:** Contains `--system-prompt`; does NOT contain `--append-system-prompt`.; only system-prompt present
- **Exit:** 0
- **Source:** [parameter_groups.md — System Prompt](../../../../docs/cli/parameter_groups.md#group--3-system-prompt)

---

### EC-2: `--append-system-prompt` alone → forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --append-system-prompt "Always JSON." "test"`
- **Then:** Contains `--append-system-prompt`; does NOT contain `--system-prompt`.; only append-system-prompt present
- **Exit:** 0
- **Source:** [parameter_groups.md — System Prompt](../../../../docs/cli/parameter_groups.md#group--3-system-prompt)

---

### EC-3: Both together → both forwarded in parse order

- **Given:** clean environment
- **When:** `clr --dry-run --system-prompt "Base." --append-system-prompt "Extra." "test"`
- **Then:** Contains both `--system-prompt` and `--append-system-prompt`.; both flags present in output
- **Exit:** 0
- **Source:** [parameter_interactions.md — system prompt combinations](../../../../docs/cli/parameter_interactions.md)

---

### EC-4: Neither present → no system-prompt flag injected

- **Given:** clean environment
- **When:** `clr --dry-run "test"`
- **Then:** Does NOT contain `--system-prompt` or `--append-system-prompt`.; no system-prompt flags injected by default
- **Exit:** 0
- **Source:** [params.md — --system-prompt default: absent](../../../../docs/cli/params.md#parameter--15---system-prompt)
