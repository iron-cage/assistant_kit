# Test: Feature 018 — Live Quota Monitor Mode

### Scope

- **Purpose**: Test cases for live quota monitor mode.
- **Source**: `docs/feature/018_live_monitor.md`
- **Covers**: AC-24 through AC-32

Feature behavioral requirement test cases for `docs/feature/018_live_monitor.md`. Each FT case maps to one acceptance criterion. Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/09_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Command IT |
|----|-----------|-----|------------|
| FT-01 | `live::0` (default) — single fetch cycle, exits; behavior unchanged | AC-24 | — |
| FT-02 | `live::1 format::json` exits 1 before any fetch | AC-25 | it24 |
| FT-03 | `live::1 interval::5` exits 1 before any fetch | AC-26 | it23 |
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
| FT-03 | `live::1 interval::5` rejected before first fetch | AC-26 | Interval Validation |
| FT-04 | `live::1 interval::60 jitter::70` rejected before first fetch | AC-27 | Jitter Validation |
| FT-05 | Live loop renders table with countdown footer | AC-28 | Live Loop |
| FT-06 | Per-account fetch preceded by random sleep 200–1500 ms | AC-29 | Stagger Delay |
| FT-07 | SIGINT exits 0 cleanly without error | AC-30 | Signal Handling |
| FT-08 | `interval::` and `jitter::` ignored when `live::0` | AC-31 | Conditional Validation |
| FT-09 | `live::`, `interval::`, `jitter::` in `.usage --help` | AC-32 | Help Output |

**Total:** 9 FT cases

---

### FT-01: `live::0` default — single fetch, exits; no loop overhead

- **Given:** `.usage` environment with at least one saved account.
- **When:** `clp .usage live::0`
- **Then:** Command performs exactly one fetch cycle, renders the table, and exits; no countdown footer, no screen clear, no loop; behavior is identical to the baseline `.usage` with no `live::` param; exit 0.
- **Exit:** 0
- **Source fn:** `f18_ft01_live_0_single_fetch` (`usage_feature_test.rs`) — renamed from `f018_ft001_live_0_single_fetch`
- **Note:** The cited test deliberately uses a no-token account (no `accessToken` field) so the fetch fails instantly with no HTTP call — not "valid credentials" as originally stated here. The row rendered is an error row, not live quota data. This is orthogonal to the claim under test (single-cycle exit, no footer, no loop), which the test verifies precisely regardless of fetch outcome.
- **Source:** [018_live_monitor.md AC-24](../../../docs/feature/018_live_monitor.md)

---

### FT-02: `live::1 format::json` rejected before first fetch

- **Given:** clean environment (no accounts required — the guard fires before any account/credential lookup).
- **When:** `clp .usage live::1 format::json`
- **Then:** Exit 1 before any fetch; stderr is non-empty. Per direct source inspection (`src/usage/api.rs:97`), the emitted message is exactly `live monitor mode is incompatible with format::json`, though the cited test only asserts stderr is non-empty, not this literal text.
- **Exit:** 1
- **Source fn:** `it024_live_incompatible_with_json`
- **Source:** [018_live_monitor.md AC-25](../../../docs/feature/018_live_monitor.md)

---

### FT-03: `live::1 interval::5` rejected before first fetch

- **Given:** clean environment (no accounts required — the guard fires before any account/credential lookup).
- **When:** `clp .usage live::1 interval::5 jitter::0`
- **Then:** Exit 1 before any fetch; stderr contains `30`. Per direct source inspection (`src/usage/api.rs:104`), the emitted message is exactly `interval must be >= 30`. Corrected from a previously-cited `interval::29` fixture value, which the cited test does not use (confirmed against `tests/docs/cli/command/09_usage.md § IT-24` and `tests/docs/cli/param/21_interval.md § EC-2`, which both correctly cite `interval::5` for this same test).
- **Exit:** 1
- **Source fn:** `it023_live_interval_below_minimum`
- **Source:** [018_live_monitor.md AC-26](../../../docs/feature/018_live_monitor.md)

---

### FT-04: `live::1 interval::60 jitter::70` rejected before first fetch

