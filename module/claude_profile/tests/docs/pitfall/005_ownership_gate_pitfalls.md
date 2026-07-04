# Pitfall Tests: Ownership Gate Pitfalls

Test cases verifying that each guard documented in `docs/pitfall/005_ownership_gate_pitfalls.md`
is in place and prevents the described ownership gate failure mode.

**Source:** [docs/pitfall/005_ownership_gate_pitfalls.md](../../../docs/pitfall/005_ownership_gate_pitfalls.md)
**Case prefix:** `PP-` (Pitfall Protection)

### Pitfall Guard Index

| ID | Pitfall | Bug | Guard Verified By |
|----|---------|-----|-------------------|
| PP-1 | Refresh G2 gate: `!is_owned OR is_occupied_elsewhere` (not just `!is_owned`) | BUG-303 | `mre_bug306_refresh_trace_reason_occupied_elsewhere` |
| PP-2 | Touch G4 gate: `!is_owned OR is_occupied_elsewhere` (not just `!is_owned`) | BUG-302 | `ft_touch_skips_occupied_elsewhere_with_trace` |
| PP-3 | Fetch G1b gate: occupied-elsewhere → approximate, not live fetch | BUG-305 | `ft23_g1_non_owned_applies_approximation` in fetch_tests.rs |
| PP-4 | `reason_label()` covers occupied-elsewhere trace path | BUG-306 | `mre_bug306_refresh_trace_reason_occupied_elsewhere` |

---

### PP-1: Refresh gate (G2) must check BOTH `!is_owned` AND `is_occupied_elsewhere`

- **Given:** An account that is owned by this machine (`is_owned = true`) but is actively
  in use on another machine (`is_occupied_elsewhere = true`).
- **When:** `should_refresh(aq)` (G2) is evaluated.
- **Then:** Refresh is skipped — `is_occupied_elsewhere = true` triggers the skip path
  regardless of `is_owned`. Fix BUG-303: the pre-fix gate only checked `!is_owned`, allowing
  refreshes that overwrote live session tokens on the other machine.
- **Rule:** All operation gates (refresh, touch, fetch) must guard: `!is_owned OR
  is_occupied_elsewhere`. Missing `is_occupied_elsewhere` allows competing with a remote
  session that could invalidate its tokens mid-session.
- **Source fn:** `mre_bug306_refresh_trace_reason_occupied_elsewhere` in
  `tests/usage/refresh_tests_b.rs`
- **Source:** [pitfall/005_ownership_gate_pitfalls.md §P1](../../../docs/pitfall/005_ownership_gate_pitfalls.md)

---

### PP-2: Touch gate (G4) must check BOTH `!is_owned` AND `is_occupied_elsewhere`

- **Given:** An account that is owned by this machine but is actively occupied on another
  machine.
- **When:** `apply_touch` (G4) is evaluated.
- **Then:** Touch subprocess is skipped — `is_occupied_elsewhere = true` triggers the skip.
  Fix BUG-302: the pre-fix gate checked only `!is_owned`, allowing a touch subprocess that
  competes with the remote session.
- **Rule:** Same as PP-1 — both conditions required for all operation gates.
- **Source fn:** `ft_touch_skips_occupied_elsewhere_with_trace` in
  `tests/usage/touch_tests.rs`; `ft07_touch_skips_non_owned_with_trace` tests the
  `!is_owned` half
- **Source:** [pitfall/005_ownership_gate_pitfalls.md §P2](../../../docs/pitfall/005_ownership_gate_pitfalls.md)

---

### PP-3: Fetch gate (G1b) routes occupied-elsewhere to approximation

- **Given:** An account with `is_owned = true` but `is_occupied_elsewhere = true`.
- **When:** `fetch_all_quota` runs (G1b gate).
- **Then:** Live `fetch_oauth_usage()` is NOT called for the occupied account. Instead,
  `approximate_quota()` is used. Fix BUG-305: the pre-fix fetch gate had no explicit
  occupied-elsewhere path for owned accounts; they proceeded to live fetch and competed
  with the remote session's HTTP traffic.
- **Rule:** The occupied-elsewhere check must appear explicitly for owned accounts, not just
  for non-owned accounts.
- **Source fn:** `ft23_g1_non_owned_applies_approximation` in `tests/usage/fetch_tests.rs`
  (covers the approximation path for occupied/non-owned accounts)
- **Source:** [pitfall/005_ownership_gate_pitfalls.md §P3](../../../docs/pitfall/005_ownership_gate_pitfalls.md)

---

### PP-4: `reason_label()` correctly identifies occupied-elsewhere skip reason

- **Given:** `apply_refresh` encounters an account that is occupied on another machine.
- **When:** The skip reason is computed via `reason_label(aq)`.
- **Then:** The emitted trace string contains `"occupied elsewhere"` — not a generic
  `"not_owned"` or silent skip. Fix BUG-306: the original `reason_label()` had no
  occupied-elsewhere branch; the trace always showed `"not_owned"` even for occupied
  accounts, hiding the true skip reason.
- **Rule:** Each distinct skip reason must have its own `reason_label()` branch. Generic
  labels obscure which guard fired and make debugging harder.
- **Source fn:** `mre_bug306_refresh_trace_reason_occupied_elsewhere`,
  `reason_label_not_owned` in `tests/usage/refresh_tests_b.rs`
- **Source:** [pitfall/005_ownership_gate_pitfalls.md §P4](../../../docs/pitfall/005_ownership_gate_pitfalls.md)
