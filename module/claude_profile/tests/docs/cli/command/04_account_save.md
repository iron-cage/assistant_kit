# Test: `.account.save`

Integration test planning for the `.account.save` command. See [commands.md](../../../../docs/cli/commands.md#command--4-accountsave) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Save creates credential file in `~/.persistent/claude/credential/` | Basic Invocation |
| IT-2 | Save creates credential store if missing | Directory Init |
| IT-3 | Save with existing name overwrites silently | Overwrite |
| IT-4 | Save with non-email name exits 1 | Validation |
| IT-5 | Save with empty `name::` exits 1 | Validation |
| IT-6 | Save when `~/.claude/.credentials.json` missing exits 2 | Error Handling |
| IT-7 | `dry::1` prints action without creating file | Dry Run |
| IT-8 | `dry::1` then `dry::0` creates file as previewed | Dry Run Fidelity |
| IT-9 | Saved file content matches active credentials exactly | Data Integrity |
| IT-10 | Missing `name::` parameter exits 1 | Required Param |
| IT-11 | Save creates `{name}.claude.json` and `{name}.settings.json` snapshots when both sources exist | Metadata Snapshot |
| IT-12 | Save succeeds when `~/.claude.json` absent — only credential file created | Metadata Snapshot / Best-Effort |
| IT-13 | Save succeeds when `settings.json` absent — credential + `.claude.json` created, no `.settings.json` | Metadata Snapshot / Best-Effort |

### Test Coverage Summary

- Basic Invocation: 1 test
- Directory Init: 1 test
- Overwrite: 1 test
- Validation: 2 tests
- Error Handling: 1 test
- Dry Run: 2 tests
- Data Integrity: 1 test
- Required Param: 1 test
- Metadata Snapshot: 1 test
- Metadata Snapshot / Best-Effort: 2 tests

**Total:** 13 integration tests

---

### IT-1: Save creates credential file in `~/.persistent/claude/credential/`

- **Given:** Create `~/.claude/.credentials.json` with valid credential content. Create `~/.persistent/claude/credential/` directory (empty).
- **When:** `clp .account.save name::work@acme.com`
- **Then:** stdout: `saved current credentials as 'work@acme.com'`. File `~/.persistent/claude/credential/work@acme.com.credentials.json` now exists.; credential file created at expected path
- **Exit:** 0
- **Source:** [commands.md — .account.save](../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-2: Save creates credential store if missing

- **Given:** Create `~/.claude/.credentials.json` with valid credential content. Ensure `~/.persistent/claude/credential/` does not exist.
- **When:** `clp .account.save name::work@acme.com`
- **Then:** stdout: `saved current credentials as 'work@acme.com'`. Both `~/.persistent/claude/credential/` directory and `~/.persistent/claude/credential/work@acme.com.credentials.json` file now exist.; directory auto-created; credential file created
- **Exit:** 0
- **Source:** [commands.md — .account.save](../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-3: Save with existing name overwrites silently

- **Given:** Create `~/.claude/.credentials.json` with credential content V2. Create `~/.persistent/claude/credential/work@acme.com.credentials.json` with older credential content V1.
- **When:** `clp .account.save name::work@acme.com`
- **Then:** stdout: `saved current credentials as 'work@acme.com'`. File `~/.persistent/claude/credential/work@acme.com.credentials.json` now contains V2 content.; file overwritten with current credentials; no error output
- **Exit:** 0
- **Source:** [commands.md — .account.save](../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-4: Save with non-email name exits 1

- **Given:** Create `~/.claude/.credentials.json` with valid credential content.
- **When:** `clp .account.save name::notanemail`
- **Then:** Error message on stderr indicating the name must be a valid email address. No file created.; no file created; error message references email format
- **Exit:** 1
- **Source:** [commands.md — .account.save](../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-5: Save with empty `name::` exits 1

- **Given:** Create `~/.claude/.credentials.json` with valid credential content.
- **When:** `clp .account.save name::`
- **Then:** Error message indicating the account name must not be empty. No file created.; no file created; error message about empty name
- **Exit:** 1
- **Source:** [commands.md — .account.save](../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-6: Save when `~/.claude/.credentials.json` missing exits 2

- **Given:** Ensure `~/.claude/.credentials.json` does not exist. Create `~/.persistent/claude/credential/` directory.
- **When:** `clp .account.save name::work@acme.com`
- **Then:** Error message indicating credentials file is unreadable or missing. No account file created.; no file created; error message about missing credentials
- **Exit:** 2
- **Source:** [commands.md — .account.save](../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-7: `dry::1` prints action without creating file

- **Given:** Create `~/.claude/.credentials.json` with valid credential content. Create `~/.persistent/claude/credential/` directory (empty).
- **When:** `clp .account.save name::work@acme.com dry::1`
- **Then:** stdout: `[dry-run] would save current credentials as 'work@acme.com'`. No file created.; dry-run message printed; no file created on disk
- **Exit:** 0
- **Source:** [commands.md — .account.save](../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-8: `dry::1` then `dry::0` creates file as previewed

- **Given:** Create `~/.claude/.credentials.json` with valid credential content. Create `~/.persistent/claude/credential/` directory (empty).
- **When:** `clp .account.save name::work@acme.com dry::1` followed by `clp .account.save name::work@acme.com`
- **Then:** First command: `[dry-run] would save current credentials as 'work@acme.com'` (no file). Second command: `saved current credentials as 'work@acme.com'` (file created).; on both; dry run creates nothing; real run creates exactly the previewed file
- **Exit:** 0
- **Source:** [commands.md — .account.save](../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-9: Saved file content matches active credentials exactly

- **Given:** Create `~/.claude/.credentials.json` with known credential content (e.g., specific JSON with `accessToken`, `refreshToken`, `expiresAt` fields).
- **When:** `clp .account.save name::work@acme.com`
- **Then:** `~/.persistent/claude/credential/work@acme.com.credentials.json` is a byte-identical copy of `~/.claude/.credentials.json`.; saved file is byte-identical to source credentials
- **Exit:** 0
- **Source:** [commands.md — .account.save](../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-10: Missing `name::` parameter exits 1

- **Given:** Create `~/.claude/.credentials.json` with valid credential content.
- **When:** `clp .account.save`
- **Then:** Error message indicating the `name::` parameter is required. No file created.; no file created; error message about required parameter
- **Exit:** 1
- **Source:** [commands.md — .account.save](../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-11: Save creates metadata snapshots when both source files exist

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. `~/.claude.json` exists with `oauthAccount.displayName = "alice"`. `~/.claude/settings.json` exists with `model = "sonnet"`.
- **When:** `clp .account.save name::work@acme.com`
- **Then:** `{credential_store}/work@acme.com.claude.json` created (copy of `~/.claude.json`); `{credential_store}/work@acme.com.settings.json` created (copy of `settings.json`); both files contain correct content.; metadata snapshot files created alongside credential file
- **Exit:** 0
- **Source:** [commands.md — .account.save](../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-12: Save succeeds when `~/.claude.json` absent — best-effort snapshot

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. `~/.claude.json` does NOT exist.
- **When:** `clp .account.save name::work@acme.com`
- **Then:** Credential file created. No `.claude.json` snapshot created. No error emitted; save succeeds silently despite missing source.; save completes successfully; no error for absent optional source
- **Exit:** 0
- **Source:** [commands.md — .account.save](../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-13: Save succeeds when `settings.json` absent — best-effort snapshot

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. `~/.claude.json` exists. `~/.claude/settings.json` does NOT exist.
- **When:** `clp .account.save name::work@acme.com`
- **Then:** Credential file created. `{credential_store}/work@acme.com.claude.json` created. No `.settings.json` snapshot created. No error emitted.; both present files snapshotted; absent source silently skipped
- **Exit:** 0
- **Source:** [commands.md — .account.save](../../../../docs/cli/commands.md#command--4-accountsave)
