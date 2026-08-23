# Parameter :: `session_ids::`

Edge case tests for the `session_ids::` parameter. Tests validate `.cost`'s comma-separated conversation selector — exact and prefix resolution across all projects, the three rejection paths (ambiguous prefix, unknown ID, empty list), request-order row emission with duplicate collapse, the cross-project duplicate tie-break, and the default resolution that applies when the parameter is omitted.

**Cross-project by design:** unlike [`session_id::`](14_session_id.md)'s single-command exact lookup and [`session::`](13_session.md)'s per-scope filtering, resolution here searches every project — a conversation is addressable from anywhere, no [`path::`](09_path.md) needed.

**Source:** [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Unique prefix resolves to its session | Happy Path |
| EC-2 | Two IDs → one row each, plus a `TOTAL` row | Happy Path |
| EC-3 | Ambiguous prefix rejected | Input Validation |
| EC-4 | Unknown ID rejected | Input Validation |
| EC-5 | Empty `session_ids::` rejected | Input Validation |
| EC-6 | Duplicate requests collapse to one row | Boundary Values |
| EC-7 | Cross-project duplicate ID resolves to the richest copy | Boundary Values |
| EC-8 | Omitted → most recent non-agent session of the resolved project | Default |
| EC-9 | Omitted with no project at cwd → exit 2 | Exit Codes |

## Test Coverage Summary

- Happy Path: 2 tests (EC-1, EC-2)
- Input Validation: 3 tests (EC-3, EC-4, EC-5)
- Boundary Values: 2 tests (EC-6, EC-7)
- Default: 1 test (EC-8)
- Exit Codes: 1 test (EC-9)

**Total:** 9 edge cases

**Behavioral Divergence Pair:** EC-1 (one conversation — single row, no `TOTAL`) ↔ EC-2 (two conversations — a `TOTAL` row appears)

## Test Cases

---

### EC-1: Unique prefix resolves to its session

- **Commands:** `.cost`
- **Given:** a storage whose sessions have distinct 8-character ID prefixes
- **When:** `clg .cost session_ids::feed0011`
- **Then:** the prefix resolves to the one matching session; its row is reported
- **Exit:** 0
- **Covered by:** `cli_cmd_cost_test.rs` — `cost_int_3_unique_prefix_resolves`
- **Source:** [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)

---

### EC-2: Two IDs → one row each, plus a `TOTAL` row

- **Commands:** `.cost`
- **Given:** two resolvable conversations
- **When:** `clg .cost session_ids::aaaa1111,bbbb2222`
- **Then:** one row per conversation in request order, followed by a `TOTAL` row — which a single-conversation invocation does not emit
- **Exit:** 0
- **Covered by:** `cli_cmd_cost_test.rs` — `cost_int_2_multi_row_total_row`
- **Source:** [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)

---

### EC-3: Ambiguous prefix rejected

- **Commands:** `.cost`
- **Given:** two sessions sharing the requested prefix
- **When:** `clg .cost session_ids::<shared-prefix>`
- **Then:** Exit 1; the error lists every matching session ID, sorted — an ambiguous selector is never silently resolved to one of the candidates
- **Exit:** 1
- **Covered by:** `cli_cmd_cost_test.rs` — `cost_int_4_ambiguous_prefix_rejected`
- **Source:** [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)

---

### EC-4: Unknown ID rejected

- **Commands:** `.cost`
- **Given:** a populated storage
- **When:** `clg .cost session_ids::<no-such-id>`
- **Then:** Exit 1; the error names the unresolvable request
- **Exit:** 1
- **Covered by:** `cli_cmd_cost_test.rs` — `cost_int_5_unknown_id_rejected`
- **Source:** [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)

---

### EC-5: Empty `session_ids::` rejected

- **Commands:** `.cost`
- **Given:** clean environment
- **When:** `clg .cost session_ids::` (or a value splitting to zero non-empty elements)
- **Then:** Exit 1; error indicating at least one session ID is required — raised before any storage access, so it does not depend on what the storage contains
- **Exit:** 1
- **Covered by:** `cli_cmd_cost_test.rs` — `cost_int_9_empty_session_ids_rejected`
- **Source:** [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)

---

### EC-6: Duplicate requests collapse to one row

- **Commands:** `.cost`
- **Given:** one conversation, requested twice in the same list
- **When:** `clg .cost session_ids::<id>,<id>`
- **Then:** one row, at the first occurrence's position — the conversation is not double-counted into the `TOTAL`
- **Exit:** 0
- **Covered by:** `cli_cmd_cost_test.rs` — `cost_int_13_duplicate_requests_collapse`
- **Source:** [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)

---

### EC-7: Cross-project duplicate ID resolves to the richest copy

- **Commands:** `.cost`
- **Given:** the same session ID present under two project directories with differing entry counts
- **When:** `clg .cost session_ids::<duplicated-id>`
- **Then:** the copy with the greatest entry count wins, so totals are not understated by picking an arbitrary duplicate (`Fix(BUG-528)` tie-break)
- **Exit:** 0
- **Covered by:** `cli_cmd_cost_test.rs` — `cost_int_14_cross_project_duplicate_picks_richest`
- **Source:** [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)

---

### EC-8: Omitted → most recent non-agent session of the resolved project

- **Commands:** `.cost`
- **Given:** a project at cwd with a session
- **When:** `clg .cost` with no `session_ids::`
- **Then:** the project's most recent non-agent session is reported as a single row, with no `TOTAL` row
- **Exit:** 0
- **Covered by:** `cli_cmd_cost_test.rs` — `cost_int_1_default_current_single_row_no_total`
- **Source:** [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)

---

### EC-9: Omitted with no project at cwd → exit 2

- **Commands:** `.cost`
- **Given:** a valid but empty storage, and a cwd owning no project
- **When:** `clg .cost` with no `session_ids::`
- **Then:** Exit 2 with `No project found for current directory` on stderr — "nothing to resolve" is distinguished from the exit 1 argument errors above
- **Exit:** 2
- **Covered by:** `cli_cmd_cost_test.rs` — `cost_int_11_no_project_exits_2`
- **Source:** [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)
