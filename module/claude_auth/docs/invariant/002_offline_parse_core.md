# Invariant: Offline Parse Core

### Scope

- **Purpose**: Guarantee that everything except the single network call stays reachable, testable, and side-effect-free with no feature enabled.
- **Responsibility**: State which surface is unconditional, why `now_ms` is a parameter, and how the split is enforced.
- **In Scope**: Unconditional surface (INV-1), no I/O in the parse path (INV-2), no clock read in the parse path (INV-3), offline test suite (INV-4).
- **Out of Scope**: The dependency shape that makes this possible (→ `001_zero_workspace_deps.md`).

### Invariant Statement

| ID | Invariant |
|----|-----------|
| INV-1 | `TOKEN_URL`, `CLIENT_ID`, `TokenRefreshResult`, `AuthError`, and `parse_response` are available with no feature enabled |
| INV-2 | The parse path performs no network, filesystem, or process I/O |
| INV-3 | The parse path reads no clock — `now_ms` is supplied by the caller |
| INV-4 | The whole test suite runs with no features, no network, and no `ureq` linked |

**Why INV-3 is an invariant and not a style choice.** `parse_response` computes an absolute
`expires_at_ms` from a relative `expires_in`, which needs a "now". Reading the clock inside the
function would make its output non-deterministic and its expiry arithmetic untestable —
a test could only assert a range, never a value. Taking `now_ms` as a parameter makes the
arithmetic a pure function, which is what lets `tests/auth_test.rs` T01 assert an exact
`expires_at_ms`. `refresh_token` reads the clock once and passes it in; that one call site is
the only impure point in the crate.

### Enforcement Mechanism

**INV-1** — the crate contains exactly one feature gate, and it is the one on `refresh_token`:

```bash
grep -n 'cfg( feature' module/claude_auth/src/lib.rs
# Expected: exactly one line — the `#[ cfg( feature = "enabled" ) ]` attribute
#   on `refresh_token`. Every other public item is therefore unconditional.
```

Note the gate is separated from `pub fn refresh_token` by an `#[ inline ]` attribute, so a
`-B1` context grep will not pair them — count the gates rather than trying to match the pair.

**INV-2 / INV-3** — `ureq` and the clock appear only inside the gated function:

```bash
grep -nE 'ureq|SystemTime|UNIX_EPOCH' module/claude_auth/src/lib.rs
# Expected: one hit in the crate-level doc comment (the feature table), and all
#   remaining hits at line numbers inside the `refresh_token` body. None in
#   parse_response, parse_string_field, or parse_u64_field.
```

**INV-4** — the suite passes with the default (empty) feature set, and `[dev-dependencies]`
is empty:

```bash
cargo nextest run -p claude_auth --no-default-features
```

### Violation Consequences

- **INV-1 violated:** A consumer that only needs to name the endpoint, match on `AuthError`, or
  parse a body it obtained elsewhere would be forced to link an HTTP stack.
- **INV-2 violated:** The offline test suite stops being offline. Tests become network-flaky and
  — per [feature/001_token_refresh.md](../feature/001_token_refresh.md) FR-5 — subject to the
  very rate limiting this crate exists to report.
- **INV-3 violated:** T01 can no longer assert an exact `expires_at_ms`, so the one piece of
  arithmetic in the crate loses its regression test.
- **INV-4 violated:** CI cannot verify the crate without network access, and a `ureq` breaking
  change starts failing tests that have nothing to do with HTTP.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [001_zero_workspace_deps.md](001_zero_workspace_deps.md) | The optional-dep shape that makes this split possible |
| doc | [feature/002_response_parsing.md](../feature/002_response_parsing.md) | The feature this invariant keeps unconditional |
| source | `../../src/lib.rs` | Implementation that must satisfy INV-1 through INV-3 |
| test | `../../tests/auth_test.rs` | T01–T06, all offline; header records the `refresh_token` coverage gap as `N/A` |
