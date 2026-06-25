# Test: `active::` Parameter — REMOVED (Feature 065)

> **REMOVED (Feature 065)**: The `active::` parameter on `.accounts` and `.usage` has been removed.
> Its functionality is replaced by the `assignee::` param: `assignee::USER@MACHINE name::X`.
> The `assignee::0` sentinel provides a shorthand for the current machine (`$USER@$HOSTNAME`).
>
> Any invocation of `active::` now exits 1 with the migration message:
> "REMOVED — use `assignee::USER@MACHINE name::X` (or `assignee::0 name::X` for current machine)"
>
> See [param/013_active.md](../../../../docs/cli/param/013_active.md) for the removal notice.
> See [feature/065_assignee_param_redesign.md](../../../../docs/feature/065_assignee_param_redesign.md) for the redesign.

All EC test cases in this file (EC-1 through EC-14) are **superseded** — `active::` no longer exists as an
active parameter. The behavioral semantics are now exercised by `64_assignee.md` EC-1 through EC-18 (which
cover `assignee::USER@MACHINE`, `assignee::0` sentinel, sanitization, dry-run, and the `active::`
REMOVED_TOGGLE migration path).

### Superseded Test Case Index (DO NOT IMPLEMENT)

| ID | Test Name | Category | Status |
|----|-----------|----------|--------|
| EC-1 | `active::user@host name::X` writes `_active_host_user = X` | Behavioral | **REMOVED** |
| EC-2 | `active::user@host` (no `name::`) clears `_active_host_user` | Behavioral | **REMOVED** |
| EC-3 | `active::badvalue` (no `@`) exits 1 | Validation | **REMOVED** |
| EC-4 | `active::@host` (empty user component) exits 1 | Validation | **REMOVED** |
| EC-5 | `active::user@` (empty machine component) exits 1 | Validation | **REMOVED** |
| EC-6 | Space in machine component sanitized to `_` | Sanitization | **REMOVED** |
| EC-7 | Dot and hyphen in machine component preserved | Sanitization | **REMOVED** |
| EC-8 | `active::user@host name::X dry::1` previews without writing | Dry-run | **REMOVED** |
| EC-9 | `active::user@host name::unknown` exits 1 (account not in store) | Validation | **REMOVED** |
| EC-10 | `active::` absent — no marker write (default omit) | Default | **REMOVED** |
| EC-11 | `active::user@host` does NOT modify `owner` field | Isolation | **REMOVED** |
| EC-12 | `active::0 name::X` exits 1 — `"0"` is not a valid `USER@MACHINE` | Validation | **REMOVED** |
| EC-13 | `force::1 active::user@host name::X` — `force::1` silently ignored; marker written | No-op | **REMOVED** |
| EC-14 | `active::user@host` (no `name::`) when marker absent — no-op exit 0 | Behavioral | **REMOVED** |
