# Invariant: Slot-Wait Message Differentiation

### Scope

- **Purpose**: Ensure the operator-facing slot-wait diagnostic names which of the three independent non-admission causes fired, so an operator can distinguish a genuine capacity problem from transient reservation contention from an unrelated live session simply holding the slot.
- **Responsibility**: State the three distinguishable non-admission causes at the admission condition in `wait_for_session_slot()`, the message-layer contract for encoding which cause fired, and the boundary with `012_gate_slot_atomicity.md` (which governs admission correctness, not message content).
- **In Scope**: `wait_for_session_slot()`'s poll-loop diagnostic message construction (`gate.rs`), the `has_capacity` boolean and the `SlotDenialCause` enum returned by `acquire_slot()` and their joint role in message differentiation, the `[at capacity]` / `[slot held by another session]` / `[lost reservation race]` message suffixes and their mapping to the admission condition's false-branches.
- **Out of Scope**: Admission condition correctness and atomicity (→ `012_gate_slot_atomicity.md`), gate-timeout exhaustion message at `max_attempts` (→ `006_exit_codes.md`), gate poll interval / attempt-limit configuration (→ `cli/003_env_param.md` Env Param 5).

### Invariant Statement

When `wait_for_session_slot()` does not admit a candidate on a given poll attempt and `!quiet`, the `eprintln!` diagnostic MUST encode which of the three independent non-admission causes produced it. A message built only from counters shared across all three (`count`, `max`, `poll_secs`, `attempt`, `max_attempts`) MUST NOT be emitted without an additional field naming the specific cause. For the two slot-side causes (`HeldByLive`, `LostReclaimRace`) the diagnostic MUST additionally carry the occupancy the denying sweep measured (`slots=H/M`; see Measured occupancy below) — naming the cause without quantifying it leaves the failing conjunct of the admission condition invisible (BUG-480).

| Condition | `has_capacity` | `acquire_slot()` result | Message content |
|-----------|----------------|--------------------------|------------------|
| `count_u32 >= max` (no slot numerically free anywhere) | `false` | not attempted | `... [at capacity]` |
| `count_u32 < max`, this index's recorded owner is alive | `true` | `Err(SlotDenialCause::HeldByLive)` | `... slots=H/M ... [slot held by another session]` |
| `count_u32 < max`, recorded owner dead, lost the atomic reclaim-ticket race to another concurrent reclaimer | `true` | `Err(SlotDenialCause::LostReclaimRace)` | `... slots=H/M ... [lost reservation race]` |

**Distinguishing fields:** `has_capacity` is computed once, before the short-circuited `&&`, and separates exhaustion from the other two causes. Within the `has_capacity=true` branch, `acquire_slot()`'s `SlotDenialCause` return value (BUG-396) is the second field required to separate "an unrelated live session already holds this index" — not a race at all, since nothing was contended — from "the recorded owner was dead and I lost the reclaim-ticket race" — the only one of the three causes that is genuinely a race. Both fields must be captured at admission-check time; neither survives past that point unless deliberately threaded through to the message site.

**Measured occupancy (BUG-480):** Admission is a conjunction — census (`count_u32 < max`) AND slot-CAS (`acquire_slot()` succeeding at some index). Within the `has_capacity=true` branch, the acquisition sweep tallies every denied index into `denied_slots` (the count-derived primary index plus each failed fallback candidate), and every slot-side denial diagnostic must interpolate that tally: the poll line carries ` slots={denied_slots}/{max}` between `active=N/M` and `attempt=`, and both exhaustion messages (`gate-wait budget exhausted — ...` and `session gate timed out — ...`) carry `, slots={denied_slots}/{max} held`. At-capacity lines (`claim == None`) are exempt and unchanged: the sweep never ran on those attempts, so `denied_slots` is unmeasured there — interpolating a fabricated `slots=0/M` would misreport zero occupancy on the one cause where occupancy was never probed. Rationale: a denial message interpolating only the census conjunct's counters misattributes the denial — production job #424 emitted 66 consecutive `active=1/8` polls ("7 free") while 8/8 slot files were held, the actual blocker appearing on no surface.

