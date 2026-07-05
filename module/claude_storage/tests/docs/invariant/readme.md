# Invariant Doc Entity

### Scope

- **Purpose**: Document test plans for the three behavioral invariants in `docs/invariant/`.
- **Responsibility**: Index of per-invariant test case planning files.
- **In Scope**: All three `claude_storage` behavioral invariants (path encoding, session family, entry type format).
- **Out of Scope**: Automated test implementations (→ `tests/` in crate), CLI command test specs (→ `tests/docs/cli/`).

Direct contract verification for the invariants defined in `docs/invariant/001-003`.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| `001_path_encoding.md` | IN- test cases for path encoding invariant (`docs/invariant/001_path_encoding.md`) | ✅ |
| `002_session_family.md` | IN- test cases for session family invariant (`docs/invariant/002_session_family.md`) | ✅ |
| `003_entry_type_format.md` | IN- test cases for entry type format invariant (`docs/invariant/003_entry_type_format.md`) | ✅ |
