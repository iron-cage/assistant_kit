# Parameter Interactions

Formal specification of co-dependencies, mutual exclusions, and cascading effects between clp parameters.

### All Interactions (3 total)

| # | Interaction | Parameters | Effect |
|---|-------------|------------|--------|
| 1 | `format::json` overrides `verbosity::` | `format::`, `verbosity::` | JSON output includes all fields regardless of verbosity level |
| 2 | `dry::` is orthogonal to output control | `dry::`, `verbosity::`, `format::` | Dry-run mode applies to mutation; does not affect output formatting |
| 3 | `format::json` overrides field-presence params | `format::`, `active::`, `account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `file::`, `saved::` | JSON output includes all fields regardless of field-presence param values |

---

### Interaction :: 1. `format::json` overrides `verbosity::`

**Parameters:** [`format::`](params.md#parameter--3-format), [`verbosity::`](params.md#parameter--2-verbosity--v)

**Effect:** When `format::json` is specified, the output always includes all available fields regardless of the `v::` level. The verbosity setting is ignored for JSON output — JSON serialization always produces a complete object.

**Rationale:** JSON consumers rely on stable schemas; omitting fields based on verbosity would break pipeline consumers that expect consistent structure.

**Commands affected:** [`.token.status`](commands.md#command--7-tokenstatus), [`.paths`](commands.md#command--8-paths), [`.usage`](commands.md#command--9-usage), [`.account.limits`](commands.md#command--11-accountlimits)

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

**Commands affected:** [`.account.save`](commands.md#command--4-accountsave), [`.account.switch`](commands.md#command--5-accountswitch), [`.account.delete`](commands.md#command--6-accountdelete)

---

### Interaction :: 3. `format::json` overrides field-presence params

**Parameters:** [`format::`](params.md#parameter--3-format), [`active::`](params.md#parameter--14-active), [`account::`](params.md#parameter--6-account), [`sub::`](params.md#parameter--7-sub), [`tier::`](params.md#parameter--8-tier), [`token::`](params.md#parameter--9-token), [`expires::`](params.md#parameter--10-expires), [`email::`](params.md#parameter--11-email), [`file::`](params.md#parameter--12-file), [`saved::`](params.md#parameter--13-saved)

**Effect:** When `format::json` is specified on `.accounts` or `.credentials.status`, the JSON output always includes all fields regardless of field-presence param values. Setting `sub::0` or `active::0` only suppresses those fields in text output, not in JSON.

**Rationale:** JSON consumers rely on stable schemas; selectively omitting fields based on presence params would break pipeline consumers that expect consistent structure. The field-presence params are a text-output formatting concern, not a data-selection concern.

**Commands affected:** [`.accounts`](commands.md#command--3-accounts), [`.credentials.status`](commands.md#command--10-credentialsstatus)

**Examples:**

```bash
# sub::0 suppresses Sub: in text, but JSON still has "subscription_type"
clp .accounts sub::0 format::json
# [{"name":"alice@acme.com","is_active":true,"subscription_type":"max",...}]  ← subscription_type still present

# active::0 suppresses Active: in text, but JSON still has "is_active"
clp .accounts active::0 format::json
# [{"name":"alice@acme.com","is_active":true,...}]  ← is_active still present

# file::0 (default) suppresses File: in text, but JSON always includes "file"
clp .credentials.status format::json
# {"subscription":"max",...,"file":"/home/user/.claude/.credentials.json","saved":2}
```
