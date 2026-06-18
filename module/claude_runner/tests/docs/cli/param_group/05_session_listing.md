# Parameter Group :: Session Listing

Test case spec for [05_session_listing.md](../../../../docs/cli/param_group/05_session_listing.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| G5-CC1 | All 3 params consumed by `dispatch_ps()` — none affect subprocess execution | Consumption Pattern |
| G5-CC2 | `--mode`, `--columns`, `--wide` accepted by `clr ps` without error | Acceptance |
| G5-CC3 | `clr ps --mode print --columns pid,task --wide` → `--columns` wins over `--wide`; mode filter applied | Interaction |
| G5-CC4 | None of the 3 Session Listing params appear in `clr run --help` output | Exclusivity |
| G5-CC5 | `--mode` and `--columns` env vars (`CLR_PS_MODE`, `CLR_PS_COLUMNS`) respected by `dispatch_ps()` | Env Var |

## Test Coverage Summary

- Consumption Pattern: 1 test (G5-CC1)
- Acceptance: 1 test (G5-CC2)
- Interaction: 1 test (G5-CC3)
- Exclusivity: 1 test (G5-CC4)
- Env Var: 1 test (G5-CC5)

**Total:** 5 tests

---

### G5-CC1: Params consumed by `dispatch_ps()` only

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps --mode all --columns pid,task`
- **Expected behavior:** Exit 0; params control table output without affecting any subprocess; no subprocess is spawned
- **Exit:** 0
- **Source:** [05_session_listing.md](../../../../docs/cli/param_group/05_session_listing.md)
- **Note:** `ps` is a read-only inspection command — it never spawns a `claude` subprocess

---

### G5-CC2: All 3 params accepted without error

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps --mode all --columns pid,task --wide`
- **Expected behavior:** Exit 0; no error on stderr about unknown flags (but `--columns` overrides `--wide`)
- **Exit:** 0
- **Source:** [05_session_listing.md](../../../../docs/cli/param_group/05_session_listing.md)

---

### G5-CC3: `--columns` wins over `--wide` with mode filter

- **Setup:** Spawn 2 fake `claude` processes: one print-mode, one interactive
- **Command:** `clr ps --mode print --columns pid,task --wide`
- **Expected behavior:** Exit 0; only print-mode session shown; only `PID` and `Task` columns visible (not all 11)
- **Exit:** 0
- **Source:** [05_session_listing.md](../../../../docs/cli/param_group/05_session_listing.md)

---

### G5-CC4: Session Listing params not in `clr run --help`

- **Command:** `clr run --help` (or `clr --help`)
- **Expected behavior:** Exit 0; stdout does NOT contain `--mode`, `--columns`, or `--wide` (these are ps-only)
- **Exit:** 0
- **Source:** [05_session_listing.md](../../../../docs/cli/param_group/05_session_listing.md)
- **Note:** Verifies semantic coherence — Session Listing params are exclusive to `clr ps`

---

### G5-CC5: Env var fallbacks respected

- **Setup:** ≥1 fake `claude` process running
- **Command:** `clr ps` with `CLR_PS_MODE=interactive` and `CLR_PS_COLUMNS=pid,elapsed` in env
- **Expected behavior:** Exit 0; only interactive sessions shown; only `PID` and `Elapsed` columns visible
- **Exit:** 0
- **Source:** [05_session_listing.md](../../../../docs/cli/param_group/05_session_listing.md)
