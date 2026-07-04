# Test: `.tail`

### Scope

- **Purpose**: Verify `.tail` follows journal events in real-time with correct filtering and formatting.
- **Responsibility**: Test case coverage for all 5 `.tail` parameters.
- **In Scope**: Type/command filter, format selection, color toggle, journal_dir override, polling behavior.
- **Out of Scope**: One-shot listing (-> `01_list.md`), aggregate stats (-> `03_stats.md`).

Test case planning for [command/02_tail.md](../../../../docs/cli/command/02_tail.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | No args -> follows all events | Default |
| IT-2 | `type::execution` -> follows execution events only | Type Filter |
| IT-3 | `command::ask format::json` -> follows filtered events as JSON | Combined Filter |
| IT-4 | `no_color::1` -> output has no ANSI escape codes | Display |
| IT-5 | `journal_dir::PATH` -> follows events from custom directory | Directory Override |

## Test Coverage Summary

- Default: 1 test (IT-1)
- Type Filter: 1 test (IT-2)
- Combined Filter: 1 test (IT-3)
- Display: 1 test (IT-4)
- Directory Override: 1 test (IT-5)

**Total:** 5 tests

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
