# Add parameter descriptions to `.credentials.status` help output

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 🎯 (Available)

## Goal

Register accurate one-line descriptions for all 10 `.credentials.status` parameters so that `clp .credentials.status.help` shows what each field-presence param controls, replacing the current blank-description output. (Motivated: the help screen lists 10 params with no text — users cannot discover their purpose without reading the full CLI docs; Observable: each param row in `clp .credentials.status.help` has a non-empty description string; Scoped: only `.credentials.status` argument registration in `src/lib.rs` — no other commands, no new deps; Testable: `clp .credentials.status.help | grep -c "::"` returns 10).

The `.credentials.status` command accepts a `format::` string parameter and 9 boolean field-presence params (`account`, `sub`, `tier`, `token`, `expires`, `email`, `org`, `file`, `saved`). The current registration uses a bare `bf(nm)` lambda (`reg_arg_opt(nm, Kind::Boolean)`) which never calls `with_description()`, leaving all descriptions empty. The fix is a new lambda `bfd(nm, desc)` that appends `.with_description(desc)` and replaces every `bf(nm)` call in the `.credentials.status` registration block.

Expected descriptions (drawn from `docs/cli/params.md` §§ 6–14):

| Param     | Default | Description                                                          |
|-----------|---------|----------------------------------------------------------------------|
| `format`  | —       | Output format: `text` (default) or `json`                            |
| `account` | 1       | Active account name from `_active` marker; `N/A` if store absent    |
| `sub`     | 1       | Subscription type (e.g. `max`, `pro`)                                |
| `tier`    | 1       | Rate-limit tier identifier                                            |
| `token`   | 1       | Token validity status (`valid`, `expiring_soon`, `expired`)          |
| `expires` | 1       | Token expiry countdown (e.g. `in 7h 24m`)                           |
| `email`   | 1       | Email from `~/.claude/.claude.json`; `N/A` when absent              |
| `org`     | 1       | Org name from `~/.claude/.claude.json`; `N/A` when absent           |
| `file`    | 0       | Path to `~/.claude/.credentials.json` (opt-in)                      |
| `saved`   | 0       | Count of `*.credentials.json` files in credential store (opt-in)    |

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/lib.rs`
  § `register_commands()` lines 119–138 — add `bfd` lambda and replace `bf("account")` … `bf("saved")` with `bfd(nm, desc)` calls
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/account_list_status_test.rs`
  — add integration test asserting non-empty descriptions in help output

## Out of Scope

- Documentation updates (already completed by doc_tsk)
- `src/commands.rs` — no handler logic changes
- Custom columnar help formatter (separate follow-up if desired)
- Adding `data_fmt` or any new crate dependency
- All other commands' help output

## Requirements

- All work must strictly adhere to all applicable rulebooks
  (discover via `kbase .rulebooks`)
- Descriptions must match `docs/cli/params.md` §§ 6–14 as single source of truth;
  never paraphrase or invent content
- Description strings are inline Rust `&str` literals — no heap allocation, no `format!`
- The `bfd` lambda must be a closure defined inside `register_commands()`, not a new
  free function (keep consistent with existing `bf`, `fmt`, `v`, `nam` lambdas)
