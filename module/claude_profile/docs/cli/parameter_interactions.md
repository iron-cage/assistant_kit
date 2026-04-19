# Parameter Interactions

Formal specification of co-dependencies, mutual exclusions, and cascading effects between clp parameters.

### All Interactions (2 total)

| # | Interaction | Parameters | Effect |
|---|-------------|------------|--------|
| 1 | `format::json` overrides `verbosity::` | `format::`, `verbosity::` | JSON output includes all fields regardless of verbosity level |
| 2 | `dry::` is orthogonal to output control | `dry::`, `verbosity::`, `format::` | Dry-run mode applies to mutation; does not affect output formatting |

---

### Interaction :: 1. `format::json` overrides `verbosity::`

**Parameters:** [`format::`](params.md#parameter--3-format), [`verbosity::`](params.md#parameter--2-verbosity--v)

**Effect:** When `format::json` is specified, the output always includes all available fields regardless of the `v::` level. The verbosity setting is ignored for JSON output — JSON serialization always produces a complete object.

**Rationale:** JSON consumers rely on stable schemas; omitting fields based on verbosity would break pipeline consumers that expect consistent structure.

**Commands affected:** All 7 Output Control commands — [`.account.list`](commands.md#command--3-accountlist), [`.account.status`](commands.md#command--4-accountstatus), [`.token.status`](commands.md#command--8-tokenstatus), [`.paths`](commands.md#command--9-paths), [`.usage`](commands.md#command--10-usage), [`.credentials.status`](commands.md#command--11-credentialsstatus), [`.account.limits`](commands.md#command--12-accountlimits)

**Examples:**

```bash
# v::0 normally suppresses labels, but format::json always returns full object
clp .token.status v::0 format::json
# {"status":"valid","expires_in_secs":2820}  ← all fields present

# v::0 without json gives bare value only
clp .token.status v::0
# valid
```

---

### Interaction :: 2. `dry::` is orthogonal to output control

**Parameters:** [`dry::`](params.md#parameter--5-dry), [`verbosity::`](params.md#parameter--2-verbosity--v), [`format::`](params.md#parameter--3-format)

**Effect:** `dry::` controls whether the mutation executes; it does not affect output formatting. The `[dry-run]` prefix is always added to the confirmation message regardless of `v::` or `format::`. The `dry::` parameter is only available on mutation commands (`.account.save`, `.account.switch`, `.account.delete`), which do not belong to the Output Control group.

**Rationale:** Mutation commands produce single fixed-line confirmation messages, not parameterized output; the Output Control parameters have no effect on them. The two concerns — execution mode and output formatting — are fully independent.

**Commands affected:** [`.account.save`](commands.md#command--5-accountsave), [`.account.switch`](commands.md#command--6-accountswitch), [`.account.delete`](commands.md#command--7-accountdelete)
