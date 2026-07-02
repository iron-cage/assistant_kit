# Research 001: Controlling the Claude Interactive Session

RC test cases for `docs/research_interactive/001_claude_interactive_session_control.md`.
Verifies testable properties of `.account.relogin` behavior discovered via research into
`claude` binary execution modes: parameter acceptance, dry-run isolation, name resolution,
and the constraint that browser auth flows cannot be verified in automated tests.

**Source:** [docs/research_interactive/001_claude_interactive_session_control.md](../../../../docs/research_interactive/001_claude_interactive_session_control.md)

### RC Case Index

| ID | Short Name | Category | Status |
|----|------------|----------|--------|
| RC-1 | `.account.relogin dry::1` exits 0 without spawning subprocess | Dry-Run Isolation | ✅ |
| RC-2 | `.account.relogin` with unknown account name exits 2 | Name Resolution | ✅ |
| RC-3 | `.account.relogin` resolves account name by prefix | Name Resolution | ✅ |
| RC-4 | Mode 2 (`--print`) constraint: non-TTY cannot trigger browser OAuth | Execution Mode Constraint | ✅ |

**Behavioral Divergence Pair:** RC-1 (`dry::1` → exits 0, no subprocess spawn) ↔ RC-4 (live relogin → spawns subprocess requiring TTY) — the two cases produce observably different subprocess behavior from the same command.

---

### RC-1: `.account.relogin dry::1` exits 0 without spawning subprocess

- **Given:** A saved account `alice` in the credential store
- **When:** `.account.relogin alice dry::1`
- **Then:** Exits 0; no subprocess (`claude auth login` or `claude`) is spawned — dry-run mode short-circuits before the subprocess launch point
- **Note:** This is the primary automated-test path for relogin parameter validation. The actual browser OAuth subprocess (Mode 1/Mode 3 per research doc) cannot be tested in automated contexts — it requires TTY and browser interaction.
- **Source fn:** `ar05_relogin_dry_explicit_name` (cli/account_relogin_test.rs)
- **Source:** [docs/research_interactive/001_claude_interactive_session_control.md §Mode 1](../../../../docs/research_interactive/001_claude_interactive_session_control.md)

---

### RC-2: `.account.relogin` with unknown account name exits 2

- **Given:** No account named `unknown@example.com` in the credential store
- **When:** `.account.relogin unknown@example.com`
- **Then:** Exits 2 — account not found in credential store; no subprocess is spawned
- **Source fn:** `ar04_relogin_not_found_exits2` (cli/account_relogin_test.rs)
- **Source:** [docs/research_interactive/001_claude_interactive_session_control.md §Implication for .account.relogin](../../../../docs/research_interactive/001_claude_interactive_session_control.md)

---

### RC-3: `.account.relogin` resolves account name by prefix

- **Given:** Saved account `alice@example.com`; no other account with prefix `alice`
- **When:** `.account.relogin alice`
- **Then:** Resolves to `alice@example.com` — relogin participates in the standard prefix-match resolution used by all account commands
- **Source fn:** `ar08_relogin_prefix_resolves` (cli/account_relogin_test.rs)
- **Source:** [docs/research_interactive/001_claude_interactive_session_control.md §Implication for .account.relogin](../../../../docs/research_interactive/001_claude_interactive_session_control.md)

---

### RC-4: Mode 2 (`--print`) constraint: cannot trigger browser OAuth in non-TTY context

- **Given:** A conceptual test attempting to verify browser OAuth via Mode 2 (`--print`, non-TTY)
- **When:** A `claude --print .` subprocess is launched with a dead `refreshToken`
- **Then:** The subprocess exits with an error or empty output — it does NOT open a browser; Mode 2 is deliberately constrained to non-interactive operation. This is a constraint, not a defect: automated tests cannot verify browser OAuth flows; only dry-run and parameter validation are automatable.
- **Note:** Research finding: only Mode 3 (`claude auth login`) can trigger browser OAuth without the full REPL. Mode 2 is the current mechanism for `refresh::1` and `touch::1`. Live relogin tests require manual execution outside the automated test suite.
- **Source fn:** `ar05_relogin_dry_explicit_name` (cli/account_relogin_test.rs; demonstrates that automated coverage stops at pre-spawn validation)
- **Source:** [docs/research_interactive/001_claude_interactive_session_control.md §Mode 2 §Implication for .account.relogin](../../../../docs/research_interactive/001_claude_interactive_session_control.md)
