# Parameter :: `latest::`

Edge case tests for the `latest::` parameter. Tests validate `.session.path`'s only disk-reading selector — newest-mtime resolution among qualifying session files — its equivalence to the bare default form, the empty-storage exit code, and mutual exclusion against the other two selectors.

**Qualifying file:** a `*.jsonl` in the resolved storage that is neither an `agent-*` file nor zero-length.

**Source:** [param/41_latest.md](../../../../docs/cli/param/41_latest.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | No selector → latest is the effective default | Default |
| EC-2 | `latest::1` explicit → identical to the default form | Happy Path |
| EC-3 | Latest picks the newer of two sessions | Happy Path |
| EC-4 | Empty storage → exit 2 | Exit Codes |
| EC-5 | `latest::` is mutually exclusive with `session::` and `topic::` | Input Validation |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Happy Path: 2 tests (EC-2, EC-3)
- Exit Codes: 1 test (EC-4)
- Input Validation: 1 test (EC-5)

**Total:** 5 edge cases

**Behavioral Divergence Pair:** EC-3 (a qualifying session exists — its path on stdout, exit 0) ↔ EC-4 (none exists — "no sessions" on stderr, exit 2)

## Test Cases

---

### EC-1: No selector → latest is the effective default

- **Commands:** `.session.path`
- **Given:** a storage holding at least one qualifying session file
- **When:** `clg .session.path` with no selector
- **Then:** the most recently modified qualifying session file's absolute path is printed
- **Exit:** 0
- **Covered by:** `cli_cmd_session_path_test.rs` — `sp_1_default_selector_resolves_latest`
- **Source:** [param/41_latest.md](../../../../docs/cli/param/41_latest.md)

---

### EC-2: `latest::1` explicit → identical to the default form

- **Commands:** `.session.path`
- **Given:** the same storage as EC-1
- **When:** `clg .session.path latest::1`
- **Then:** the result matches the bare default form exactly — the explicit spelling exists for script readability, and changes nothing about resolution
- **Exit:** 0
- **Covered by:** `cli_cmd_session_path_test.rs` — `sp_2_latest_explicit_matches_default`
- **Source:** [param/41_latest.md](../../../../docs/cli/param/41_latest.md)

---

### EC-3: Latest picks the newer of two sessions

- **Commands:** `.session.path`
- **Given:** two qualifying session files with distinct modification times
- **When:** `clg .session.path latest::1`
- **Then:** the newer file's path is printed — resolution is by mtime, not by name or creation order
- **Exit:** 0
- **Covered by:** `cli_cmd_session_path_test.rs` — `sp_4_latest_picks_newer_session`
- **Source:** [param/41_latest.md](../../../../docs/cli/param/41_latest.md)

---

### EC-4: Empty storage → exit 2

- **Commands:** `.session.path`
- **Given:** a valid storage holding no qualifying session file
- **When:** `clg .session.path latest::1`
- **Then:** Exit 2 with `no sessions in {storage}` on stderr — "nothing to resolve" is distinguished from the exit 1 usage errors
- **Exit:** 2
- **Covered by:** `cli_cmd_session_path_test.rs` — `sp_3_empty_storage_exits_2`
- **Source:** [param/41_latest.md](../../../../docs/cli/param/41_latest.md)

---

### EC-5: `latest::` is mutually exclusive with `session::` and `topic::`

- **Commands:** `.session.path`
- **Given:** clean environment
- **When:** `clg .session.path latest::1 topic::review` (and any other selector pair)
- **Then:** Exit 1 — more than one selector is an argument error; `latest::` does not silently lose to, or override, a competing selector
- **Exit:** 1
- **Covered by:** `cli_cmd_session_path_test.rs` — `sp_7_selectors_mutually_exclusive`
- **Source:** [param/41_latest.md](../../../../docs/cli/param/41_latest.md)
