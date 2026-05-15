# Parameter Groups

| Group | Parameters | Used By |
|-------|------------|---------|
| [Output Control](#group--1-output-control) | `format::` | `.accounts` (format only), `.token.status`, `.paths`, `.usage`, `.account.limits` |
| [Field Presence](#group--2-field-presence) | `active::`, `account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `file::`, `saved::`, `display_name::`, `role::`, `billing::`, `model::` | `.accounts`, `.credentials.status` |

**Total:** 2 groups

---

### Group :: 1. Output Control

**Parameters:** `format::`
**Pattern:** Read-only output formatting
**Purpose:** Controls presentation layer for commands that display information without modifying state.

| Parameter | Type | Description |
|-----------|------|-------------|
| [`format::`](params.md#parameter--2-format) | [`OutputFormat`](types.md#type--2-outputformat) | Output format: `text`, `json`, or `table` (`.accounts` only) |

**Used By:** [`.accounts`](commands.md#command--3-accounts), [`.token.status`](commands.md#command--7-tokenstatus), [`.paths`](commands.md#command--8-paths), [`.usage`](commands.md#command--9-usage), [`.credentials.status`](commands.md#command--10-credentialsstatus), [`.account.limits`](commands.md#command--11-accountlimits) — 6 commands

**Typical Patterns:**

```bash
# Scripting: structured JSON for pipeline consumption
clp .accounts format::json
clp .usage format::json

# Interactive: default text for human reading
clp .token.status
clp .usage
```

**Semantic Coherence Test**

> "Does parameter X control **output serialization format**?"

| Parameter | Controls output format? | In group? |
|-----------|-------------------------|-----------|
| `format::` | Yes — controls text vs JSON vs table serialization | Yes |
| `name::` | No — identifies target account, not presentation | No |
| `threshold::` | No — controls classification boundary, not presentation | No |
| `dry::` | No — controls execution mode, not presentation | No |
| `active::` | No — controls field inclusion, not presentation format | No (Field Presence) |

All members pass. No false inclusions.

**Why NOT These Parameters**

- **`name::`** — Identifies a target entity, not output style. Mutation commands (save, use, delete) don't produce formatted output in the Output Control sense.
- **`threshold::`** — Modifies classification logic (when to report ExpiringSoon), not how results are displayed. A classification parameter, not a presentation parameter.
- **`dry::`** — Controls whether mutation happens, not how output is formatted. Orthogonal concern (execution control vs output control).
- **Field Presence params** — Control which individual output lines appear (field selection), not how the output is serialized.

**Cross-References**

- [params.md](params.md) — individual parameter specifications
- [types.md](types.md) — `OutputFormat` type definition
- [commands.md](commands.md) — command specifications using this group
- [parameter_interactions.md](parameter_interactions.md) — `format::json` override rules

**Notes**

- `format::json` overrides field-presence params — see [parameter_interactions.md](parameter_interactions.md#interaction--2-formatjson-overrides-field-presence-params) for the authoritative rule.
- `format::table` ignores field-presence params and uses fixed columns — see [parameter_interactions.md](parameter_interactions.md#interaction--3-formattable-ignores-field-presence-params). Only accepted by `.accounts`.
- Commands not in this group (`.account.save`, `.account.use`, `.account.delete`) produce fixed single-line confirmation messages not affected by formatting parameters.

---

### Group :: 2. Field Presence

**Parameters:** `active::`, `account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `file::`, `saved::`, `display_name::`, `role::`, `billing::`, `model::`
**Pattern:** Per-field boolean presence control
**Purpose:** Each param independently controls whether one output line appears in text output. Shared params (`sub::`, `tier::`, `expires::`, `email::`, `display_name::`, `role::`, `billing::`, `model::`) work identically across both commands.

| Parameter | Type | Default | Commands | Controls |
|-----------|------|---------|----------|----------|
| [`active::`](params.md#parameter--13-active) | `bool` | `1` | `.accounts` only | Active/inactive status line |
| [`account::`](params.md#parameter--5-account) | `bool` | `1` | `.credentials.status` only | Active account name line |
| [`sub::`](params.md#parameter--6-sub) | `bool` | `1` | Both | Subscription type line |
| [`tier::`](params.md#parameter--7-tier) | `bool` | `1` | Both | Rate-limit tier line |
| [`token::`](params.md#parameter--8-token) | `bool` | `1` | `.credentials.status` only | Token status line |
| [`expires::`](params.md#parameter--9-expires) | `bool` | `1` | Both | Token expiry duration line |
| [`email::`](params.md#parameter--10-email) | `bool` | `1` | Both | Email address line |
| [`file::`](params.md#parameter--11-file) | `bool` | `0` | `.credentials.status` only | Credentials file path line (opt-in) |
| [`saved::`](params.md#parameter--12-saved) | `bool` | `0` | `.credentials.status` only | Saved account count line (opt-in) |
| [`display_name::`](params.md#parameter--14-display_name) | `bool` | `0` | Both | Display name line (opt-in) |
| [`role::`](params.md#parameter--15-role) | `bool` | `0` | Both | Organisation role line (opt-in) |
| [`billing::`](params.md#parameter--16-billing) | `bool` | `0` | Both | Billing type line (opt-in) |
| [`model::`](params.md#parameter--17-model) | `bool` | `0` | Both | Active model line (opt-in) |

**Used By (2 commands):** [`.accounts`](commands.md#command--3-accounts), [`.credentials.status`](commands.md#command--10-credentialsstatus)

**Typical Patterns:**

```bash
# Default: all on-by-default fields
clp .accounts
clp .credentials.status

# Compact: suppress less-essential fields
clp .accounts sub::0 tier::0 email::0
clp .credentials.status email::0

# Debug .credentials.status: add file path and account count
clp .credentials.status file::1 saved::1

# Bare names only (.accounts)
clp .accounts active::0 sub::0 tier::0 expires::0 email::0

# Token-only (.credentials.status)
clp .credentials.status account::0 sub::0 tier::0 expires::0 email::0
```

**Semantic Coherence Test**

> "Does parameter X independently control ONE output field?"

| Parameter | Controls one output field? | In group? |
|-----------|---------------------------|-----------|
| `active::` | Yes — Active: line (`.accounts`) | Yes |
| `account::` | Yes — Account: line (`.credentials.status`) | Yes |
| `sub::` | Yes — Sub: line | Yes |
| `tier::` | Yes — Tier: line | Yes |
| `token::` | Yes — Token: line (`.credentials.status`) | Yes |
| `expires::` | Yes — Expires: line | Yes |
| `email::` | Yes — Email: line (both commands) | Yes |
| `file::` | Yes — File: line (`.credentials.status`) | Yes |
| `saved::` | Yes — Saved: line (`.credentials.status`) | Yes |
| `display_name::` | Yes — Display: line | Yes |
| `role::` | Yes — Role: line | Yes |
| `billing::` | Yes — Billing: line | Yes |
| `model::` | Yes — Model: line | Yes |
| `format::` | No — controls serialisation format, not field selection | No (Output Control) |

All members pass. No false inclusions.

**Why NOT `format::`**

- **`format::`** — selects serialisation (text, JSON, or table), not field inclusion. `format::json` and `format::table` both render all fields regardless of field-presence params — the two axes are independent.

**Cross-References**

- [params.md](params.md) — individual field-presence parameter specifications
- [commands.md](commands.md#command--3-accounts) — `.accounts` command spec
- [commands.md](commands.md#command--10-credentialsstatus) — `.credentials.status` command spec
- [parameter_interactions.md](parameter_interactions.md) — `format::json` override rule for field-presence params
