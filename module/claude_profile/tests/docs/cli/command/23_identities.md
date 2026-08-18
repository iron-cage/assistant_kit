# Command Tests :: `.identities`

### Scope

- **Purpose**: Integration test cases for the `.identities` fleet-seat listing command.
- **Source**: `docs/cli/command/011_identity.md`, `docs/feature/076_identity_tag_filter.md`
- **Covers**: AC-14, AC-15 (📋 planned — no tests exist yet)

### Test Cases

| IT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| IT-01 | AC-14 | Union of markers + filters + owners → one row per Identity | `t14_identities_lists_union` |
| IT-02 | AC-14 | No markers/filters/owners → `(no identities)`, exit 0 | `t14_identities_lists_union` (empty assertion) |
| IT-03 | AC-15 | `format::json` → array of `{"identity","active","owned","include","exclude"}` | `t15_identity_commands_json` |
| IT-04 | AC-15 | Unsupported format (`format::table`) → exit 1 | `t15_identity_commands_json` (rejection assertion) |
| IT-05 | AC-14 | Marker file matching no known raw Identity → filename-derived row (sanitized display) | `t18_identities_filename_derived_row` |
| IT-06 | — | `.identities` appears in `clp .help` after registration | `dot04_all_visible_commands_present` (extend on implementation) |

### Notes

- **📋 Planned — implementation pending.** Source fn names are prescriptive for `tests/cli/identity_filter_test.rs` (IT-06: `tests/cli/dot_test.rs`); none exist yet. Correct drifted names here when implementation lands.
- All IT cases use a temporary isolated credential store with controlled `$USER`/`$HOSTNAME`.
- IT-01/IT-03 share the FT fixture — see `tests/docs/feature/076_identity_tag_filter.md` FT-14/FT-15 (same underlying tests, indexed there for AC traceability).
- IT-05 covers the reverse-derivation fallback in `docs/cli/command/011_identity.md`'s Algorithm step 3 (last-`_` split, sanitized display) — the case where a marker/filter file's Identity appears in no `owner` field.
- Read-only command: every case must assert no store file changed.

---

### IT-01: Union of three sources

- **Given:** `_active_*` marker for `alice@desk` naming `alice@acme.com`; `_filter_*` file for `bob@laptop`; an account whose `owner` is `carol@ws1`.
- **When:** `clp .identities`
- **Then:** Three sorted rows with columns Identity/Active/Owned/Include/Exclude — `alice@desk` shows its active account; `bob@laptop` shows its filter sets; `carol@ws1` shows `Owned 1`. Exits 0.
- **Exit:** 0
- **Source fn:** `t14_identities_lists_union` *(planned)*
- **Source:** [076_identity_tag_filter.md AC-14](../../../../docs/feature/076_identity_tag_filter.md)

---

### IT-02: Empty union prints `(no identities)`

- **Given:** Store with accounts but no markers, no filter files, no `owner` fields.
- **When:** `clp .identities`
- **Then:** Stdout is `(no identities)`. Exits 0.
- **Exit:** 0
- **Source fn:** `t14_identities_lists_union` *(planned; empty assertion)*
- **Source:** [076_identity_tag_filter.md AC-14](../../../../docs/feature/076_identity_tag_filter.md)

---

### IT-03: `format::json` shape

- **Given:** Fixture as IT-01.
- **When:** `clp .identities format::json`
- **Then:** Stdout is a JSON array of `{"identity": …, "active": …|null, "owned": N, "include": […], "exclude": […]}` objects. Exits 0.
- **Exit:** 0
- **Source fn:** `t15_identity_commands_json` *(planned)*
- **Source:** [076_identity_tag_filter.md AC-15](../../../../docs/feature/076_identity_tag_filter.md)

---

### IT-04: Unsupported format exits 1

- **Given:** Any state.
- **When:** `clp .identities format::table`
- **Then:** Exits 1; stderr states `format::` must be `text` or `json`.
- **Exit:** 1
- **Source fn:** `t15_identity_commands_json` *(planned; rejection assertion)*
- **Source:** [076_identity_tag_filter.md AC-15](../../../../docs/feature/076_identity_tag_filter.md)

---

### IT-05: Filename-derived row for unmatched marker

- **Given:** A `_active_devbox_dave` marker whose Identity appears in no `owner` field and is not the current Identity.
- **When:** `clp .identities`
- **Then:** A row appears for `dave@devbox` (derived by last-`_` split of the filename suffix, sanitized display form), with its Active account populated from the marker.
- **Exit:** 0
- **Source fn:** `t18_identities_filename_derived_row` *(planned)*
- **Source:** [011_identity.md](../../../../docs/cli/command/011_identity.md)

---

### IT-06: `.identities` appears in `clp .help`

- **Given:** Any environment (post-implementation).
- **When:** `clp .help`
- **Then:** Output contains `.identities`. Exits 0.
- **Exit:** 0
- **Source fn:** `dot04_all_visible_commands_present` *(extend on implementation — `tests/cli/dot_test.rs`)*
- **Source:** [011_identity.md](../../../../docs/cli/command/011_identity.md)
