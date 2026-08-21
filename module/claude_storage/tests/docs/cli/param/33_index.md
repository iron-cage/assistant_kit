# Parameter :: `index::`

Edge case tests for the `index::` parameter on `.show`. Tests validate the 1-based single-message selector — boundaries, error handling, and composition with `last::`, `show_entries::`, and `fields::`.

**Source:** [param/33_index.md](../../../../docs/cli/param/33_index.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `index::N` shows only the Nth message, normal chat-log format | Behavior |
| EC-2 | `index::1` selects the first message | Boundary |
| EC-3 | `index::` at the last valid position selects the last message | Boundary |
| EC-4 | `index::0` rejected | Error Handling |
| EC-5 | Negative `index::` rejected | Error Handling |
| EC-6 | Out-of-range `index::` rejected, error names the actual count | Error Handling |
| EC-7 | Counts within the `last::`-windowed slice in project-overview, not the full session | Composition |
| EC-8 | Composed with `show_entries::1` narrows the raw list to one line | Composition |
| EC-9 | Composed with `fields::` projects one message's requested attributes | Composition |
| EC-10 | Omitted `index::` leaves every in-scope message shown, unchanged | Default |

## Test Coverage Summary

- Behavior: 1 test (EC-1)
- Boundary: 2 tests (EC-2, EC-3)
- Error Handling: 3 tests (EC-4, EC-5, EC-6)
- Composition: 3 tests (EC-7, EC-8, EC-9)
- Default: 1 test (EC-10)

**Total:** 10 edge cases

## Test Cases

---

### EC-1: `index::N` shows only the Nth message, normal chat-log format

- **Commands:** `.show`
- **Given:** session with 4 known entries, each with distinguishable content
- **When:** `clg .show session_id::ID index::2`
- **Then:** stdout shows only the 2nd entry's chat-log content; entries 1, 3, 4 absent entirely
- **Exit:** 0
- **Source:** [param/33_index.md](../../../../docs/cli/param/33_index.md); same test as [command/03_show.md INT-17](../command/03_show.md) (`int_17_index_narrows_session_detail_one_message`)

---

### EC-2: `index::1` selects the first message

- **Commands:** `.show`
- **Given:** session with 4 known entries
- **When:** `clg .show session_id::ID index::1`
- **Then:** stdout shows only the 1st entry's content
- **Exit:** 0
- **Source:** [param/33_index.md](../../../../docs/cli/param/33_index.md); same test as [command/03_show.md INT-17](../command/03_show.md) boundary variant (`index_boundary_first_position`)

---

### EC-3: `index::` at the last valid position selects the last message

- **Commands:** `.show`
- **Given:** session with 4 known entries
- **When:** `clg .show session_id::ID index::4`
- **Then:** stdout shows only the 4th (last) entry's content — the boundary immediately below the out-of-range case in EC-6
- **Exit:** 0
- **Source:** [param/33_index.md](../../../../docs/cli/param/33_index.md); same test as [command/03_show.md INT-17](../command/03_show.md) boundary variant (`index_boundary_last_position`)

---

### EC-4: `index::0` rejected

- **Commands:** `.show`
- **Given:** any valid session
- **When:** `clg .show session_id::ID index::0`
- **Then:** stderr contains `index must be a positive integer (1-based), got 0`; stdout empty
- **Exit:** 1
- **Source:** [param/33_index.md](../../../../docs/cli/param/33_index.md); same test as [command/03_show.md](../command/03_show.md) validation coverage (`index_zero_rejected`)

---

### EC-5: Negative `index::` rejected

- **Commands:** `.show`
- **Given:** any valid session
- **When:** `clg .show session_id::ID index::-1`
- **Then:** stderr contains `index must be a positive integer (1-based), got -1`; stdout empty
- **Exit:** 1
- **Source:** [param/33_index.md](../../../../docs/cli/param/33_index.md); same test as [command/03_show.md](../command/03_show.md) validation coverage (`index_negative_rejected`)

---

### EC-6: Out-of-range `index::` rejected, error names the actual count

- **Commands:** `.show`
- **Given:** session with 4 known entries
- **When:** `clg .show session_id::ID index::99`
- **Then:** stderr contains `index out of range: 99 (4 entries)`; stdout empty
- **Exit:** 1
- **Source:** [param/33_index.md](../../../../docs/cli/param/33_index.md); same test as [command/03_show.md INT-18](../command/03_show.md) (`int_18_index_out_of_range_rejected`)

---

### EC-7: Counts within the `last::`-windowed slice in project-overview, not the full session

- **Commands:** `.show`
- **Given:** cwd-resolved project, most-recently-active session with 20 known entries
- **When:** `clg .show last::5 index::1`
- **Then:** stdout shows the 1st message of the 5-entry tail window (i.e., the 16th message of the full session) — not the 1st message of the session's complete history
- **Exit:** 0
- **Source:** [param/33_index.md](../../../../docs/cli/param/33_index.md); same test as [command/03_show.md](../command/03_show.md) composition coverage (`index_counts_within_tail_window_not_full_session`)

---

### EC-8: Composed with `show_entries::1` narrows the raw list to one line

- **Commands:** `.show`
- **Given:** session with 4 known entries
- **When:** `clg .show session_id::ID show_metadata::1 show_entries::1 index::3`
- **Then:** stdout's raw entries list shows exactly one line — entry 3's — instead of all 4
- **Exit:** 0
- **Source:** [param/33_index.md](../../../../docs/cli/param/33_index.md); same test as [command/03_show.md INT-6](../command/03_show.md) composition variant (`index_narrows_raw_entries_list`)

---

### EC-9: Composed with `fields::` projects one message's requested attributes

- **Commands:** `.show`
- **Given:** session with 4 known entries, entry 3 a known assistant message
- **When:** `clg .show session_id::ID fields::uuid,model index::3`
- **Then:** stdout shows only entry 3's `uuid` and `model` lines
- **Exit:** 0
- **Source:** [param/33_index.md](../../../../docs/cli/param/33_index.md); same test as [command/03_show.md INT-19](../command/03_show.md) (`int_19_fields_index_composed_single_message_projection`) — same underlying test as [param/32_fields.md EC-12](32_fields.md), asserted once, not duplicated

---

### EC-10: Omitted `index::` leaves every in-scope message shown, unchanged

- **Commands:** `.show`
- **Given:** session with known entries, captured before the `fields::`/`index::` feature existed (golden fixture)
- **When:** `clg .show session_id::ID` (no `index::`)
- **Then:** stdout shows every entry — identical set of messages to pre-feature behavior
- **Exit:** 0
- **Source:** [param/33_index.md](../../../../docs/cli/param/33_index.md); same test as [command/03_show.md INT-2](../command/03_show.md) (`int_2_session_id_shows_conversation_content`, re-asserted post-change)
