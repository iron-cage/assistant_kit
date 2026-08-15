# Parameter :: `live::`

Edge case tests for the `live::` parameter. Tests validate boolean enforcement, default-off behavior, `format::json` incompatibility guard, and single-shot vs. continuous-loop modes. Used by `.usage` to enable the live monitor loop.

**Source:** [params.md#parameter--20-live](../../../../docs/cli/param/020_live.md)

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

### EC-1: `live::1` — enters continuous loop mode (verified via SIGKILL after 10s, not Ctrl-C)

- **Given:** A real Anthropic OAuth access token (test skips with a message if `live_active_token()` returns `None`). One saved account `myaccount` carrying that real token.
- **When:** `clp .usage live::1 interval::30 jitter::0`, allowed to run for 10 seconds, then force-killed via `Child::kill()` (SIGKILL) — not interrupted with Ctrl-C/SIGINT.
- **Then:** The raw captured output bytes (captured up to the kill) contain `Next update` — proving the countdown footer was rendered, i.e. the loop was entered. Exit status is never captured or asserted (SIGKILL leaves no clean exit code). Clean SIGINT exit-0 behavior is covered separately by `it030_live_sigint_exits_0`, not this test.
- **Exit:** N/A — process is SIGKILLed; exit status is not captured or asserted
- **Source fn:** `it021_lim_it_live_mode` [live — requires credentials]
- **Source:** [params.md#parameter--20-live](../../../../docs/cli/param/020_live.md)
---

### EC-2: `live::0` — explicit disable accepted; single fetch

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage live::0`
- **Then:** Single fetch and render; exits 0 immediately; no loop, no countdown footer, no screen clear.
- **Exit:** 0
- **Source fn:** `it041_live_0_single_fetch_exits_0`
- **Source:** [params.md#parameter--20-live](../../../../docs/cli/param/020_live.md)
---

### EC-3: `live::1 format::json` rejected before first fetch

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage live::1 format::json`
- **Then:** Exit 1 before any fetch; stderr contains `live monitor mode is incompatible with format::json`.
- **Exit:** 1
- **Source fn:** `it024_live_incompatible_with_json`
- **Source:** [params.md#parameter--20-live](../../../../docs/cli/param/020_live.md)
---

### EC-4: `live::2` rejected

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage live::2`
- **Then:** Exit 1 with error referencing `live::`; must be 0 or 1.
- **Exit:** 1
- **Source fn:** `it042_live_2_rejected`
- **Source:** [params.md#parameter--20-live](../../../../docs/cli/param/020_live.md)
---

### EC-5: `live::yes` rejected

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage live::yes`
- **Then:** Exit 1 with type validation error referencing `live::`.
- **Exit:** 1
- **Source fn:** `it043_live_yes_rejected` (in `tests/cli/usage_live_test.rs`) — renumbered from `it053` when the `it0NN` series shifted by -10
- **Source:** [params.md#parameter--20-live](../../../../docs/cli/param/020_live.md)
---

### EC-6: Default value is `0` (single-shot) — only verified via help-text listing

- **Given:** None — no account or credential setup.
- **When:** `clp .usage.help`
- **Then:** Exits 0. stdout contains the substrings `live`, `interval`, and `jitter` — confirming the three live-monitor params are documented in help output (AC-32). This test does NOT invoke bare `.usage` (no `live::`) and does NOT assert single-fetch/no-loop runtime behavior for the true default — no test in the suite exercises the omitted-`live::` case end-to-end. Closest adjacent evidence: `it041_live_0_single_fetch_exits_0` (EC-2) proves single-fetch behavior for explicit `live::0`, not omission.
- **Exit:** 0
- **Source fn:** `it031_usage_help_shows_live_params`
- **Source:** [params.md#parameter--20-live](../../../../docs/cli/param/020_live.md)
