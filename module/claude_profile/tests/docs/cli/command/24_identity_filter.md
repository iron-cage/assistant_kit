# Command Tests :: `.identity.filter`

### Scope

- **Purpose**: Integration test cases for the `.identity.filter` get/set/clear command over the per-Identity Tag Filter file.
- **Source**: `docs/cli/command/011_identity.md`, `docs/feature/076_identity_tag_filter.md`
- **Covers**: AC-01 through AC-08, AC-15 (✅ implemented)

### Test Cases

| IT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| IT-01 | AC-01 | Get with no filter file → `include=[] exclude=[] (permit-all)`, exit 0 | `identity_filter_t01_filter_get_permit_all` |
| IT-02 | AC-02 | `include::a,b` writes sorted deduplicated include, empty exclude | `identity_filter_t02_filter_include_write` |
| IT-03 | AC-03 | Each given side fully replaces; both sides in one call | `identity_filter_t03_filter_both_sides_replace` |
| IT-04 | AC-04 | `include ∩ exclude ≠ ∅` → exit 1 naming overlap, nothing written | `identity_filter_t04_filter_overlap_exits_1` |
| IT-05 | AC-05 | Invalid tag in either set → exit 1 naming it, nothing written | `identity_filter_t05_filter_invalid_tag_exits_1` |
| IT-06 | AC-06 | `clear::1` deletes; idempotent; with `include::`/`exclude::` → exit 1 | `identity_filter_t06_filter_clear_idempotent_and_exclusive` |
| IT-07 | AC-07 | `identity::USER@MACHINE` targets that seat; malformed → exit 1 | `identity_filter_t07_filter_identity_targeting` |
| IT-08 | AC-08 | Include matching zero tagged accounts → stderr warning, exit 0 | `identity_filter_t08_filter_typo_guard_warns` |
| IT-09 | AC-15 | Get with `format::json`; unsupported format → exit 1 | `identity_filter_t15_identity_commands_json` |
| IT-10 | — | `.identity.filter` appears in `clp .help` after registration | `dot04_all_visible_commands_present` |

### Notes

- ✅ Implemented — source fns live in `tests/cli/identity_filter_test.rs` (fn names carry the `identity_filter_` file prefix); IT-10 in `tests/cli/dot_test.rs`.
- All IT cases use a temporary isolated credential store with controlled `$USER`/`$HOSTNAME` — the default filter filename depends on the current Identity.
- IT-01–IT-08 are the command-level index of `tests/docs/feature/076_identity_tag_filter.md` FT-01–FT-08 (same underlying tests, indexed there for AC traceability); per-case Given/When/Then live in that FT spec and are not duplicated here.
- Gate 11 selection behavior (`rotate::1`, auto-switch, footer `Next`) is out of this command's IT scope — covered by FT-09/FT-11/FT-12/FT-13 in the FT spec, implemented in `tests/usage/sort_next_tests_b.rs` (`test_cc_gate11_*`) and `identity_filter_t13_usage_reports_excluded_count`.
- Mode dispatch contract under test: get (no set/clear params), set (`include::`/`exclude::`), clear (`clear::1`) — combining clear with set exits 1 (IT-06); `identity::` and `format::` compose with any mode (IT-07/IT-09).

---

### IT-09: `format::json` get; unsupported format exits 1

- **Given:** Filter file `{"include": ["kimi_pool"], "exclude": ["personal"]}` for the current Identity.
- **When:** `clp .identity.filter format::json`; then `clp .identity.filter format::table`
- **Then:** JSON get emits `{"identity": …, "include": ["kimi_pool"], "exclude": ["personal"]}` (exit 0); `format::table` exits 1.
- **Exit:** 0 / 1
- **Source fn:** `identity_filter_t15_identity_commands_json`
- **Source:** [076_identity_tag_filter.md AC-15](../../../../docs/feature/076_identity_tag_filter.md)

---

### IT-10: `.identity.filter` appears in `clp .help`

- **Given:** Any environment (post-implementation).
- **When:** `clp .help`
- **Then:** Output contains `.identity.filter`. Exits 0.
- **Exit:** 0
- **Source fn:** `dot04_all_visible_commands_present` *(`tests/cli/dot_test.rs`)*
- **Source:** [011_identity.md](../../../../docs/cli/command/011_identity.md)
