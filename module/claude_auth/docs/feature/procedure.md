# Feature Documentation Operations

- **Actor:** Developer
- **Trigger:** A new feature is added or an existing one is significantly changed.
- **Emits:** —

## Rule

A change to `TOKEN_URL`, `CLIENT_ID`, the request body's key set, or the HTTP status mapping is
a **wire-protocol change**, not an implementation detail — it changes what a live Anthropic
endpoint will accept. Record the new shape in [001_token_refresh.md](001_token_refresh.md) in
the same change that edits `src/lib.rs`; never let the constant and the doc disagree, because
the wire behavior cannot be verified by the offline test suite.

A change to which JSON fields are required, or to how a value is extracted, belongs in
[002_response_parsing.md](002_response_parsing.md) and must ship with a `tests/auth_test.rs`
case — that surface *is* fully testable offline, so an untested parsing change is a gap, not a
tradeoff.

## Add Feature Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Write the Acceptance Criteria section naming the `tests/auth_test.rs` IDs that cover it, or stating explicitly why no test can
4. Register in `readme.md` Overview Table: add row with ID, Name, Purpose, Status
5. Increment the `feature/` instance count in `../entity.md` and add a Master Doc Instances row

## Update Feature Documentation

1. Edit the target `NNN_*.md` file
2. If the covering test IDs changed: update the Acceptance Criteria table to match `tests/auth_test.rs`
3. If name or purpose changed: update `readme.md` Overview Table row and `../entity.md`

## Example

Adding feature document `003_scope_negotiation`:

1. Check `readme.md` Overview Table — current highest ID is `002`
2. Create `003_scope_negotiation.md` in this directory
3. Acceptance Criteria: name the new `tests/auth_test.rs` cases, e.g. T07–T09
4. Add row: `| 003 | [Scope Negotiation](003_scope_negotiation.md) | Requested vs granted OAuth scope handling | ✅ |`
5. Bump `feature/` instances to 3 in `../entity.md` and add the Master Doc Instances row
