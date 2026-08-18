# Command Tests :: `.account.tag`

### Scope

- **Purpose**: Integration test cases for the `.account.tag` per-account tag mutation command.
- **Source**: `docs/cli/command/001_account.md` (Command 25), `docs/feature/075_account_tags.md`
- **Covers**: AC-03, AC-05 through AC-10 (✅ implemented)

### Test Cases

| IT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| IT-01 | AC-05 | `add::a,b` unions into the existing set (dedup, sorted) | `account_tag_t05_tag_add_unions_set` |
| IT-02 | AC-06 | `remove::a` removes; absent tag is a no-op success | `account_tag_t06_tag_remove_idempotent` |
| IT-03 | AC-07 | `tags::a,b` replaces the whole set; combining ops exits 1 | `account_tag_t07_tag_replace_and_mutual_exclusion` |
| IT-04 | AC-08 | No operation given → exit 1 naming `add::`/`remove::`/`tags::` | `account_tag_t08_tag_no_operation_exits_1` |
| IT-05 | AC-09 | First tag write (incl. `remove::`) migrates legacy `role` to tag, deletes field | `account_tag_t09_first_tag_write_migrates_role` |
| IT-06 | AC-10 | Ungated (non-owner OK); `name::X,Y` batch; `dry::1` touches nothing | `account_tag_t10_tag_ungated_batch_dry` |
| IT-07 | AC-03 | Invalid tag in any operation → exit 1 naming it, no write | `account_tag_t03_invalid_tag_exits_1_no_write` (extend with `.account.tag` assertion) |
| IT-08 | — | Unknown account name → exit 2, nothing written | `account_tag_t19_account_tag_unknown_account_exits_2` |
| IT-09 | — | Missing required `name::` → exit 1 | `account_tag_t20_account_tag_missing_name_exits_1` |
| IT-10 | — | `.account.tag` appears in `clp .help` after registration | `dot04_all_visible_commands_present` (extend on implementation) |

### Notes

- ✅ Implemented — source fns live in `tests/cli/account_tag_test.rs` (fn names carry the `account_tag_` file prefix); IT-10 in `tests/cli/dot_test.rs`.
- All IT cases use a temporary isolated credential store; no real user environment.
- IT-01–IT-06 are the command-level index of `tests/docs/feature/075_account_tags.md` FT-05–FT-10 (same underlying tests, indexed there for AC traceability); per-case Given/When/Then live in that FT spec and are not duplicated here.
- IT-06 must assert both halves of the ungated doctrine: no ownership gate (writes metadata only), and no credential file is ever touched by any operation.
- IT-05 must fire the migration via `.account.tag` specifically (both `add::` and `remove::` variants) — the `.account.save tags::` trigger path is FT-09's variant.

---

### IT-07: Invalid tag on any operation exits 1

- **Given:** `alice@test.com` saved with `"tags": ["ci"]`.
- **When:** `clp .account.tag name::alice@test.com add::Bad!Tag` (also via `remove::`/`tags::`)
- **Then:** Exits 1; stderr names the offending tag (post-lowercasing form); the stored set is unchanged.
- **Exit:** 1
- **Source fn:** `account_tag_t03_invalid_tag_exits_1_no_write` *(extend with `.account.tag` path assertion)*
- **Source:** [075_account_tags.md AC-03](../../../../docs/feature/075_account_tags.md)

---

### IT-08: Unknown account exits 2

- **Given:** No account named `ghost@test.com`.
- **When:** `clp .account.tag name::ghost@test.com add::ci`
- **Then:** Exits 2; stderr names the missing account; nothing written. In a `name::X,Y` batch, an unknown member fails the invocation before any member is modified.
- **Exit:** 2
- **Source fn:** `account_tag_t19_account_tag_unknown_account_exits_2`
- **Source:** [001_account.md Command 25](../../../../docs/cli/command/001_account.md)

---

### IT-09: Missing `name::` exits 1

- **Given:** Any state.
- **When:** `clp .account.tag add::ci`
- **Then:** Exits 1; stderr states `name::` is required.
- **Exit:** 1
- **Source fn:** `account_tag_t20_account_tag_missing_name_exits_1`
- **Source:** [001_account.md Command 25](../../../../docs/cli/command/001_account.md)

---

### IT-10: `.account.tag` appears in `clp .help`

- **Given:** Any environment (post-implementation).
- **When:** `clp .help`
- **Then:** Output contains `.account.tag`. Exits 0.
- **Exit:** 0
- **Source fn:** `dot04_all_visible_commands_present` *(`tests/cli/dot_test.rs`)*
- **Source:** [001_account.md Command 25](../../../../docs/cli/command/001_account.md)
