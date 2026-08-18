# Command Tests :: `.tags`

### Scope

- **Purpose**: Integration test cases for the `.tags` distinct-tag listing command.
- **Source**: `docs/cli/command/010_tag.md`, `docs/feature/075_account_tags.md`
- **Covers**: AC-11, AC-12 (📋 planned — no tests exist yet)

### Test Cases

| IT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| IT-01 | AC-11 | Mixed fixture → sorted rows with account and filter counts | `t11_tags_lists_union_sorted` |
| IT-02 | AC-11 | Untagged store → `(no tags)`, exit 0 | `t11_tags_lists_union_sorted` (empty-store assertion) |
| IT-03 | AC-11 | Filter-only tag → row with `Accounts = 0`, `Filters ≥ 1` | `t11_tags_lists_union_sorted` (typo-hazard assertion) |
| IT-04 | AC-12 | `format::json` → array of `{"tag","accounts","filters"}` | `t12_tags_json_shape` |
| IT-05 | — | Unsupported format (`format::table`) → exit 1 | `t17_tags_bad_format_exits_1` |
| IT-06 | — | `.tags` appears in `clp .help` after registration | `dot04_all_visible_commands_present` (extend on implementation) |

### Notes

- **📋 Planned — implementation pending.** Source fn names are prescriptive for `tests/cli/account_tag_test.rs` (IT-06: `tests/cli/dot_test.rs`); none exist yet. Correct drifted names here when implementation lands.
- All IT cases use a temporary isolated credential store.
- IT-01/IT-03/IT-04 share the FT fixture: accounts carrying `ci` (×2) and `kimi_pool` (×1); a `_filter_*` file referencing `kimi_pool` and `typo_tag` — see `tests/docs/feature/075_account_tags.md` FT-11/FT-12 (same underlying tests, indexed there for AC traceability).
- Read-only command: every case must assert no store file changed.

---

### IT-01: Listing with counts

- **Given:** Accounts: two carrying `ci`, one carrying `kimi_pool`; one `_filter_*` file referencing `kimi_pool`.
- **When:** `clp .tags`
- **Then:** Sorted rows `ci 2 0` and `kimi_pool 1 1` with `Tag`/`Accounts`/`Filters` columns. Exits 0.
- **Exit:** 0
- **Source fn:** `t11_tags_lists_union_sorted` *(planned)*
- **Source:** [075_account_tags.md AC-11](../../../../docs/feature/075_account_tags.md)

---

### IT-02: Empty union prints `(no tags)`

- **Given:** Store with accounts but no `tags` keys and no `_filter_*` files.
- **When:** `clp .tags`
- **Then:** Stdout is `(no tags)`. Exits 0.
- **Exit:** 0
- **Source fn:** `t11_tags_lists_union_sorted` *(planned; empty-store assertion)*
- **Source:** [075_account_tags.md AC-11](../../../../docs/feature/075_account_tags.md)

---

### IT-03: Filter-only tag shows zero accounts

- **Given:** `_filter_*` file includes `typo_tag`; no account carries it.
- **When:** `clp .tags`
- **Then:** Row `typo_tag 0 1` present — the typo-hazard surface ([feature/076 AC-08](../../../../docs/feature/076_identity_tag_filter.md)'s write-time counterpart).
- **Exit:** 0
- **Source fn:** `t11_tags_lists_union_sorted` *(planned; typo-hazard assertion)*
- **Source:** [075_account_tags.md AC-11](../../../../docs/feature/075_account_tags.md)

---

### IT-04: `format::json` shape

- **Given:** Fixture as IT-01.
- **When:** `clp .tags format::json`
- **Then:** Stdout is a JSON array of `{"tag": …, "accounts": N, "filters": N}` objects sorted by tag. Exits 0.
- **Exit:** 0
- **Source fn:** `t12_tags_json_shape` *(planned)*
- **Source:** [075_account_tags.md AC-12](../../../../docs/feature/075_account_tags.md)

---

### IT-05: Unsupported format exits 1

- **Given:** Any state.
- **When:** `clp .tags format::table`
- **Then:** Exits 1; stderr states `format::` must be `text` or `json`.
- **Exit:** 1
- **Source fn:** `t17_tags_bad_format_exits_1` *(planned)*
- **Source:** [010_tag.md](../../../../docs/cli/command/010_tag.md)

---

### IT-06: `.tags` appears in `clp .help`

- **Given:** Any environment (post-implementation).
- **When:** `clp .help`
- **Then:** Output contains `.tags`. Exits 0.
- **Exit:** 0
- **Source fn:** `dot04_all_visible_commands_present` *(extend on implementation — `tests/cli/dot_test.rs`)*
- **Source:** [010_tag.md](../../../../docs/cli/command/010_tag.md)
