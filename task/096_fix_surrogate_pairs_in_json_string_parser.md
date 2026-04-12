# Handle UTF-16 surrogate pairs in parse_json_string_value

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 🎯 (Available)

## Goal

Fix `parse_json_string_value` in `commands.rs` to correctly decode UTF-16 surrogate pairs (`\uD800`–`\uDFFF`) instead of silently dropping or corrupting them, ensuring emoji and supplementary-plane characters received from the GitHub Releases API survive into the displayed changelog, verified by `w3 .test level::3`. (Motivated: GitHub release bodies routinely contain emoji encoded as surrogate pairs in JSON (`\uD83D\uDE80` = 🚀); these are currently silently dropped, producing corrupted history output; Observable: `parse_json_string_value` reassembles surrogate pairs into the correct Unicode scalar value; Scoped: only `parse_json_string_value` in `commands.rs`; Testable: `cargo nextest run --test integration --features enabled -E 'test(surrogate)'`)

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_manager/src/commands.rs` — `parse_json_string_value`: detect lead surrogate (`\uD800`–`\uDBFF`), consume trailing surrogate (`\uDC00`–`\uDFFF`), compute the supplementary code point `0x10000 + (lead - 0xD800) * 0x400 + (trail - 0xDC00)`, encode as UTF-8; return error for lone surrogates
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_manager/tests/integration/read_commands_test.rs` — add TC-460: `parse_json_string_value` with `\uD83D\uDE80` decodes to `🚀`; TC-461: lone lead surrogate returns error; TC-462: lone trail surrogate returns error

## Out of Scope

- Changing `extract_releases` tag-prefix parsing (covered in TSK-097)
- Adding a full RFC 8259-compliant JSON parser (only surrogate pair gap filled)
- Other `\u` escape sequences already handled correctly

## Description

`parse_json_string_value` processes `\uXXXX` escapes by converting 4 hex digits directly to a `char`, but never handles surrogate pairs. A surrogate pair consists of two consecutive `\uXXXX` escapes: a lead surrogate (U+D800–U+DBFF) followed by a trail surrogate (U+DC00–U+DFFF). Together they encode a supplementary Unicode code point above U+FFFF. GitHub release bodies routinely contain emoji as surrogate pairs (`\uD83D\uDE80` = 🚀), which are currently silently dropped or corrupted, producing garbled `.version.history` output.

The fix detects when a decoded `\uXXXX` is a lead surrogate, consumes the required trail surrogate, and computes the supplementary code point as `0x10000 + (lead - 0xD800) * 0x400 + (trail - 0xDC00)`, then encodes it as UTF-8. Lone surrogates (a lead without a trail, or a trail without a lead) are malformed JSON and must return an error.

All existing BMP `\uXXXX` behaviour (U+0000–U+D7FF, U+E000–U+FFFF) is unchanged by this fix.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   TDD: write failing tests before implementing; confirm they fail before fixing
-   Unit-level tests may call `parse_json_string_value` directly (it is a `pub(crate)` or internal function; expose only as needed for testing)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note test_organisation rules on test placement and doc comment format.
2. **Write Test Matrix** — populate all rows below before opening any test file.
3. **Write failing tests** — add TC-460, TC-461, TC-462 to `read_commands_test.rs` (or a dedicated unit test if `parse_json_string_value` is not pub); run suite, confirm failures.
4. **Read source** — read the full `parse_json_string_value` implementation; identify exactly where `\u` escapes are processed; note how the current code handles `\u` (likely a simple 4-hex-digit decode without surrogate awareness).
5. **Implement** — after decoding a `\uXXXX` escape, check if the resulting `u16` is a lead surrogate; if so, require the next token to be `\uXXXX` in the trail range, compute the scalar, encode as UTF-8 char; if the trail is missing or out of range, return an error.
6. **Green state** — `w3 .test level::3` must pass with zero failures and zero warnings.
7. **Submit for Validation** — trigger SUBMIT transition.
8. **Update task status** — on validation pass set ✅ in `task/readme.md`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `"\uD83D\uDE80"` (surrogate pair for 🚀) | surrogate pair decode | returns `"🚀"` |
| T02 | `"\uD83C\uDF89"` (🎉) | another pair | returns `"🎉"` |
| T03 | `"\uD800"` (lone lead surrogate) | error case | returns Err |
| T04 | `"\uDC00"` (lone trail surrogate) | error case | returns Err |
| T05 | `"\uD800\uD800"` (lead followed by another lead) | malformed pair | returns Err |
| T06 | `"hello \uD83D\uDE80 world"` | surrogate in context | returns `"hello 🚀 world"` |
| T07 | `"\u0041"` (A, no surrogate) | existing BMP behavior unchanged | returns `"A"` |

## Acceptance Criteria

-   `parse_json_string_value("\uD83D\uDE80")` returns `Ok("🚀")`
-   `parse_json_string_value("\uD800")` returns `Err` (lone lead)
-   `parse_json_string_value("\uDC00")` returns `Err` (lone trail)
-   Existing `\uXXXX` behavior for BMP code points (U+0000–U+D7FF, U+E000–U+FFFF) unchanged
-   T01–T07 all pass
-   `w3 .test level::3` passes with zero failures

## Validation

### Checklist

Desired answer for every question is YES.

**Surrogate pair handling**
- [ ] C1 — Does `parse_json_string_value("\uD83D\uDE80")` return `Ok("🚀")`?
- [ ] C2 — Does the implementation check `>= 0xD800 && <= 0xDBFF` to detect lead surrogates?
- [ ] C3 — Does a lone lead surrogate return `Err`?
- [ ] C4 — Does a lone trail surrogate return `Err`?
- [ ] C5 — Is existing BMP `\u` behavior unchanged (TC-T07 passes)?

**Out of Scope confirmation**
- [ ] C6 — Is `extract_releases` tag parsing logic unchanged?

### Measurements

- [ ] M1 — surrogate decode: unit test `tc460_surrogate_pair_rocket` passes (was: would produce garbled output or panic)
- [ ] M2 — lone lead error: unit test `tc461_lone_lead_surrogate_error` passes (was: would silently produce garbage char)

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --features enabled` → 0 warnings

### Anti-faking checks

- [ ] AF1 — surrogate detection code: `grep -n "0xD800\|0xDBFF\|0xDC00\|0xDFFF" src/commands.rs` → at least 2 matches (lead and trail range checks present)
- [ ] AF2 — pair computation: `grep -n "0x10000" src/commands.rs` → at least 1 match (supplementary codepoint formula present)

## Outcomes

[Added upon task completion.]
