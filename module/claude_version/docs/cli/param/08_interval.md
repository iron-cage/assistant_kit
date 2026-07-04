# Parameter :: 8. `interval::`

-- **Summary:** Guard check frequency in seconds; 0 = one-shot mode.
-- **Type:** u64
-- **Default:** 0 (one-shot)
-- **Commands:** `.version.guard`
-- **Group:** none

Controls the check frequency for `.version.guard`. When `0` (default), the
guard runs once and exits. When `N > 0`, the guard loops every `N` seconds
until interrupted.

- **Type:** u64 (unsigned integer, seconds)
- **Default:** 0 (one-shot)
- **Validation:** must be a non-negative integer

```sh
clv.version.guard interval::0      # one-shot (default)
clv.version.guard interval::60     # check every 60 seconds
clv.version.guard interval::3600   # check every hour
```

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.version.guard`](../command/version.md#command--5-versionguard) | 0 (one-shot) | >0 enables watch mode; loops every N seconds until interrupted |

### Referenced Type

| # | Type |
|---|------|
| 1 | `u64` |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |
