# Parameter Interactions

Formal specification of co-dependencies, mutual exclusions, and cascading effects between clp parameters.

### All Interactions (3 total)

| # | Interaction | Parameters | Effect |
|---|-------------|------------|--------|
| 1 | `dry::` is orthogonal to output control | `dry::`, `format::` | Dry-run mode applies to mutation; does not affect output formatting |
| 2 | `format::json` overrides field-presence params | `format::`, `active::`, `account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `file::`, `saved::`, `display_name::`, `role::`, `billing::`, `model::` | JSON output includes all fields regardless of field-presence param values |
| 3 | `format::table` ignores field-presence params | `format::`, `active::`, `sub::`, `tier::`, `expires::`, `email::`, `display_name::`, `role::`, `billing::`, `model::` | Table output uses fixed columns regardless of field-presence param values |

---

### Interaction :: 1. `dry::` is orthogonal to output control

**Parameters:** [`dry::`](params.md#parameter--4-dry), [`format::`](params.md#parameter--2-format)

**Effect:** `dry::` controls whether the mutation executes; it does not affect output formatting. The `[dry-run]` prefix is always added to the confirmation message regardless of `format::`. The `dry::` parameter is only available on mutation commands (`.account.save`, `.account.use`, `.account.delete`), which do not belong to the Output Control group.

**Rationale:** Mutation commands produce single fixed-line confirmation messages, not parameterized output; the Output Control parameter has no effect on them. The two concerns — execution mode and output formatting — are fully independent.

**Commands affected:** [`.account.save`](commands.md#command--4-accountsave), [`.account.use`](commands.md#command--5-accountuse), [`.account.delete`](commands.md#command--6-accountdelete)

---

### Interaction :: 2. `format::json` overrides field-presence params

**Parameters:** [`format::`](params.md#parameter--2-format), [`active::`](params.md#parameter--13-active), [`account::`](params.md#parameter--5-account), [`sub::`](params.md#parameter--6-sub), [`tier::`](params.md#parameter--7-tier), [`token::`](params.md#parameter--8-token), [`expires::`](params.md#parameter--9-expires), [`email::`](params.md#parameter--10-email), [`file::`](params.md#parameter--11-file), [`saved::`](params.md#parameter--12-saved), [`display_name::`](params.md#parameter--14-display_name), [`role::`](params.md#parameter--15-role), [`billing::`](params.md#parameter--16-billing), [`model::`](params.md#parameter--17-model)

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

---

### Interaction :: 3. `format::table` ignores field-presence params

**Parameters:** [`format::`](params.md#parameter--2-format), [`active::`](params.md#parameter--13-active), [`sub::`](params.md#parameter--6-sub), [`tier::`](params.md#parameter--7-tier), [`expires::`](params.md#parameter--9-expires), [`email::`](params.md#parameter--10-email), [`display_name::`](params.md#parameter--14-display_name), [`role::`](params.md#parameter--15-role), [`billing::`](params.md#parameter--16-billing), [`model::`](params.md#parameter--17-model)

**Effect:** When `format::table` is specified on `.accounts`, the table always uses a fixed column set (flag, Account, Sub, Tier, Expires, Email) regardless of field-presence param values. Passing `sub::0` or `tier::0` alongside `format::table` has no effect on table columns.

**Rationale:** Table layout requires fixed column widths computed across all rows; selectively omitting columns based on field-presence params would break alignment and produce inconsistent table structures. Field-presence params are a text-output concern; table is a distinct, fixed-schema rendering mode.

**Commands affected:** [`.accounts`](commands.md#command--3-accounts)

**Examples:**

```bash
# sub::0 has no effect in table mode — Sub column still appears
clp .accounts sub::0 format::table

# All field-presence params ignored in table mode
clp .accounts active::0 sub::0 tier::0 format::table
```