**Preserved prefix (TSK-452):** Every gate-wait diagnostic line begins with the structured prefix `"gate-wait  active="` (double space between `gate-wait` and `active=`; TSK-452 replaced the pre-TSK-452 `"active; waiting"` body). The differentiating cause appears in the `(reason: ...)` trailer at the end of the same line. Assertions in `tests/config_file_test.rs` T01–T05, `tests/concurrency_gate_test.rs` T01/T04, and `tests/concurrency_gate_ext2_test.rs` T27/T32/T34 all assert the new prefix.

### Boundary With Invariant 012 (Gate Slot Atomicity)

This invariant is deliberately narrow: it governs **what the message says**, not **whether admission is correct**. `012_gate_slot_atomicity.md`'s own Condition table already documents these non-admission outcomes as legitimate, correctly-arbitrated results ("falls to wait-and-retry exactly as the `>= max` case does") — that document is authoritative for *why* each outcome is correct admission behavior. This invariant does not restate or challenge that; it adds the orthogonal requirement that all three already-correct outcomes must be distinguishable in the operator-facing text describing them.

<!-- BUG-480 task/claude_runner/bug/480_gate_diagnostic_hides_slot_occupancy.md — fixed: slot-side denial diagnostics now carry the measured occupancy (slots=H/M on the poll line, slots=H/M held on both exhaustion messages), at-capacity lines exempt; see Invariant Statement : Measured occupancy and Provenance : BUG-480 -->
### Enforcement Mechanism

In `src/cli/gate.rs`, `acquire_slot()` returns a typed cause and `wait_for_session_slot()` applies the differentiation as follows:

```rust
enum SlotDenialCause
{
  /// The recorded owner of this index is alive — no reservation was
  /// contested; the slot is simply in active use for however long that
  /// session runs.
  HeldByLive,
  /// The recorded owner was dead, but another concurrent caller won the
  /// atomic reclaim-ticket race for this same index first.
  LostReclaimRace,
}

fn acquire_slot( dir : &Path, index : u32, pid : u32, since : u64 ) -> Result< (), SlotDenialCause >
{
  // ... Ok(()) on successful claim or successful dead-owner reclaim;
  // Err(HeldByLive) when the recorded owner is alive (including the
  // narrow window where a just-claimed record exists but is not yet
  // readable — see Fix(BUG-396) comment at the call site for why an
  // unreadable record means "alive", never "stale");
  // Err(LostReclaimRace) only when the recorded owner was dead and this
  // caller lost the atomic reclaim-ticket race to another reclaimer.
}

// Fix(BUG-393): distinguish global exhaustion (no slot numerically free)
// from every other non-admission cause — both previously produced
// byte-identical text since the message only interpolated the count/max
// counters shared across every false-branch of the compound admission
// condition above.
// Fix(BUG-396): acquire_slot() previously returned a bare `bool`, so the
// `has_capacity=true` branch could not further distinguish "an unrelated
// live session already holds this index" (not a race — confirmed via
// production evidence, job #40, to be the overwhelmingly common case)
// from "the recorded owner was dead and I lost the reclaim-ticket race"
// (the only genuinely race-shaped cause). Both previously collapsed to
// the single label "lost reservation race", which is factually wrong for
// the live-hold case.
let has_capacity = count_u32 < max;
// Fix(BUG-480): tally denied indices beside the sweep's surviving Result —
// the blocking quantity (slot occupancy) must survive to the message sites,
// or every denial line interpolates only census-side counters.
let mut denied_slots : u32 = 0;
let claim = if has_capacity
{
  let mut result = acquire_slot( &dir, count_u32, pid, since );
  if result.is_err()
  {
    denied_slots += 1;
    for candidate in 0..max
    {
      if candidate == count_u32 { continue; }
      result = acquire_slot( &dir, candidate, pid, since );
      if result.is_ok() { break; }
      denied_slots += 1;
    }
  }
  Some( result )
}
else { None };
if let Some( Ok( () ) ) = claim
{
  // ... admitted, return ...
}
// ...
if !quiet
{
  let cause = match claim
  {
    Some( Err( SlotDenialCause::HeldByLive ) )       => "slot held by another session",
    Some( Err( SlotDenialCause::LostReclaimRace ) )  => "lost reservation race",
    // None: has_capacity was false. Some(Ok(())): unreachable (admitted
    // branch already returned above) but required for match exhaustiveness —
    // shares this arm rather than duplicating it (clippy::match_same_arms).
    None | Some( Ok( () ) )                          => "at capacity",
  };
  // Fix(BUG-480): measured-sweep lines carry the occupancy tally; the
  // at-capacity arm stays fieldless — denied_slots is unmeasured there.
  let slots_field = match claim
  {
    Some( Err( _ ) ) => format!( " slots={denied_slots}/{max}" ),
    _ => String::new(),
  };
  eprintln!(
    "{}gate-wait  active={display_count}/{max}{slots_field} attempt={attempt}/{effective_max_attempts} wait={poll_secs}s (reason: {cause})",
    claude_core::trace_ts()
  );
}
```

