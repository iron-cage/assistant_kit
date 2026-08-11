# IT — Command 20: `.model.select` (retirement stub)

### Scope

- **Purpose**: Integration test cases for `.model.select`'s post-retirement behavior — every invocation form now returns a single migration-error stub, unconditionally, exit 1. This file replaces the prior IT-01 through IT-12 live get/set/reset scenarios (Feature 069, now superseded) with the 3 forms the stub still accepts as *syntax* (though it acts on none of them).
- **Source**: `docs/cli/command/007_model.md` (primary — Command 20, retirement-stub description), `docs/feature/035_model_command.md` (AC-26 — hidden-from-listing), `docs/feature/069_model_select_command.md` (historical only — superseded original design, AC-01 through AC-12)
- **Covers**: T23 (task 465's own Test Matrix row covering all 3 invocation forms of the retired command)

### Superseded

`docs/feature/069_model_select_command.md`'s AC-01 through AC-12 described `.model.select`'s original live get/set/reset behavior against `~/.clr/config.toml`'s `model` key. That behavior no longer exists — the command body is now a static migration-error stub (`src/commands/model_select.rs`). Feature 069 is retained for historical reference only; this file no longer tests any of its ACs. The replacement functionality lives in `.model scope::subprocess model::VALUE` / `reset_model::1`, covered by `17_model.md` (IT-09, IT-10, IT-17) and `035_model_command.md` (FT-09, FT-10, FT-17).

### Test Cases

| IT | Scenario | Source fn |
|----|----------|-----------|
| IT-01 | `.model.select` (bare/get form) → exit 1, migration message naming the replacement syntax | ✅ `t23_get_form_exits_1_with_migration_message` |
| IT-02 | `.model.select id::claude-opus-4-8` (`id::` form) → exit 1, same migration message; `id::` is ignored, not acted on | ✅ `t23_id_form_exits_1_with_migration_message` |
| IT-03 | `.model.select reset::1` (`reset::` form) → exit 1, same migration message; `reset::1` is ignored, not acted on | ✅ `t23_reset_form_exits_1_with_migration_message` |

### Notes

- All 3 forms are covered in a single Test Matrix row (T23) in task 465's own file, since they share one code path (the stub ignores all parameters and always returns the same error) — reflected here as 3 distinct IT cases only because each is a syntactically distinct invocation worth confirming individually accepts parsing and still exits 1.
- None of the 3 forms write `.clr/config.toml` as a side effect — asserted in each source fn.
- **Corrects prior IT-11** (old revision of this file, now removed): previously asserted `.model.select` appears in `clp .help`. This is now FALSE — per AC-26 (Feature 035), `.model.select` is hidden from the listing via `hidden_from_list(true)` while remaining registered/dispatchable. The current listing assertion (`.model.select` absent) is `dot13_model_select_hidden_from_listing` in `tests/cli/dot_test.rs` — a shared CLI-listing concern, not re-tested here.
- `.model.select` remains registered and dispatchable (these 3 IT cases confirm that: dispatch succeeds, argument parsing succeeds, only the command *body* is a stub) — it is hidden from `.help`/`.` output only, not deregistered outright. See `docs/cli/command/007_model.md`'s Command 20 section for the exact stub description and full migration-message wording contract.

---

### IT-01: Bare `.model.select` — get form

- **Given:** Fresh `HOME`.
- **When:** `clp .model.select`
- **Then:** Exit 1. Stderr contains `model.select`, `REMOVED`, and `.model scope::subprocess` (the named replacement syntax). No `.clr/config.toml` created.
- **Exit:** 1
- **Source fn:** ✅ `t23_get_form_exits_1_with_migration_message`
- **Source:** [007_model.md — Command 20](../../../../docs/cli/command/007_model.md)

---

### IT-02: `.model.select id::claude-opus-4-8` — `id::` form

- **Given:** Fresh `HOME`.
- **When:** `clp .model.select id::claude-opus-4-8`
- **Then:** Exit 1. Same migration message as IT-01 — the stub does not read or act on `id::`. No `.clr/config.toml` created.
- **Exit:** 1
- **Source fn:** ✅ `t23_id_form_exits_1_with_migration_message`
- **Source:** [007_model.md — Command 20](../../../../docs/cli/command/007_model.md)

---

### IT-03: `.model.select reset::1` — `reset::` form

- **Given:** Fresh `HOME`.
- **When:** `clp .model.select reset::1`
- **Then:** Exit 1. Same migration message as IT-01 — the stub does not read or act on `reset::1`. No `.clr/config.toml` created.
- **Exit:** 1
- **Source fn:** ✅ `t23_reset_form_exits_1_with_migration_message`
- **Source:** [007_model.md — Command 20](../../../../docs/cli/command/007_model.md)
