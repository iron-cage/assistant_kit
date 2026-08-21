# Parameter :: `fields::`

Edge case tests for the `fields::` parameter on `.show`. Tests validate the attribute-projection field selector — valid/invalid tokens, `all`, normalization, defaults, and composition with `index::`, `last::`, and `show_metadata::`.

**Source:** [param/32_fields.md](../../../../docs/cli/param/32_fields.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Single field shows just that field for every entry | Behavior |
| EC-2 | Multiple fields shown in request order | Behavior |
| EC-3 | `all` shows every one of the 18 fields, including ones content mode drops | Behavior |
| EC-4 | Invalid field token rejected | Error Handling |
| EC-5 | `all` combined with another token rejected | Error Handling |
| EC-6 | Empty value rejected | Error Handling |
| EC-7 | Case-insensitive, whitespace-trimmed token parsing matches canonical byte-for-byte | Case Insensitivity |
| EC-8 | Duplicate tokens collapse to one occurrence | Normalization |
| EC-9 | Omitted `fields::` leaves default chat-log output unchanged | Default |
| EC-10 | Applies to the project-overview tail window, not just session-detail | Composition |
| EC-11 | Assistant-only field on a `user` entry renders as `—` | Edge Case |
| EC-12 | Composed with `index::` narrows projection to exactly one message | Composition |
| EC-13 | User-only field (`thinking_level`) on an `assistant` entry renders as `—` | Edge Case |

## Test Coverage Summary

- Behavior: 3 tests (EC-1, EC-2, EC-3)
- Error Handling: 3 tests (EC-4, EC-5, EC-6)
- Case Insensitivity: 1 test (EC-7)
- Normalization: 1 test (EC-8)
- Default: 1 test (EC-9)
- Composition: 2 tests (EC-10, EC-12)
- Edge Case: 2 tests (EC-11, EC-13)

**Total:** 13 edge cases

## Test Cases

---

### EC-1: Single field shows just that field for every entry

- **Commands:** `.show`
- **Given:** session with 3 known entries at distinct timestamps
- **When:** `clg .show session_id::ID fields::timestamp`
- **Then:** stdout shows exactly one `timestamp` field line per entry (plus the entry header); no other field, no chat-log message text
- **Exit:** 0
- **Source:** [param/32_fields.md](../../../../docs/cli/param/32_fields.md); same test as [command/03_show.md INT-13](../command/03_show.md) (`int_13_fields_single_field_every_entry`)

---

### EC-2: Multiple fields shown in request order

- **Commands:** `.show`
- **Given:** session with a known assistant entry
- **When:** `clg .show session_id::ID fields::model,uuid`
- **Then:** stdout lists the `model` line before the `uuid` line for that entry — matches request order, not canonical vocabulary order
- **Exit:** 0
- **Source:** [param/32_fields.md](../../../../docs/cli/param/32_fields.md); same test as [command/03_show.md INT-14](../command/03_show.md) (`int_14_fields_multi_field_request_order`)

---

### EC-3: `all` shows every one of the 18 fields, including ones content mode drops

- **Commands:** `.show`
- **Given:** session with an assistant entry carrying a `tool_use` block and a successful `tool_result` block, plus a user entry with `thinking_metadata` present
- **When:** `clg .show session_id::ID fields::all`
- **Then:** stdout includes all 18 canonical field lines, including `parent_uuid`, `cwd`, `version`, `git_branch`, `request_id`, the user entry's `thinking_level`/`thinking_disabled`, the tool_use block's `id` and full `input` JSON, and the successful tool_result's content — none ever shown by default chat-log mode
- **Exit:** 0
- **Source:** [param/32_fields.md](../../../../docs/cli/param/32_fields.md); same test as [command/03_show.md INT-15](../command/03_show.md) (`int_15_fields_all_shows_every_dropped_attribute`)

---

### EC-4: Invalid field token rejected

- **Commands:** `.show`
- **Given:** any valid session
- **When:** `clg .show session_id::ID fields::bogus`
- **Then:** stderr contains `unknown field 'bogus' — valid fields: uuid, parent_uuid, timestamp, entry_type, cwd, session_id, version, git_branch, user_type, is_sidechain, content, thinking_level, thinking_disabled, model, message_id, stop_reason, stop_sequence, request_id, or all`; stdout empty
- **Exit:** 1
- **Source:** [param/32_fields.md](../../../../docs/cli/param/32_fields.md); same test as [command/03_show.md INT-16](../command/03_show.md) (`int_16_fields_invalid_token_rejected`)

---

### EC-5: `all` combined with another token rejected

- **Commands:** `.show`
- **Given:** any valid session
- **When:** `clg .show session_id::ID fields::all,uuid`
- **Then:** stderr contains `'all' cannot be combined with other fields`; stdout empty
- **Exit:** 1
- **Source:** [param/32_fields.md](../../../../docs/cli/param/32_fields.md); same test as [type/15_field_selector.md TC-5](../type/15_field_selector.md) (`fields_all_combined_with_other_rejected`)

---

### EC-6: Empty value rejected

- **Commands:** `.show`
- **Given:** any valid session
- **When:** `clg .show session_id::ID fields::`
- **Then:** rejected — unilang's own argument parser requires a value token after `fields::` and rejects the missing token before `FieldSelector::parse()` ever runs; stderr mentions `fields`, stdout empty
- **Exit:** 1
- **Source:** [param/32_fields.md](../../../../docs/cli/param/32_fields.md); framework-level parse error, distinct from [type/15_field_selector.md TC-9](../type/15_field_selector.md)'s unit-level `FieldSelector::parse("")` contract (`fields_empty_value_rejected`)

---

### EC-7: Case-insensitive, whitespace-trimmed token parsing matches canonical byte-for-byte

- **Commands:** `.show`
- **Given:** session with a known entry
- **When:** `clg .show session_id::ID "fields:: UUID , Timestamp "` compared against `clg .show session_id::ID fields::uuid,timestamp`
- **Then:** stdout is byte-identical between the two invocations
- **Exit:** 0
- **Source:** [param/32_fields.md](../../../../docs/cli/param/32_fields.md); same test as [type/15_field_selector.md TC-6](../type/15_field_selector.md), [TC-7](../type/15_field_selector.md) (`fields_case_insensitive_whitespace_trimmed_matches_canonical`)

---

### EC-8: Duplicate tokens collapse to one occurrence

- **Commands:** `.show`
- **Given:** session with a known entry
- **When:** `clg .show session_id::ID fields::uuid,uuid` compared against `clg .show session_id::ID fields::uuid`
- **Then:** stdout is byte-identical between the two invocations — `uuid` appears exactly once per entry, not twice
- **Exit:** 0
- **Source:** [param/32_fields.md](../../../../docs/cli/param/32_fields.md); same test as [type/15_field_selector.md TC-8](../type/15_field_selector.md) (`fields_duplicate_token_collapses`)

---

### EC-9: Omitted `fields::` leaves default chat-log output unchanged

- **Commands:** `.show`
- **Given:** session with known entries, captured before the `fields::`/`index::` feature existed (golden fixture)
- **When:** `clg .show session_id::ID` (no `fields::`)
- **Then:** stdout matches the documented default chat-log content format (see [`command/03_show.md`](../command/03_show.md)) — same fields shown as before this feature, same content selected; only the punctuation/color restyling (→ [`../../../../docs/cli/readme.md` § Local Style Conventions](../../../../docs/cli/readme.md)) may differ from the pre-restyle byte form
- **Exit:** 0
- **Source:** [param/32_fields.md](../../../../docs/cli/param/32_fields.md); same test as [command/03_show.md INT-2](../command/03_show.md) (`int_2_session_id_shows_conversation_content`, re-asserted post-change)

---

### EC-10: Applies to the project-overview tail window, not just session-detail

- **Commands:** `.show`
- **Given:** cwd-resolved project, most-recently-active session with ≥5 known entries
- **When:** `clg .show fields::timestamp last::5`
- **Then:** stdout shows the project summary block unchanged, followed by field-projection blocks (not chat-log content) for the last 5 entries
- **Exit:** 0
- **Source:** [param/32_fields.md](../../../../docs/cli/param/32_fields.md); same test as [command/03_show.md INT-20](../command/03_show.md) (`int_20_fields_applies_to_project_overview_tail_window`)

---

### EC-11: Assistant-only field on a `user` entry renders as `—`

- **Commands:** `.show`
- **Given:** session whose first entry is a `user` message
- **When:** `clg .show session_id::ID fields::model index::1`
- **Then:** stdout shows `model · —` for that entry — no error, no panic
- **Exit:** 0
- **Source:** [param/32_fields.md](../../../../docs/cli/param/32_fields.md); same test as [type/15_field_selector.md](../type/15_field_selector.md) role-gap coverage (`fields_assistant_only_field_on_user_entry_renders_em_dash`)

---

### EC-12: Composed with `index::` narrows projection to exactly one message

- **Commands:** `.show`
- **Given:** session with 4 known entries, entry 3 a known assistant message
- **When:** `clg .show session_id::ID fields::uuid,model index::3`
- **Then:** stdout shows only entry 3's `uuid` and `model` lines — no other entry, no other field
- **Exit:** 0
- **Source:** [param/32_fields.md](../../../../docs/cli/param/32_fields.md); same test as [command/03_show.md INT-19](../command/03_show.md) (`int_19_fields_index_composed_single_message_projection`)

---

### EC-13: User-only field (`thinking_level`) on an `assistant` entry renders as `—`

- **Commands:** `.show`
- **Given:** session whose second entry is an `assistant` message
- **When:** `clg .show session_id::ID fields::thinking_level index::2`
- **Then:** stdout shows `thinking_level · —` for that entry — no error, no panic; mirrors EC-11's role-gap rendering in the opposite direction (user-only field on an assistant entry, rather than assistant-only field on a user entry)
- **Exit:** 0
- **Source:** [param/32_fields.md](../../../../docs/cli/param/32_fields.md); same test as [type/15_field_selector.md](../type/15_field_selector.md) role-gap coverage (`fields_user_only_field_on_assistant_entry_renders_em_dash`)
