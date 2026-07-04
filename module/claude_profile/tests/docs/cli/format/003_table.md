# Format 003: table

FM test cases for `docs/cli/format/003_table.md`. Verifies the `format::table` output
contract: exclusive `.accounts` scope, table structural layout, flag column semantics
(`✓`/`*`/`@`), and field-presence param ignorance in table mode.

**Source:** [docs/cli/format/003_table.md](../../../../docs/cli/format/003_table.md)

### FM Case Index

| ID | Short Name | Category | Status |
|----|------------|----------|--------|
| FM-1 | `format::table` is accepted only by `.accounts` — all other commands reject with exit 1 | Scope Restriction | ✅ |
| FM-2 | Table output structure: title, blank line, header row, separator row, data rows | Structure | ✅ |
| FM-3 | `✓` flag marks the current (live session) account; `*` marks active-marker-but-not-current | Flag Semantics | ✅ |
| FM-4 | Field-presence params are ignored in table mode — all columns always appear | Field Presence | ✅ |

**Behavioral Divergence Pair:** FM-1 (`.accounts format::table` → exit 0 with table) ↔ FM-1 (`.usage format::table` → exit 1, rejected) — the same `format::table` parameter produces success on `.accounts` and failure on all other commands.

---

### FM-1: `format::table` accepted only by `.accounts` — all others exit 1

- **Given:** `.usage format::table`, `.paths format::table`, `.credentials.status format::table`
- **When:** Each command is invoked
- **Then:** All exit 1 with an error message like `unknown format 'table': expected text or json` — only `.accounts format::table` exits 0
- **Source fn:** `acc34_accounts_table_format` (cli/accounts_list_test_b.rs; verifies table accepted for .accounts)
- **Source:** [docs/cli/format/003_table.md §Scope](../../../../docs/cli/format/003_table.md)

---

### FM-2: Table output has: title, blank line, header row, separator row, data rows

- **Given:** `.accounts format::table` with at least one saved account
- **When:** Output is captured
- **Then:** Output contains exactly: title line (`Accounts`), blank line, header row (column names), separator row (`-` chars), and one data row per account — five structural sections in order
- **Source fn:** `acc34_accounts_table_format` (cli/accounts_list_test_b.rs)
- **Source:** [docs/cli/format/003_table.md §Structure](../../../../docs/cli/format/003_table.md)

---

### FM-3: `✓` flag marks current live session account; `*` marks active-but-not-current

- **Given:** Two accounts: one active in the live `~/.claude/.credentials.json` session (`✓`) and one marked active in the credential store but NOT matching the current live session (`*`)
- **When:** `.accounts format::table` output is captured
- **Then:** The live session account has `✓` in the flag column; the non-current active marker account has `*`; all other accounts have a space — flag priority: `✓` > `*` > `@` > blank
- **Source fn:** `acc34_accounts_table_format` (cli/accounts_list_test_b.rs)
- **Source:** [docs/cli/format/003_table.md §Flag semantics](../../../../docs/cli/format/003_table.md)

---

### FM-4: Field-presence params are ignored in table mode — all columns always appear

- **Given:** `.accounts format::table sub::0 tier::0 email::0`
- **When:** The command runs
- **Then:** Table output contains all columns (Account, Sub, Tier, Expires, Email) regardless of the suppression params — `format::table` overrides field-presence toggles just like `format::json`
- **Source fn:** `acc34_accounts_table_format` (cli/accounts_list_test_b.rs; all columns present in table output)
- **Source:** [docs/cli/format/003_table.md §Notes](../../../../docs/cli/format/003_table.md)