- New test must not rely on default-value details that may change — test for presence of
  the param name and a non-empty description string, not exact wording

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks` from the `claude_profile` module root;
   note `code_style.rulebook.md` constraints on lambda closure style and line length.

2. **Read source of truth** — Read
   `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/docs/cli/params.md`
   §§ 6–14 (lines ≈134–310) and
   `docs/feature/012_live_credentials_status.md` for the complete param list
   and their canonical descriptions.

3. **Read current registration** — Read
   `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/lib.rs`
   lines 119–145 to understand the existing `bf` / `reg_arg_opt` pattern and the
   `ArgumentDefinition::with_description()` API (from unilang).

4. **Write failing test** — In
   `tests/cli/account_list_status_test.rs`, add a test that calls
   `clp .credentials.status.help` via `run_cli` and asserts that the output
   contains each of the 9 boolean param names (`account`, `sub`, `tier`, `token`,
   `expires`, `email`, `org`, `file`, `saved`) followed by a non-empty description.
   The test must fail before the implementation (descriptions are currently blank).

5. **Add `bfd` lambda in `register_commands()`** — After the existing `bf` lambda
   (line 124), add:
   ```rust
   let bfd = | nm : &'static str, desc : &'static str |
     reg_arg_opt( nm, Kind::Boolean ).with_description( desc );
   ```
   Replace the `.credentials.status` `vec![ … ]` arguments block to use `bfd(nm, desc)`
   for each of the 9 boolean params. Keep `fmt()` unchanged (it has no description
   requirement). Keep all other commands' `bf()` usages unchanged.

6. **Validate** — Run `w3 .test level::3` inside Docker. All tests must pass,
   including the new help-description test.

7. **Walk Validation Checklist** — check every item. Every answer must be YES.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `clp .credentials.status.help` | default (no params) | Output contains `account::` with non-empty description |
| `clp .credentials.status.help` | default (no params) | Output contains `saved::` with non-empty description |
| `clp .credentials.status.help` | default (no params) | Output contains all 9 boolean param names |
| `clp .credentials.status.help` | default (no params) | Output contains `file::` with opt-in note (default: 0) |
| `clp .credentials.status sub::0 token::0` | runtime invocation | Command still executes normally (registration change only) |

## Acceptance Criteria

- `clp .credentials.status.help` output contains a non-empty description for each of
  the 9 boolean field-presence parameters
- `clp .credentials.status.help` output contains a non-empty description for `format::`
- Descriptions match the canonical text in `docs/cli/params.md` §§ 6–14 (no paraphrase)
- All existing `.credentials.status` functional tests continue to pass
- New test added to `tests/cli/account_list_status_test.rs` covering help description presence
- `w3 .test level::3` exits 0 with no warnings

## Validation

### Checklist

Desired answer for every question is YES.

**Help output correctness**
- [ ] Does `clp .credentials.status.help` show a non-empty description for `account::`?
- [ ] Does `clp .credentials.status.help` show a non-empty description for `sub::`?
- [ ] Does `clp .credentials.status.help` show a non-empty description for `tier::`?
- [ ] Does `clp .credentials.status.help` show a non-empty description for `token::`?
- [ ] Does `clp .credentials.status.help` show a non-empty description for `expires::`?
- [ ] Does `clp .credentials.status.help` show a non-empty description for `email::`?
- [ ] Does `clp .credentials.status.help` show a non-empty description for `org::`?
- [ ] Does `clp .credentials.status.help` show a non-empty description for `file::`?
- [ ] Does `clp .credentials.status.help` show a non-empty description for `saved::`?

**Source of truth fidelity**
- [ ] Does each description match the canonical text in `docs/cli/params.md` §§ 6–14?
- [ ] Do descriptions note the default value (1 or 0) for each param?

**Regression**
- [ ] Does `clp .credentials.status` (no params) still run and produce normal output?
- [ ] Does `clp .credentials.status account::0 token::1` still suppress/show lines correctly?

**Test coverage**
- [ ] Does `tests/cli/account_list_status_test.rs` contain a new test for help output?
- [ ] Does the new test fail before the implementation (i.e., is it a real failing test)?

**Out of Scope confirmation**
- [ ] Is `src/commands.rs` unchanged?
- [ ] Are all other commands' registrations (`bf()` uses) unchanged?
- [ ] Is `Cargo.toml` unchanged (no new deps)?

### Measurements

**M1 — Help output description presence**
Command: `clp .credentials.status.help | grep -c "Default:"`
Before: `0` (no descriptions rendered). Expected: `≥9`. Deviation: any value < 9.

**M2 — Full test suite**
Command: `w3 .test level::3`
Before: all pass (new test not yet added). Expected: `test result: ok. N passed`. Deviation: any failure.

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — no `spec.md` or `spec/` created in this dev crate
- [ ] I3 — no backup files (`*_backup.rs`, `*_old.rs`, etc.) created

### Anti-faking checks

**AF1 — `bfd` lambda exists in lib.rs**
Check: `grep -c "with_description" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/src/lib.rs`
Expected: `≥9`. Why: confirms each of the 9 boolean args received a description call, not just one token.

**AF2 — Test asserts content, not just exit code**
Check: `grep -c "account\|sub\|tier\|token\|expires\|email\|org\|file\|saved" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_profile/tests/cli/account_list_status_test.rs`
Expected: `≥9`. Why: confirms the test validates actual param names in output, not just a zero exit code.

## Outcomes

<!-- populated upon task completion -->

---

**Advisability:** 3 × 4 × 2 × 5 = **120**
(Value 3 — meaningful discoverability improvement; Easiness 4 — 9 string literals in one function; Priority 2 — UX polish, not blocking; Safety 5 — read-only output command, no state change)
