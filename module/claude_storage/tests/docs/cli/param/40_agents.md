# Parameter :: `agents::`

Edge case tests for the `agents::` parameter. Tests validate `.cost`'s agent fold-in toggle — whether a conversation's row aggregates its family's agent sessions or reports the root alone — across both storage layouts, plus Boolean validation.

**Not a reuse of [`agent::`](01_agent.md):** that parameter is a session-*type filter* on `.list`/`.projects`. Here `0`/`1` means exclude/include subordinate sessions in an aggregate — a different semantic, which is why the plural name exists.

**Source:** [param/40_agents.md](../../../../docs/cli/param/40_agents.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Default `agents::1` folds the family's agent sessions in | Default |
| EC-2 | `agents::0` reports the root session alone | Happy Path |
| EC-3 | Invalid `agents::` value rejected | Input Validation |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Happy Path: 1 test (EC-2)
- Input Validation: 1 test (EC-3)

**Total:** 3 edge cases

**Behavioral Divergence Pair:** EC-1 (`1` — agent tokens included, `Agents` counts the folded files) ↔ EC-2 (`0` — root only, `Agents` shows `0`)

## Test Cases

---

### EC-1: Default `agents::1` folds the family's agent sessions in

- **Commands:** `.cost`
- **Given:** a conversation with agent sessions present in both supported layouts — hierarchical `subagents/` and flat `agent-*.jsonl`
- **When:** `clg .cost` with no `agents::`
- **Then:** the row's totals include every agent session in the family per the [Session Family invariant](../../../../docs/invariant/002_session_family.md), and the `Agents` column counts the folded files
- **Exit:** 0
- **Covered by:** `cli_cmd_cost_test.rs` — `cost_int_6_agents_folded_by_default`
- **Source:** [param/40_agents.md](../../../../docs/cli/param/40_agents.md)

---

### EC-2: `agents::0` reports the root session alone

- **Commands:** `.cost`
- **Given:** the same family fixture as EC-1
- **When:** `clg .cost agents::0`
- **Then:** the row covers the root session only and `Agents` shows `0` — answering "how much was the main thread, versus the agents it spawned?"
- **Exit:** 0
- **Covered by:** `cli_cmd_cost_test.rs` — `cost_int_7_agents_zero_root_only`
- **Source:** [param/40_agents.md](../../../../docs/cli/param/40_agents.md)

---

### EC-3: Invalid `agents::` value rejected

- **Commands:** `.cost`
- **Given:** clean environment
- **When:** `clg .cost agents::<not 0 or 1>`
- **Then:** Exit 1; error indicating `agents` must be `0` or `1` — validated before any storage access, since having a default does not exempt a parameter from explicit validation
- **Exit:** 1
- **Covered by:** `cli_cmd_cost_test.rs` — `cost_int_8_agents_invalid_rejected`
- **Source:** [param/40_agents.md](../../../../docs/cli/param/40_agents.md)
