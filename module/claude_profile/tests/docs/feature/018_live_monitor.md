# Test: Feature 018 — Live Quota Monitor Mode

Feature behavioral requirement test cases for `docs/feature/018_live_monitor.md`. Each FT case maps to one acceptance criterion. Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/009_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Command IT |
|----|-----------|-----|------------|
| FT-01 | `live::0` (default) — single fetch cycle, exits; behavior unchanged | AC-24 | — |
| FT-02 | `live::1 format::json` exits 1 before any fetch | AC-25 | it24 |
| FT-03 | `live::1 interval::29` exits 1 before any fetch | AC-26 | it23 |
| FT-04 | `live::1 interval::60 jitter::70` exits 1 before any fetch | AC-27 | it22 |
| FT-05 | `live::1` renders table, countdown footer, waits, repeats | AC-28 | it21 |
| FT-06 | Per-account stagger delay of 200–1500 ms in live mode | AC-29 | — |
| FT-07 | Ctrl-C (SIGINT) in live mode exits 0 cleanly | AC-30 | it30 |
| FT-08 | `interval::` and `jitter::` not validated when `live::0` | AC-31 | it28 |
| FT-09 | `live::`, `interval::`, `jitter::` in `.usage --help` with defaults | AC-32 | — |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | `live::0` default — single fetch, exits; no loop overhead | AC-24 | Default Behavior |
| FT-02 | `live::1 format::json` rejected before first fetch | AC-25 | Incompatibility Guard |
| FT-03 | `live::1 interval::29` rejected before first fetch | AC-26 | Interval Validation |
| FT-04 | `live::1 interval::60 jitter::70` rejected before first fetch | AC-27 | Jitter Validation |
| FT-05 | Live loop renders table with countdown footer | AC-28 | Live Loop |
| FT-06 | Per-account fetch preceded by random sleep 200–1500 ms | AC-29 | Stagger Delay |
| FT-07 | SIGINT exits 0 cleanly without error | AC-30 | Signal Handling |
| FT-08 | `interval::` and `jitter::` ignored when `live::0` | AC-31 | Conditional Validation |
| FT-09 | `live::`, `interval::`, `jitter::` in `.usage --help` | AC-32 | Help Output |

**Total:** 9 FT cases

---

### FT-01: `live::0` default — single fetch, exits; no loop overhead

- **Given:** `.usage` environment with valid credentials and at least one saved account.
- **When:** `clp .usage live::0`
- **Then:** Command performs exactly one fetch cycle, renders the table, and exits; no countdown footer, no screen clear, no loop; behavior is identical to the baseline `.usage` with no `live::` param; exit 0.
- **Exit:** 0
- **Source fn:** `f018_ft001_live_0_single_fetch`
- **Source:** [018_live_monitor.md AC-24](../../../docs/feature/018_live_monitor.md)

---

### FT-02: `live::1 format::json` rejected before first fetch

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage live::1 format::json`
- **Then:** Exit 1 before any fetch; stderr contains `live monitor mode is incompatible with format::json`.
- **Exit:** 1
- **Source fn:** `it024_live_incompatible_with_json`
- **Source:** [018_live_monitor.md AC-25](../../../docs/feature/018_live_monitor.md)

---

### FT-03: `live::1 interval::29` rejected before first fetch

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage live::1 interval::29`
- **Then:** Exit 1 before any fetch; stderr contains `interval must be >= 30`.
- **Exit:** 1
- **Source fn:** `it023_live_interval_below_minimum`
- **Source:** [018_live_monitor.md AC-26](../../../docs/feature/018_live_monitor.md)

---

### FT-04: `live::1 interval::60 jitter::70` rejected before first fetch

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage live::1 interval::60 jitter::70`
- **Then:** Exit 1 before any fetch; stderr contains `jitter must not exceed interval`.
- **Exit:** 1
- **Source fn:** `it022_live_jitter_exceeds_interval`
- **Source:** [018_live_monitor.md AC-27](../../../docs/feature/018_live_monitor.md)

---

### FT-05: Live loop renders table with countdown footer

- **Given:** `.usage` environment with valid credentials and at least one saved account.
- **When:** `clp .usage live::1` (interrupted with Ctrl-C after first cycle completes)
- **Then:** stdout (before interrupt) contains the quota table; a countdown footer line matching `Next update in M:SS (at HH:MM:SS UTC)  [Ctrl-C to exit]` is written; the loop begins a second cycle before interrupt; exit 0 on Ctrl-C.
- **Exit:** 0
- **Source fn:** `it021_lim_it_live_mode` [live — requires credentials]
- **Source:** [018_live_monitor.md AC-28](../../../docs/feature/018_live_monitor.md)

---

### FT-06: Per-account fetch preceded by random sleep 200–1500 ms

- **Given:** `.usage` environment with at least one saved account; `trace::1` enabled for timing visibility.
- **When:** `clp .usage live::1 trace::1` (interrupted after first cycle)
- **Then:** trace output on stderr shows per-account fetch steps interleaved with observed delays in the 200–1500 ms range; accounts are not fetched simultaneously (stagger present); exit 0.
- **Exit:** 0
- **Source fn:** `TBD — no dedicated test`
- **Source:** [018_live_monitor.md AC-29](../../../docs/feature/018_live_monitor.md)

---

### FT-07: SIGINT exits 0 cleanly without error

- **Given:** `.usage` environment in `live::1` mode; the command is in the countdown wait phase.
- **When:** SIGINT (Ctrl-C) is sent to the process.
- **Then:** The process exits 0; no panic, no partial table line left on screen; the cursor is restored; the exit is treated as a clean user-initiated stop, not an error.
- **Exit:** 0
- **Source fn:** `it030_live_sigint_exits_0`
- **Source:** [018_live_monitor.md AC-30](../../../docs/feature/018_live_monitor.md)

---

### FT-08: `interval::` and `jitter::` ignored when `live::0`

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage live::0 interval::1 jitter::999`
- **Then:** Command accepted; single fetch and render; invalid `interval::` and `jitter::` values are not validated because `live::0`; exit 0.
- **Exit:** 0
- **Source fn:** `it028_interval_jitter_ignored_when_not_live`
- **Source:** [018_live_monitor.md AC-31](../../../docs/feature/018_live_monitor.md)

---

### FT-09: `live::`, `interval::`, `jitter::` appear in `.usage --help` with defaults

- **Given:** clean environment.
- **When:** `clp .usage --help`
- **Then:** stdout or stderr contains all three of `live::`, `interval::`, `jitter::` with their default values (`0`, `30`, `0` respectively).
- **Exit:** 0
- **Source fn:** `it031_usage_help_shows_live_params`
- **Source:** [018_live_monitor.md AC-32](../../../docs/feature/018_live_monitor.md)
