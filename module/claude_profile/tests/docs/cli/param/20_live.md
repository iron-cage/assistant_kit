# Parameter :: `live::`

Edge case tests for the `live::` parameter. Tests validate boolean enforcement, default-off behavior, `format::json` incompatibility guard, and single-shot vs. continuous-loop modes. Used by `.usage` to enable the live monitor loop.

**Source:** [params.md#parameter--20-live](../../../../docs/cli/param/20_live.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `live::1` accepted — enters continuous loop mode | Loop Enabled |
| EC-2 | `live::0` accepted — single fetch and exit | Single-shot |
| EC-3 | `live::1 format::json` rejected before first fetch | Incompatibility |
| EC-4 | `live::2` rejected (out of range) | Boundary Values |
| EC-5 | `live::yes` rejected (type validation) | Type Validation |
| EC-6 | Default value is `0` (single-shot) | Default |

## Test Coverage Summary

- Loop Enabled: 1 test (EC-1)
- Single-shot: 1 test (EC-2)
- Incompatibility: 1 test (EC-3)
- Boundary Values: 1 test (EC-4)
- Type Validation: 1 test (EC-5)
- Default: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (loop enabled — continuous refresh) ↔ EC-6 (absent by default — single fetch)

## Test Cases
---

### EC-1: `live::1` — enters continuous loop mode

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage live::1` (interrupted with Ctrl-C after first cycle)
- **Then:** Command clears screen, renders table, displays countdown footer in format `Next update in M:SS (at HH:MM:SS UTC)  [Ctrl-C to exit]`, then waits; exits 0 on Ctrl-C.
- **Exit:** 0
- **Source fn:** `it21_lim_it_live_mode` [live — requires credentials]
- **Source:** [params.md#parameter--20-live](../../../../docs/cli/param/20_live.md)
---

### EC-2: `live::0` — explicit disable accepted; single fetch

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage live::0`
- **Then:** Single fetch and render; exits 0 immediately; no loop, no countdown footer, no screen clear.
- **Exit:** 0
- **Source fn:** `it41_live_0_single_fetch_exits_0`
- **Source:** [params.md#parameter--20-live](../../../../docs/cli/param/20_live.md)
---

### EC-3: `live::1 format::json` rejected before first fetch

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage live::1 format::json`
- **Then:** Exit 1 before any fetch; stderr contains `live monitor mode is incompatible with format::json`.
- **Exit:** 1
- **Source fn:** `it24_live_incompatible_with_json`
- **Source:** [params.md#parameter--20-live](../../../../docs/cli/param/20_live.md)
---

### EC-4: `live::2` rejected

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage live::2`
- **Then:** Exit 1 with error referencing `live::`; must be 0 or 1.
- **Exit:** 1
- **Source fn:** `it42_live_2_rejected`
- **Source:** [params.md#parameter--20-live](../../../../docs/cli/param/20_live.md)
---

### EC-5: `live::yes` rejected

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage live::yes`
- **Then:** Exit 1 with type validation error referencing `live::`.
- **Exit:** 1
- **Source fn:** `it43_live_yes_rejected`
- **Source:** [params.md#parameter--20-live](../../../../docs/cli/param/20_live.md)
---

### EC-6: Default value is `0` (single-shot)

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage` (no `live::` param)
- **Then:** Single fetch and render; exits 0; behavior identical to `live::0`; no loop or countdown footer.
- **Exit:** 0
- **Source fn:** `it31_usage_help_shows_live_params`
- **Source:** [params.md#parameter--20-live](../../../../docs/cli/param/20_live.md)
