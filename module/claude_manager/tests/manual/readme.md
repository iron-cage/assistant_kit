# tests/manual

Manual testing plan for `cm` (claude_manager) — scenarios that require
human verification or cannot be reliably automated in CI.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `readme.md` | Manual testing plan and known issues |

## Manual Test Scenarios

### M-01: Network-unavailable path for `.version.history`

- **Trigger:** disable network (e.g., `unshare -n`) and delete cache
- **Expected:** exit 2, error message mentions network failure
- **Why manual:** Cannot reliably simulate network failure in CI without root-level sandbox

### M-02: Unilang help footer example (known P1 issue)

- **Test:** run `cm .account.list` (or any command) and inspect the help footer
- **Expected by spec:** footer example should reference `cm`
- **Actual:** footer may show `". .list help"` (from published `unilang ~0.48` library)
- **Status:** Known issue in upstream `unilang` crate; out of scope for direct fix
- **Workaround:** none; the error-path example (`cm .version.show`) is correct

### M-03: Unknown-param error `??` suggestion (known P4 issue)

- **Test:** run `cm .account.list bogus::x`
- **Expected by spec:** error should suggest a valid way to see parameters
- **Actual:** error message may say `..account.list ??` (double-dot, unilang formatting bug)
- **Status:** Known issue in upstream `unilang ~0.48`; double-dot from `".{cmd_name}"` format
  when cmd_name already starts with `.`

### M-04: Token expiry warning threshold

- **Test:** set up credentials with expiry 30 minutes from now, run `.account.status v::2`
- **Expected:** token shows `"expiring in 30m"` (not `"valid"`)
- **Why manual:** requires controlling the system clock or expiry time precisely