`has_capacity` is bound before the admission check (short-circuit semantics preserved — `acquire_slot()` is still only called when `has_capacity` is `true`) and reused at the message site rather than recomputed, so the message always reflects the same evaluation that drove the admission decision on this attempt. `claim` similarly captures `acquire_slot()`'s full typed result once and reuses it at the message site. `denied_slots` is tallied inside the same sweep that produced `claim`, so the occupancy the message names is the occupancy this attempt actually measured — never a separate re-probe that could disagree with the admission decision it describes. The exhaustion messages (`gate-wait budget exhausted`, `session gate timed out`) apply the identical `claim`-gated conditional with the variant suffix `, slots={denied_slots}/{max} held`.

### Violation Consequences

If the message reverts to a 2-way (or 1-way) distinction — e.g. collapsing `HeldByLive` and `LostReclaimRace` back into a single "lost reservation race" label, as BUG-393's original fix did — or drops the measured occupancy field from slot-side denial lines (BUG-480):

- An operator reading the wait diagnostic cannot tell "the gate is genuinely full, more sessions must finish" (exhaustion — may warrant a `--max-sessions` increase) from "an unrelated live session already holds this specific slot index, will resolve whenever that session ends — no capacity change needed, no race occurred" (`HeldByLive`) from "the previous owner was dead and I lost the reclaim-ticket race, will likely succeed on the very next attempt" (`LostReclaimRace` — transient, self-resolving on a short timescale). Production evidence (job #40, BUG-396) showed the `HeldByLive` case mislabeled as "lost reservation race" at 4/6 sessions active — an operator correctly recognized 4 < 6 could not be a capacity or race condition and flagged the message as factually wrong.
- T15 (`t15_slot_wait_message_names_live_hold_when_owner_alive` in `tests/concurrency_gate_ext_test.rs`) fails — it asserts exactly one racer's captured stderr names `"slot held by another session"` and neither racer's stderr names `"at capacity"` or `"lost reservation race"`.
- T16 (`t16_slot_wait_message_names_genuine_reclaim_race_for_dead_owner` in `tests/concurrency_gate_ext_test.rs`) fails — it pre-seeds a confirmed-dead owner and asserts the losing racer's first poll attempt names `"lost reservation race"`, proving that label still fires for the one cause it is actually accurate for.
- T38 (`t38_slot_side_denial_names_measured_occupancy` in `tests/concurrency_gate_test.rs`) fails — it seeds the sole slot's record with a live owner and an empty census, then asserts the poll line carries both `active=0/1` and `slots=1/1` and the timeout message carries `slots=1/1 held`; dropping the field (or the census/occupancy divergence it exposes) breaks its assertions.
- The regression is silent at the admission-correctness layer: `012_gate_slot_atomicity.md`'s own tests (T08, T14) continue to pass unchanged, because they assert on admitted-count bounds, not message content — this invariant's own tests (T15, T16) are the only coverage that would catch a reversion.

### Sources

| File | Notes |
|------|-------|
| `../../src/cli/gate.rs` | `SlotDenialCause` enum, `acquire_slot()`'s typed return, and `wait_for_session_slot()`'s `has_capacity` binding plus the differentiated `eprintln!` at the poll-loop's non-admission fall-through |

### Tests

| File | Notes |
|------|-------|
| `../../tests/concurrency_gate_ext_test.rs` | T15: races exactly 2 concurrent `clr` invocations against `--max-sessions 1` with 0 pre-existing occupiers; the loser observes the winner's (still-live, possibly zombie-via-/proc) slot record and its stderr must name `"slot held by another session"`, never `"at capacity"` or `"lost reservation race"` |
| `../../tests/concurrency_gate_ext_test.rs` | T16: pre-seeds a confirmed-dead owner (spawned and reaped) for the sole slot, then races 2 callers to reclaim it with an injected reclaim delay (`CLR_GATE_RECLAIM_TEST_DELAY_MS`); the loser's first poll attempt must name `"lost reservation race"` — the one scenario where that label is accurate |
| `../../tests/concurrency_gate_test.rs` | T38: seeds the sole slot's record with a live owner (the test's own PID) and an empty census, so census says `0/1` while occupancy is `1/1`; asserts the poll line carries both `active=0/1` and `slots=1/1` and the exhaustion message carries `slots=1/1 held` — cause label and measured blocking quantity must appear together (BUG-480) |

