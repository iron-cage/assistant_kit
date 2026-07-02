# tests/manual

Manual testing plan for `clv` (claude_version) — scenarios that require
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

- **Test:** run `clv .status` (or any command) and inspect the help footer
- **Expected by spec:** footer example should reference `clv`
- **Actual:** footer may show `". .list help"` (from published `unilang ~0.48` library)
- **Status:** Known issue in upstream `unilang` crate; out of scope for direct fix
- **Workaround:** none; the error-path example (`clv .version.show`) is correct

### M-03: Unknown-param error `??` suggestion (known P4 issue)

- **Test:** run `clv .status bogus::x`
- **Expected by spec:** error should suggest a valid way to see parameters
- **Actual:** error message may say `..status ??` (double-dot, unilang formatting bug)
- **Status:** Known issue in upstream `unilang ~0.48`; double-dot from `".{cmd_name}"` format
  when cmd_name already starts with `.`

### M-04: Token expiry warning threshold

- **Note:** out of scope for `claude_version` — belongs to the account management crate.
- **Test:** set up credentials with expiry 30 minutes from now, run the account status command
- **Expected:** token shows `"expiring in 30m"` (not `"valid"`)
- **Why manual:** requires controlling the system clock or expiry time precisely
