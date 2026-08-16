# Format 003: table

FM test cases for `docs/cli/format/003_table.md`. Verifies the `format::table` output
contract: exclusive `.accounts` scope, table structural layout, flag column semantics
(`✓`/`*`/`@`), and field-presence param ignorance in table mode.

**Source:** [docs/cli/format/003_table.md](../../../../docs/cli/format/003_table.md)

### FM Case Index

| ID | Short Name | Category | Status |
|----|------------|----------|--------|
| FM-1 | `format::table` is accepted only by `.accounts` — all other commands reject with exit 1 | Scope Restriction | ✅ |
| FM-2 | Table output includes the `Account` header and one row per account | Structure | ✅ |
| FM-3 | `✓`/`*` flag priority is documented but not asserted by the cited test | Flag Semantics | ✅ |
| FM-4 | `format::table` shows `Account` header; field-presence suppression untested by cited test | Field Presence | ✅ |

**Behavioral Divergence Pair:** FM-1 (`.accounts format::table` → exit 0 with table) ↔ FM-1 (`.usage format::table` → exit 1, rejected) — the same `format::table` parameter produces success on `.accounts` and failure on all other commands.

---

### FM-1: `format::table` accepted only by `.accounts` — all others exit 1

- **Given:** `.usage format::table`, `.paths format::table`, `.credentials.status format::table`
- **When:** Each command is invoked
- **Then:** All exit 1 with an error message like `unknown format 'table': expected text or json` — only `.accounts format::table` exits 0
- **Source fn:** `acc34_accounts_table_format` (cli/accounts_list_test_b.rs; verifies table accepted for .accounts)
- **Source:** [docs/cli/format/003_table.md §Scope](../../../../docs/cli/format/003_table.md)

---

### FM-2: Table output includes the `Account` column header and one row per account

- **Given:** `.accounts format::table` with two saved accounts (`alice@acme.com`, `work@acme.com`)
- **When:** Output is captured
- **Then:** Output exits 0 and contains the `Account` column-header label plus both account email addresses — the test does not assert on the title line, blank-line separator, dash separator row, or the exact ordering of the five structural sections the product doc describes
- **Source fn:** `acc34_accounts_table_format` (cli/accounts_list_test_b.rs)
- **Source:** [docs/cli/format/003_table.md §Structure](../../../../docs/cli/format/003_table.md)

---

### FM-3: `✓`/`*` flag priority is documented but not asserted by the cited table-format test

- **Given:** `acc34_accounts_table_format` saves two accounts — `alice@acme.com` (not active-marked) and `work@acme.com` (active-marked via the credential-store active marker) — and sets up no live `~/.claude/.credentials.json` session
- **When:** `.accounts format::table` output is captured
- **Then:** Output exits 0 and contains the `Account` header plus both email addresses; the test never writes live credentials and never asserts on `✓`, `*`, `@`, or blank flag-column output — the `✓` > `*` > `@` > blank priority claim is not verified by this test, nor by any other test in the suite (no test combines `format::table` with a live session)
- **Source fn:** `acc34_accounts_table_format` (cli/accounts_list_test_b.rs)
- **Source:** [docs/cli/format/003_table.md §Flag semantics](../../../../docs/cli/format/003_table.md)

---

### FM-4: `Account` header appears in table mode — field-presence suppression is not exercised by this test

- **Given:** `.accounts format::table` (no `sub::`/`tier::`/`email::` params passed — the cited test never invokes them)
- **When:** The command runs
- **Then:** Table output contains the `Account` column header; the test does not pass `sub::0`/`tier::0`/`email::0` and does not assert on the `Sub`, `Tier`, `Expires`, or `Email` columns individually, so the "field-presence params ignored, all columns always appear" claim is not verified by this test
- **Source fn:** `acc34_accounts_table_format` (cli/accounts_list_test_b.rs)
- **Source:** [docs/cli/format/003_table.md §Notes](../../../../docs/cli/format/003_table.md)