### Provenance

| Source | Notes |
|--------|-------|
| BUG-393 | Root bug: `eprintln!` at `gate.rs:368-373` (pre-fix line numbers) interpolated only shared counters, producing byte-identical text for both false-branches of the compound admission condition at `gate.rs:334`. Fix: `has_capacity` bound before the check and threaded into the message as a `[lost reservation race]` / `[at capacity]` suffix appended after the pre-existing `"active; waiting"` text. |
| BUG-396 | Follow-on bug: BUG-393's fix only distinguished exhaustion from "everything else" and mislabeled the entire `has_capacity=true` bucket "lost reservation race" — factually wrong for the dominant real-world case (an unrelated live session already holding the slot; confirmed via production job #40 evidence and live `/tmp/clr-gate/` + `/proc/{pid}` inspection). Fix: `acquire_slot()` now returns `Result<(), SlotDenialCause>` with `HeldByLive` / `LostReclaimRace` variants instead of a bare `bool`, and the message site matches on the full 3-way `(has_capacity, SlotDenialCause)` space instead of `has_capacity` alone. |
| BUG-480 | Fixed: with cause labels correct (BUG-393/396), the diagnostics still interpolated only census-side counters — production job #424 emitted 66 consecutive `gate-wait active=1/8 ... [slot held by another session]` polls ("7 free") while 8/8 slot files were held; the failing conjunct's measured value appeared on no surface, and `clr ps` compounded the misreading with 80 phantom `Queued` rows (BUG-479). Fix: the acquisition sweep tallies denied indices into `denied_slots`, the poll line interpolates ` slots={denied_slots}/{max}` and both exhaustion messages `, slots={denied_slots}/{max} held` — measured sweeps only, at-capacity lines exempt (sweep never ran; tally unmeasured). Regression test T38 was empirically confirmed to fail pre-fix (denial line carried no `slots=` token) and pass post-fix. See `task/claude_runner/bug/480_gate_diagnostic_hides_slot_occupancy.md`. |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/012_gate_slot_atomicity.md](012_gate_slot_atomicity.md) | Governs admission correctness for the same two false-branches this invariant's message must distinguish; that document is authoritative for why both outcomes are correct, this one for how they must be reported |

### Features

| File | Relationship |
|------|--------------|
| [feature/001_runner_tool.md](../feature/001_runner_tool.md) | Defines the print-mode execution path whose concurrency gate emits this diagnostic |
