# Parameter :: 15. `record_only::`

-- **Summary:** Persist the resolved preference without installing.
-- **Type:** bool
-- **Default:** false (0)
-- **Commands:** `.version.install`
-- **Group:** Execution Control

Writes `preferredVersionSpec`/`preferredVersionResolved` to `settings.json` — the
same write the success path performs — then returns before `perform_install()`
runs: no `curl`, no download, no binary swap. Lets a caller re-point
`.version.show`/`.version.guard` at a new target version without reinstalling
`claude`. Fires unconditionally: unlike the idempotency guard, it does not check
whether the target already matches the installed version. `force::` has no
install to bypass under `record_only::` and is silently ignored rather than
rejected. Mutually exclusive with `dry::` (exit 1 if both set) — `dry::` promises
"preview, no changes" while `record_only::` promises "write settings.json now",
a direct contradiction rather than an inert combination.

- **Type:** bool
- **Default:** false (0)
- **Validation:** strictly `0` or `1`; `true`, `yes`, `TRUE` etc. rejected with exit 1
- **Mutual exclusion:** `dry::1` + `record_only::1` → exit 1, `ArgumentMissing`

```sh
clv .version.install record_only::1                    # record "stable" as preferred, no install
clv .version.install version::2.1.99 record_only::1    # record v2.1.99 as preferred, no install
clv .version.install record_only::1 dry::1              # exit 1: mutually exclusive
```

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.version.install`](../command/version.md#command-4-versioninstall) | false | Persist preference only; `perform_install()` never runs |

### Referenced Type

| # | Type |
|---|------|
| 1 | `bool` |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|-----------|-----------|
| 1 | [Execution Control](../param_group/02_execution_control.md) | Partial | `dry::`, `force::` |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |
