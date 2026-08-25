# Test: `.tail`

### Scope

- **Purpose**: Verify `.tail` follows journal events in real-time with correct filtering and formatting.
- **Responsibility**: Test case coverage for all 10 `.tail` parameters — seven filters, `format`, and the two global parameters.
- **In Scope**: Type/command filter, format selection, color toggle, journal_dir override, polling behavior, and the two filters `.tail` deliberately does **not** take.
- **Out of Scope**: One-shot listing (-> `01_list.md`), aggregate stats (-> `03_stats.md`).

Test case planning for [command/02_tail.md](../../../../docs/cli/command/02_tail.md).

## Test Case Index

| ID | Test Name | Category | Status | Implemented as |
|----|-----------|----------|--------|----------------|
| IT-1 | No args -> follows all events | Default | ✅ | `viewer_integration_test.rs::ec13_tail_starts_and_can_be_killed` |
| IT-2 | `type::execution` -> follows execution events only | Type Filter | ⏳ | — |
| IT-3 | `command::ask format::json` -> follows filtered events as JSON | Combined Filter | ⏳ | `ec34_tail_format_renders_and_rejects_before_blocking` covers `format::` only |
| IT-4 | `no_color::1` -> output has no ANSI escape codes | Display | ⏳ | — |
| IT-5 | `journal_dir::PATH` -> follows events from custom directory | Directory Override | ✅ | `viewer_integration_test.rs::ec13_tail_starts_and_can_be_killed` |
| IT-6 | `format::` renders each variant; a bad one exits 1 before blocking | Format | ✅ | `viewer_integration_test.rs::ec34_tail_format_renders_and_rejects_before_blocking` |
| IT-7 | `since::` and `limit::` exit 1 rather than being accepted and ignored | Retraction | ✅ | `viewer_integration_test.rs::ec28_unknown_param_exits_1` |

## Test Coverage Summary

- Default: 1 test (IT-1)
- Type Filter: 1 test (IT-2)
- Combined Filter: 1 test (IT-3)
- Display: 1 test (IT-4)
- Directory Override: 1 test (IT-5)
- Format: 1 test (IT-6)
- Retraction: 1 test (IT-7)

**Total:** 7 tests (4 executable)

Every `.tail` case is bounded in wall-clock time in its implementation. `.tail`
blocks forever by design, so the failure mode of any regression here is a hang —
and a hung test reports nothing at all, it just stalls the suite until the
runner's own timeout kills it, naming nothing.

---

### IT-1: No args -> follows all events

- **Given:** journal actively receiving new events
- **When:** `clj .tail`
- **Then:** each new event is printed as it arrives, in table format
- **Exit:** 0 (on interrupt)
- **Source:** [command/02_tail.md](../../../../docs/cli/command/02_tail.md)

---

### IT-2: `type::execution` -> follows execution events only

- **Given:** journal receiving a mix of event types
- **When:** `clj .tail type::execution`
- **Then:** only execution-type events are printed as they arrive
- **Exit:** 0 (on interrupt)
- **Source:** [command/02_tail.md](../../../../docs/cli/command/02_tail.md), [param/03_type.md](../../../../docs/cli/param/03_type.md)

---

### IT-3: `command::ask format::json` -> follows filtered events as JSON

- **Given:** journal receiving both `run` and `ask` command events
- **When:** `clj .tail command::ask format::json`
- **Then:** only `ask` events are printed, each formatted as a single JSON object per line
- **Exit:** 0 (on interrupt)
- **Source:** [command/02_tail.md](../../../../docs/cli/command/02_tail.md), [param/04_command.md](../../../../docs/cli/param/04_command.md), [param/10_format.md](../../../../docs/cli/param/10_format.md)

---

### IT-4: `no_color::1` -> output has no ANSI escape codes

- **Given:** journal actively receiving new events
- **When:** `clj .tail no_color::1`
- **Then:** printed lines contain no ANSI color escape sequences
- **Exit:** 0 (on interrupt)
- **Source:** [command/02_tail.md](../../../../docs/cli/command/02_tail.md), [param/24_no_color.md](../../../../docs/cli/param/24_no_color.md)

---

### IT-5: `journal_dir::PATH` -> follows events from custom directory

- **Given:** a non-default journal directory containing active events
- **When:** `clj .tail journal_dir::/tmp/custom_journal`
- **Then:** events are read and followed from the specified directory instead of the default
- **Exit:** 0 (on interrupt)
- **Source:** [command/02_tail.md](../../../../docs/cli/command/02_tail.md), [param/21_journal_dir.md](../../../../docs/cli/param/21_journal_dir.md)

---

### IT-6: `format::` renders each variant; a bad one exits 1 before blocking

- **Given:** journal with events already written — `tail()` replays the current
  day's file from its start, so nothing needs appending after the spawn
- **When:** `clj .tail format::X` for X in jsonl, json, csv
- **Then:** `jsonl` and `json` each print one complete standalone JSON object per
  line (never `[`, which opens an array a never-ending stream cannot close);
  `csv` prints its header row first
- **And:** `clj .tail format::bogus` exits 1 promptly rather than after an
  indefinite wait for an event that may never arrive
- **Exit:** killed by the caller for the valid formats; 1 for `format::bogus`
- **Source:** [command/02_tail.md](../../../../docs/cli/command/02_tail.md), [param/10_format.md](../../../../docs/cli/param/10_format.md), [type/06_output_format.md](../../../../docs/cli/type/06_output_format.md)

---

### IT-7: `since::` and `limit::` exit 1 rather than being accepted and ignored

- **Given:** any journal directory
- **When:** `clj .tail since::1h`, then `clj .tail limit::5`
- **Then:** each exits **1** with `unknown parameter`, naming the offending key and listing `.tail`'s accepted set
- **Exit:** 1
- **Note:** `.tail` took the whole filter vocabulary until this case. `TailIter` calls `event_matches` with `since_cutoff : None` and never reads `filter.limit`, so both parsed cleanly, applied to nothing, and exited 0 — a filter that silently does not filter is worse than one that is refused
- **Note:** bounded on wall-clock time like every other `.tail` case, and for a sharper reason: if the rejection regresses, the parameter is accepted, `.tail` starts following, and the run never returns. The regression would present as a hung suite naming nothing rather than as this failure
- **And:** run it by hand with `clj .tail since::1h; echo "exit=$?"` — it returns immediately
- **Source:** [command/02_tail.md](../../../../docs/cli/command/02_tail.md), [param/01_since.md](../../../../docs/cli/param/01_since.md), [param/09_limit.md](../../../../docs/cli/param/09_limit.md), [invariant/003_cli_surface_consistency.md](../../invariant/003_cli_surface_consistency.md)
