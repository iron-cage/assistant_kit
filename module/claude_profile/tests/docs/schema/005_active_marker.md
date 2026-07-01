# Schema 005: Active Marker — `_active_{host}_{user}`

SC test cases for `docs/schema/005_active_marker.md`. Verifies the per-machine
active-account marker: filename derivation from env vars, non-alphanumeric sanitization,
plain-text content format, and `other_machines_active()` read semantics.

**Source:** [docs/schema/005_active_marker.md](../../../../docs/schema/005_active_marker.md)

### SC Case Index

| ID | Short Name | Category | Status |
|----|------------|----------|--------|
| SC-1 | Save writes active marker with correct `_active_{host}_{user}` filename | Write | ✅ |
| SC-2 | Marker filename derived from `$HOSTNAME` + `$USER` env vars | Filename Derivation | ✅ |
| SC-3 | Non-alphanumeric chars in hostname/user are replaced with `_` | Sanitization | ✅ |
| SC-4 | Marker content is plain text account name — no JSON | Content Format | ✅ |
| SC-5 | `other_machines_active()` excludes current machine, returns all others | Cross-Machine Read | ✅ |

---

### SC-1: `account.save` writes active marker with correct filename shape

- **Given:** `$HOSTNAME = "w003"` and `$USER = "user1"` in the environment
- **When:** `.account.save alice` is invoked
- **Then:** A file named `_active_w003_user1` is created in the credential store — the active marker filename matches `_active_{HOSTNAME}_{USER}` pattern
- **Source fn:** `as16_save_writes_active_marker` (cli/account_mutations_test_b.rs)
- **Source:** [docs/schema/005_active_marker.md §File Location §Filename Derivation](../../../../docs/schema/005_active_marker.md)

---

### SC-2: Marker filename is derived from `$HOSTNAME` and `$USER` env vars

- **Given:** Custom `$HOSTNAME` and `$USER` values set in the test environment
- **When:** `active_marker_filename()` is called
- **Then:** Returns `_active_{HOSTNAME}_{USER}` — env var values are used directly (with sanitization); fallback chain (`$HOSTNAME` → `/etc/hostname` → `"local"`) activates only when env vars are absent
- **Source fn:** `sc2_005_active_marker_filename_uses_env_vars` (account_tests.rs)
- **Source:** [docs/schema/005_active_marker.md §Filename Derivation](../../../../docs/schema/005_active_marker.md)

---

### SC-3: Non-alphanumeric characters in hostname/user are replaced with `_`

- **Given:** `$HOSTNAME = "w003.local"` (contains `.`) or `$USER = "user@corp"` (contains `@`)
- **When:** `active_marker_filename()` is called
- **Then:** Dots, `@`, and all other non-`[a-zA-Z0-9\-.]` characters are replaced with `_` — safe for use as a filename component
- **Source fn:** `sc3_005_active_marker_sanitizes_nonalphanumeric_to_underscore` (account_tests.rs)
- **Source:** [docs/schema/005_active_marker.md §Filename Derivation](../../../../docs/schema/005_active_marker.md)

---

### SC-4: Active marker content is a single plain-text account name — no JSON

- **Given:** An active marker file `_active_w003_user1` exists in the credential store
- **When:** The file is read as raw text
- **Then:** Content is a single email address string (e.g., `alice@example.com`), trimmed of whitespace, no JSON structure, no metadata
- **Source fn:** `ft13_025_sessions_table_parses_marker_identity_from_filename` (usage/render_tests_b.rs; reads marker filename to identify current session)
- **Source:** [docs/schema/005_active_marker.md §Content Format](../../../../docs/schema/005_active_marker.md)

---

### SC-5: `other_machines_active()` excludes current machine's marker, returns all others

- **Given:** Multiple `_active_*` files exist in the credential store (one for the current machine, two for other machines)
- **When:** `other_machines_active()` is called
- **Then:** Returns a `HashSet<String>` containing the account names from the OTHER machines' markers — the current machine's own marker is excluded; missing/unreadable files are silently skipped
- **Source fn:** `ft30_009_sessions_table_shown_auto_multiple_markers` (usage/render_tests_b.rs)
- **Source:** [docs/schema/005_active_marker.md §other_machines_active() API](../../../../docs/schema/005_active_marker.md)
