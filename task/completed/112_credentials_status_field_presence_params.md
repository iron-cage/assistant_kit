# Implement field-presence params for `.credentials.status`

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Done)

## Goal

Replace the `verbosity::` parameter in `.credentials.status` with 9 independent boolean field-presence params (`account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `org::`, `file::`, `saved::`), add `Account:`, `Expires:`, `File:`, and `Saved:` output lines to the routine, and ensure `format::json` always serialises all 9 fields (Motivated: verbosity was being misused as a field gate — controlling which fields appear rather than how much detail each field shows, breaking the orthogonality principle; each field must be independently suppressible; Observable: `clp .credentials.status` shows 7 default-on lines including `Account:` and `Expires:`, `clp .credentials.status file::1 saved::1` adds two opt-in lines, `format::json` always includes all 9 keys; Scoped: `unilang.commands.yaml` argument list for `.credentials.status`, `credentials_status_routine()` in `src/commands.rs`, `tests/cli/credentials_test.rs` cred01–cred07; Testable: `w3 .test level::3` passes, cred01–cred07 all pass).

The current implementation uses a `(format, verbosity)` match with three text branches that couple field selection to an integer level. The replacement reads 9 boolean flags, reads `_active` for the account name, counts `*.credentials.json` in the accounts dir for saved, and emits each line conditionally. JSON output hard-codes all 9 fields regardless of boolean flags.

All documentation has been updated to reflect this design (see `docs/feature/012_live_credentials_status.md`, `docs/cli/commands.md`, `docs/cli/params.md`, `docs/cli/parameter_groups.md`, `docs/cli/parameter_interactions.md`, `tests/doc/cli/testing/command/11_credentials_status.md`).

## In Scope

### Source Changes

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/adapter.rs` § `BOOL_PARAMS` — add `"account"`, `"sub"`, `"tier"`, `"token"`, `"expires"`, `"email"`, `"org"`, `"file"`, `"saved"` so `account::true` normalizes to `account::1` before unilang parsing
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/lib.rs` § `register_commands()` line 117 — replace `vec![ v(), fmt() ]` for `.credentials.status` with `vec![ fmt() ]` + 9 `Kind::Boolean` args; this is what the runtime actually uses (YAML is metadata-only)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/lib.rs` § help text — update `.credentials.status` help line from `[v::0-2] [format::text|json]` to show field-presence syntax
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/unilang.commands.yaml` § `.credentials.status` arguments — remove `verbosity`; add 9 boolean arguments (`account`, `sub`, `tier`, `token`, `expires`, `email`, `org` default `true`, `file`/`saved` default `false`); update examples
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/commands.rs` § `credentials_status_routine()` — parse format only from `OutputOptions`; parse 9 boolean flags directly; read `_active` marker for account name; count `*.credentials.json` for saved count; emit each text line conditionally; emit all 9 JSON fields unconditionally

### Test Changes

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/credentials_test.rs` — update module-level doc (Test Matrix table, doc text); update cred01–cred05; add cred06 (suppress), cred07 (opt-in)

## Out of Scope

- Documentation updates for docs/cli/ and docs/feature/ (already completed by this doc_tsk pass)
- Email-based account names (Task 110)
- POSIX flag removal (Task 109)
- Credential store relocation (Task 111)
- Changes to `src/output.rs` — `OutputOptions` verbosity field remains; `credentials_status_routine` simply ignores it

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Code style: 2-space indentation, custom formatting; never use `cargo fmt`
- Tests in `tests/` directory of the crate; no inline `#[cfg(test)]` modules
- Feature doc `docs/feature/012_live_credentials_status.md` (FR-17) is the authoritative source for expected behaviour
- `format::json` override rule (Interaction 3): JSON always serialises all 9 fields regardless of boolean flag values — see `docs/cli/parameter_interactions.md`
- `account::` line reads `{credential_store}/_active`; shows `"N/A"` when absent — does NOT call `account::list()`
- `saved::` line counts `*.credentials.json` files in `credential_store`; shows `0` when directory is absent

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_style.rulebook.md` formatting and `test_organization.rulebook.md` test placement rules.
2. **Read feature doc** — Read `docs/feature/012_live_credentials_status.md` as authoritative source for every field's default, label, format, and N/A behaviour.
3. **Read `src/lib.rs`** — Read the `register_commands()` function; note `.credentials.status` line uses `v()` and `fmt()`. Also read the help text `println!` block.
4. **Read `src/adapter.rs`** — Read the `BOOL_PARAMS` constant; note it only has `"dry"`.
5. **Read `src/commands.rs`** — Read current `credentials_status_routine()` fully; note the `(format, verbosity)` match block to replace, and the existing `read_live_cred_meta()` helper.
6. **Read `unilang.commands.yaml`** — Read the `.credentials.status` block to identify exact text to replace.
7. **Write failing tests** — In `credentials_test.rs`: add `cred06_suppress_all_default_on` and `cred07_opt_in_file_and_saved`. Confirm they compile but fail before the implementation.
8. **Update `BOOL_PARAMS` in `adapter.rs`** — Add the 9 field-presence param names so `account::true` normalizes to `account::1` before unilang parsing.
9. **Update `register_commands()` in `lib.rs`** — Replace `.credentials.status` registration from `vec![ v(), fmt() ]` to `vec![ fmt() ]` + 9 boolean params. Add a `bf` closure analogous to `v`, `fmt`, `dry`:
   ```rust
   let bf = | nm : &'static str | reg_arg_opt( nm, Kind::Boolean );
   reg_cmd( registry, ".credentials.status", "...",
     vec![ fmt(), bf("account"), bf("sub"), bf("tier"), bf("token"),
           bf("expires"), bf("email"), bf("org"), bf("file"), bf("saved") ],
     Box::new( credentials_status_routine ) );
   ```
10. **Update help text in `lib.rs`** — Change the `.credentials.status` help line to show `[format::text|json] [account::1] [sub::1] ... [file::0] [saved::0]` (or a concise form).
11. **Update YAML** — In `unilang.commands.yaml`, replace the `verbosity` argument with 9 boolean arguments; update examples to use the new param syntax.
12. **Update `credentials_status_routine()`** — Replace the `(format, verbosity)` match with per-flag conditional output. Helper pattern for boolean flags:
   ```rust
   let show_account = matches!( cmd.arguments.get("account"), Some(Value::Boolean(true)) | None );
   ```
   (None defaults to true for default-on flags; `file`/`saved` use `Some(Value::Boolean(true))` only — false by default.)
   Read account name:
   ```rust
   let account = std::fs::read_to_string( credential_store.join("_active") )
     .ok()
     .map( |s| s.trim().to_string() )
     .filter( |s| !s.is_empty() )
     .unwrap_or_else( || "N/A".to_string() );
   ```
   Read saved count:
   ```rust
   let saved = std::fs::read_dir( &credential_store )
     .map( |rd| rd.filter_map( Result::ok )
       .filter( |e| e.file_name().to_string_lossy().ends_with( ".credentials.json" ) )
       .count() )
     .unwrap_or(0);
   ```
   JSON: include all 9 fields unconditionally; text: emit only flagged lines.
13. **Update existing tests (cred01–cred05)** — Update to match new default behaviour: remove `v::2`/`v::1` params; add `Account:` / `N/A` assertions; add Expires assertion to cred02; update JSON field assertions in cred03 to all 9 keys; update cred05 to expect ≥ 3× `N/A`.
14. **Validate** — Run `w3 .test level::3` inside Docker. All tests must pass.
15. **Walk Validation Checklist** — check every item.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| No credential store | `.credentials.json` only, no `_active` | cred01: exit 0; `Account: N/A`; `Sub:` and `Token:` visible |
| Default with `.claude.json` | Both credential files present, no params | cred02: exit 0; 7 lines: `Account:`, `Sub:`, `Tier:`, `Token:`, `Expires:`, `Email:`, `Org:` |
| `format::json` | `.credentials.json` + `.claude.json` | cred03: exit 0; JSON object with all 9 keys including `file` and `saved` |
| Missing `.credentials.json` | `.claude/` dir exists, no credentials file | cred04: exit non-zero; stderr contains "credential" |
| Default, no `.claude.json` | `.credentials.json` only, no credential store | cred05: exit 0; `Email: N/A`, `Org: N/A`, `Account: N/A` (3× N/A minimum) |
| `account::0 sub::0 tier::0 expires::0 email::0 org::0` | All default-on suppressed | cred06: exit 0; only `Token:` line in output; no `Sub:`, `Tier:`, `Email:`, `Org:`, `Account:`, `Expires:` |
| `file::1 saved::1` | Default output + opt-in flags | cred07: exit 0; `File:` line present with path; `Saved:` line present |

## Acceptance Criteria

- `clp .credentials.status` (no params) shows exactly 7 lines: Account, Sub, Tier, Token, Expires, Email, Org — in that order
- `clp .credentials.status format::json` returns a JSON object with all 9 keys: `subscription`, `tier`, `token`, `expires_in_secs`, `email`, `org`, `account`, `file`, `saved`
- `clp .credentials.status account::0 sub::0 tier::0 expires::0 email::0 org::0` outputs exactly one line (Token)
- `clp .credentials.status file::1 saved::1` shows File: and Saved: appended after the 7 default-on fields
- `Account:` reads `{credential_store}/_active`; shows `N/A` when absent
- `Saved:` shows `0` when `credential_store` dir is absent
- All 7 integration tests cred01–cred07 pass under `w3 .test level::3`

## Validation

### Checklist

Desired answer for every question is YES.

**Field presence (text output)**
- [x] Does `clp .credentials.status` (no params) output `Account:` line?
- [x] Does `clp .credentials.status` (no params) output `Expires:` line?
- [x] Does `clp .credentials.status account::0` suppress the `Account:` line?
- [x] Does `clp .credentials.status file::1` output the `File:` line?
- [x] Does `clp .credentials.status saved::1` output the `Saved:` line?

**JSON completeness**
- [x] Does `format::json` output contain `"account"` key?
- [x] Does `format::json` output contain `"expires_in_secs"` key?
- [x] Does `format::json` output contain `"file"` key?
- [x] Does `format::json` output contain `"saved"` key?
- [x] Does `format::json` output contain all 9 keys even when `file::0 saved::0`?

**N/A handling**
- [x] Does `Account:` show `N/A` when no `_active` marker exists?
- [x] Does `Saved:` show `0` when `credential_store` dir is absent?

**Test suite**
- [x] Do all 7 integration tests cred01–cred07 pass?

**Out of Scope confirmation**
- [x] Is `src/output.rs` unchanged?
- [x] Does no call to `account::list()` appear in `credentials_status_routine()`?
- [x] Does `lib.rs` register `.credentials.status` WITHOUT `v()` (verbosity)?
- [x] Does `adapter.rs` `BOOL_PARAMS` include all 9 field-presence param names?

### Measurements

**M1 — cred01–cred07 all pass**
Command: `w3 .test level::3 2>&1 | grep -E "PASS|FAIL|error" | tail -5`
Before: cred06 and cred07 fail (params not in YAML). Expected: all 7 pass. Deviation: any cred failure.

**M2 — JSON has 9 fields**
Command: `clp .credentials.status format::json | python3 -c "import json,sys; d=json.load(sys.stdin); print(len(d))"`
Expected: `9`. Deviation: fewer fields.

### Invariants

- [x] I1 — test suite: `w3 .test level::3` → 0 failures

### Anti-faking checks

**AF1 — verbosity arg absent from YAML**
Check: `grep -c '"verbosity"' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/unilang.commands.yaml`
Expected: value decremented by 1 from current (currently 5 occurrences for verbosity across other commands; `.credentials.status` block must not contain it). Direct check: `grep -A 40 '"\.credentials\.status"' unilang.commands.yaml | grep -c '"verbosity"'` → `0`.

**AF2 — 9 boolean args registered in YAML**
Check: `grep -A 120 '"\.credentials\.status"' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/unilang.commands.yaml | grep -c '"name":'`
Expected: `11` (1 for command name + `format` + 9 field-presence). Deviation: fewer args means a field is missing.

**AF3 — match on verbosity removed from routine**
Check: `grep -n "opts.verbosity\|verbosity" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/commands.rs | grep credentials -A 0`
Expected: no match within the `credentials_status_routine` function body.

**AF4 — lib.rs registers 9 boolean args (not v())**
Check: `grep -A 5 '\.credentials\.status' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/lib.rs`
Expected: line shows `bf(` calls for `account`, `sub`, etc.; does NOT show `v()`.

**AF5 — BOOL_PARAMS has all 9 new params**
Check: `grep -A 5 'BOOL_PARAMS' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/adapter.rs`
Expected: contains `"account"`, `"sub"`, `"tier"`, `"token"`, `"expires"`, `"email"`, `"org"`, `"file"`, `"saved"`.

## Outcomes

Completed. All 5 source files modified and all tests pass under `w3 .test level::3`.

**Source changes applied:**
- `src/adapter.rs` — `BOOL_PARAMS` now includes all 9 field-presence param names
- `src/lib.rs` — `.credentials.status` registered with `bf()` closures for 9 boolean args; help text updated
- `unilang.commands.yaml` — verbosity removed; 10 args (format + 9 boolean); examples updated
- `src/commands.rs` — `credentials_status_routine()` uses per-field `show_*` booleans; reads `_active` via `credential_store`; counts saved accounts; JSON always emits all 9 fields
- `tests/cli/credentials_test.rs` — cred01–cred05 updated; cred06 (suppress) and cred07 (opt-in) added

**Validation:** `w3 .test level::3` — 4/4 jobs ✅ (local nextest, workspace nextest, doc tests, clippy)