- **Given:** clean environment (no accounts required — the guard fires before any account/credential lookup).
- **When:** `clp .usage live::1 interval::60 jitter::70`
- **Then:** Exit 1 before any fetch; stderr is non-empty. Per direct source inspection (`src/usage/api.rs:111`), the emitted message is exactly `jitter must not exceed interval`, though the cited test only asserts stderr is non-empty, not this literal text.
- **Exit:** 1
- **Source fn:** `it022_live_jitter_exceeds_interval`
- **Source:** [018_live_monitor.md AC-27](../../../docs/feature/018_live_monitor.md)

---

### FT-05: Live loop renders table with countdown footer

- **Given:** `.usage` environment with valid credentials and at least one saved account.
- **When:** `clp .usage live::1 interval::30 jitter::0`, captured for a fixed 10-second window then force-killed.
- **Then:** stdout captured within the 10-second window contains a countdown footer — the test asserts the substring `Next update`. Per direct source inspection (`src/usage/live.rs:133`), the full line format is `  Next update in {m}:{s:02} (at {next_hms} UTC)  [Ctrl-C to exit]`.
- **Exit:** N/A — the cited test terminates the process via `Child::kill()` (SIGKILL), not Ctrl-C/SIGINT, and does not assert an exit code at all (`child.wait()`'s result is discarded). It does not verify "a second cycle begins" or "exit 0 on Ctrl-C" — SIGINT clean-exit is covered separately by FT-07/AC-30.
- **Source fn:** `it021_lim_it_live_mode` [live — requires credentials]
- **Source:** [018_live_monitor.md AC-28](../../../docs/feature/018_live_monitor.md)

---

### FT-06: Per-account fetch preceded by random sleep 200–1500 ms

- **Given:** `.usage` environment with at least one saved account; `trace::1` enabled for timing visibility.
- **When:** `clp .usage live::1 trace::1` (interrupted after first cycle)
- **Then:** trace output on stderr shows per-account fetch steps interleaved with observed delays in the 200–1500 ms range; accounts are not fetched simultaneously (stagger present); exit 0.
- **Exit:** 0
- **Source fn:** `f18_ft06_live_stagger_per_account_trace`
- **Source:** [018_live_monitor.md AC-29](../../../docs/feature/018_live_monitor.md)

---

### FT-07: SIGINT exits 0 cleanly without error

- **Given:** `.usage` environment in `live::1` mode; the command is in the countdown wait phase.
- **When:** SIGINT (Ctrl-C) is sent to the process.
- **Then:** The process exits 0; stdout contains `Monitor stopped.` — the exit is treated as a clean user-initiated stop, not an error.
- **Exit:** 0
- **Source fn:** `it030_live_sigint_exits_0`
- **Note:** Corrected — `src/usage/live.rs` implements no cursor-hide/show ANSI sequence at all (verified via direct source inspection), so there is no cursor state to "restore"; that claim was inaccurate and has been removed. "No partial table line left on screen" is untested by the cited test (or any other) and not a guarantee made by the implementation; also removed. The cited test's actual distinguishing assertion — stdout contains `Monitor stopped.` — is now reflected above.
- **Source:** [018_live_monitor.md AC-30](../../../docs/feature/018_live_monitor.md)

---

### FT-08: `interval::` and `jitter::` ignored when `live::0`

- **Given:** Empty credential store (no accounts saved); `live::0` is the default (not passed explicitly).
- **When:** `clp .usage interval::5 jitter::70` — values chosen so `jitter::70 > interval::5` would fail the live-mode guard if `live::1` were set.
- **Then:** Command accepted; exit 0; stdout contains `no accounts` (empty store — there is nothing to fetch). The interval/jitter guards do not fire, proving they are not validated when live mode is inactive; this test does not exercise the fetch-and-render path.
- **Exit:** 0
- **Source fn:** `it028_interval_jitter_ignored_when_not_live`
- **Note:** Corrected — the cited test uses `interval::5 jitter::70` (not `interval::1 jitter::999`) against an empty store, and asserts the `no accounts` message, not "single fetch and render" of quota data.
- **Source:** [018_live_monitor.md AC-31](../../../docs/feature/018_live_monitor.md)

---

### FT-09: `live::`, `interval::`, `jitter::` appear in `.usage --help` with defaults

- **Given:** clean environment.
- **When:** `clp .usage --help`
- **Then:** stdout or stderr contains all three of `live::`, `interval::`, `jitter::` with their default values (`0`, `30`, `0` respectively).
- **Exit:** 0
- **Source fn:** `it031_usage_help_shows_live_params`
- **Source:** [018_live_monitor.md AC-32](../../../docs/feature/018_live_monitor.md)
