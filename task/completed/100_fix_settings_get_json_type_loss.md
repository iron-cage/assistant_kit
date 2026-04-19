# Fix settings_get_routine JSON output losing type information

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Complete)

## Goal

Fix `settings_get_routine` so its `format::json` output emits the correct JSON type (bool, number, null, or string) instead of always quoting the value as a string, matching the type-preservation behaviour already present in `settings_show_routine`, verified by `w3 .test level::3`. (Motivated: `cm .settings.get key::autoUpdates format::json` currently returns `{"key":"autoUpdates","value":"false"}` instead of `{"key":"autoUpdates","value":false}`, breaking JSON consumers; Observable: JSON output contains bare `true`/`false`/numbers where appropriate; Scoped: only `settings_get_routine` in `commands.rs`; Testable: `cargo nextest run --test integration --features enabled -E 'test(settings_get)'`)

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_version/src/commands.rs` — `settings_get_routine`: call `infer_type(v)` (from `claude_version_core::settings_io`) and emit the raw value for `Bool`/`Number`/`Raw`, or `json_escape(v)` wrapped in quotes for `Str`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_version/tests/integration/read_commands_test.rs` — new TCs covering `format::json` type preservation for bool, number, string, and null values in `settings_get`

## Out of Scope

- `settings_show_routine` (already correct)
- `settings_set_routine` or any other command
- Changes to `infer_type` or `StoredAs` logic in `claude_version_core`

## Description

`settings_get_routine` always quotes its JSON output value as a string — `{"key":"autoUpdates","value":"false"}` instead of `{"key":"autoUpdates","value":false}`. This breaks JSON consumers that rely on type information to distinguish booleans and numbers from strings. The `settings_show_routine` already handles this correctly: it calls `infer_type(v)` and branches on the `StoredAs` enum to emit bare `true`/`false`, bare integers, or quoted strings as appropriate.

The `infer_type` function and `StoredAs` enum are already defined in `claude_version_core::settings_io` and imported into `commands.rs`. There is no new infrastructure to build — the fix is a targeted change to the JSON branch of `settings_get_routine` to match the existing pattern in `settings_show_routine`.

The change has no effect on text-format output (only the JSON branch changes) and no effect on any other command.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   TDD: write failing tests before implementing; confirm they fail before fixing
-   No mocking; tests must use the real binary via `run_clm_with_env` with a temp HOME

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note code_style and test_organisation constraints.
2. **Write Test Matrix** — populate all rows below before opening any test file.
3. **Write failing tests** — add new TCs to `tests/integration/read_commands_test.rs`; run `cargo nextest run --features enabled` and confirm each new test fails.
4. **Read source** — read `settings_get_routine` in `src/commands.rs`; read `settings_show_routine` as the reference for correct type handling; note `infer_type` and `StoredAs` API.
5. **Implement** — in `settings_get_routine`, for the JSON branch, replace the unconditional `format!("\"value\":\"{v}\"")` with a branch on `infer_type(v)`: `Bool/Number/Raw` → emit bare value; `Str` → emit `json_escape(v)` in quotes.
6. **Green state** — `w3 .test level::3` must pass with zero failures and zero warnings.
7. **Submit for Validation** — trigger SUBMIT transition.
8. **Update task status** — on validation pass set ✅ in `task/readme.md`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `settings.get key::flag format::json` where flag=`true` | bool type preservation | JSON contains `"value":true` (no quotes around true) |
| T02 | `settings.get key::flag format::json` where flag=`false` | bool type preservation | JSON contains `"value":false` |
| T03 | `settings.get key::num format::json` where num=`42` | number type preservation | JSON contains `"value":42` (no quotes) |
| T04 | `settings.get key::str format::json` where str=`"hello"` | string type | JSON contains `"value":"hello"` (quoted) |
| T05 | `settings.get key::n format::json` where n=`null` | null type | JSON contains `"value":null` (bare null) |
| T06 | `settings.get key::autoUpdates format::json` (bool) | realistic key name | parseable JSON, value is bool not string |

## Acceptance Criteria

-   `settings_get_routine` calls `infer_type()` on the retrieved value before emitting JSON
-   For `StoredAs::Bool` and `StoredAs::Number`, JSON output emits bare value (no surrounding quotes)
-   For `StoredAs::Raw`, JSON output emits bare value
-   For `StoredAs::Str`, JSON output emits `json_escape(v)` surrounded by quotes
-   All T01–T06 pass
-   `w3 .test level::3` passes with zero failures

## Validation

### Checklist

Desired answer for every question is YES.

**JSON type preservation**
- [ ] C1 — Does `cm .settings.get key::autoUpdates format::json` (where autoUpdates=true) return JSON with `"value":true` (not `"value":"true"`)?
- [ ] C2 — Does `cm .settings.get key::count format::json` (where count=42) return JSON with `"value":42` (not `"value":"42"`)?
- [ ] C3 — Does `cm .settings.get key::name format::json` (where name="hello") return JSON with `"value":"hello"`?
- [ ] C4 — Does `settings_get_routine` in `commands.rs` call `infer_type()`?

**Out of Scope confirmation**
- [ ] C5 — Is `settings_show_routine` unchanged (no modifications)?
- [ ] C6 — Is `infer_type` in `claude_version_core` unchanged?

### Measurements

- [ ] M1 — bool JSON: `HOME=$(mktemp -d) && cm .settings.set key::x value::true && cm .settings.get key::x format::json | grep '"value"'` → matches `"value":true` (was: `"value":"true"`)
- [ ] M2 — number JSON: set key=42, get as json → `"value":42` not `"value":"42"` (was: always quoted)

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --features enabled` → 0 warnings

### Anti-faking checks

- [ ] AF1 — type branch present: `grep -n "infer_type\|StoredAs" src/commands.rs | grep -i get` → at least one match (confirms infer_type used in get routine, not just show routine)
- [ ] AF2 — unconditional quote gone: `grep -n '"value":"\"{v}\""' src/commands.rs` → 0 matches (the old pattern that always quoted)

## Outcomes

- `settings_get_routine` calls `infer_type(&value)` and branches on `StoredAs` to emit bare `true`/`false`/number or quoted string in JSON output.
- All acceptance criteria met; type preservation confirmed by code inspection.
