# Parameter Groups

| Group | Parameters | Used By |
|-------|------------|---------|
| [Output Control](#group--1-output-control) | `verbosity::`, `format::` | `.accounts` (format only), `.token.status`, `.paths`, `.usage`, `.account.limits` |
| [Field Presence](#group--2-field-presence) | `active::`, `account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `org::`, `file::`, `saved::`, `display_name::`, `role::`, `billing::`, `model::` | `.accounts`, `.credentials.status` |

**Total:** 2 groups

---

### Group :: 1. Output Control

**Parameters:** `verbosity::`, `format::`
**Pattern:** Read-only output formatting
**Purpose:** Controls presentation layer for commands that display information without modifying state.

| Parameter | Type | Description |
|-----------|------|-------------|
| [`verbosity::`](params.md#parameter--2-verbosity--v) | [`VerbosityLevel`](types.md#type--2-verbositylevel) | Output detail: 0=quiet, 1=normal, 2=verbose |
| [`format::`](params.md#parameter--3-format) | [`OutputFormat`](types.md#type--3-outputformat) | Output format: `text` or `json` |

**Used By:** [`.accounts`](commands.md#command--3-accounts) *(format only)*, [`.token.status`](commands.md#command--7-tokenstatus), [`.paths`](commands.md#command--8-paths), [`.usage`](commands.md#command--9-usage), [`.account.limits`](commands.md#command--11-accountlimits) — 5 commands total (4 full, 1 partial)

**Typical Patterns:**

```bash
# Scripting: bare JSON for pipeline consumption
clp .accounts format::json

# Interactive: default labels for human reading
clp .token.status

# Debugging: full metadata for diagnostics
clp .paths v::2
```

**Semantic Coherence Test**

> "Does parameter X control **output presentation**?"

| Parameter | Controls output presentation? | In group? |
|-----------|-------------------------------|-----------|
| `verbosity::` | Yes — controls label density and metadata detail | Yes |
| `format::` | Yes — controls text vs JSON serialization | Yes |
| `name::` | No — identifies target account, not presentation | No |
| `threshold::` | No — controls classification boundary, not presentation | No |
| `dry::` | No — controls execution mode, not presentation | No |
| `active::` | No — controls field inclusion, not presentation format | No (Field Presence) |

All members pass. No false inclusions.

**Why NOT These Parameters**

- **`name::`** — Identifies a target entity, not output style. Mutation commands (save, switch, delete) don't produce formatted output in the Output Control sense.
- **`threshold::`** — Modifies classification logic (when to report ExpiringSoon), not how results are displayed. A classification parameter, not a presentation parameter.
- **`dry::`** — Controls whether mutation happens, not how output is formatted. Orthogonal concern (execution control vs output control).
- **Field Presence params** — Control which individual output lines appear (field selection), not how the output is serialized or how dense it is.

**Partial Implementors**

- **`.accounts`** — Full implementor of `format::` only. Does not support `verbosity::` — uses individual Field Presence params instead of density levels.
- **`.credentials.status`** — Full implementor of `format::` only. Does not support `verbosity::` — same Field Presence pattern.

**Cross-References**

- [params.md](params.md) — individual parameter specifications
- [types.md](types.md) — `VerbosityLevel`, `OutputFormat` type definitions
- [commands.md](commands.md) — command specifications using this group
- [parameter_interactions.md](parameter_interactions.md) — `format::json` / `verbosity::` interaction rule

**Notes**

- `format::json` overrides `verbosity::` — see [parameter_interactions.md](parameter_interactions.md#interaction--1-formatjson-overrides-verbosity) for the authoritative rule.
- Commands not in this group (`.account.save`, `.account.switch`, `.account.delete`) produce fixed single-line confirmation messages not affected by formatting parameters.

---

### Group :: 2. Field Presence

**Parameters:** `active::`, `account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `org::`, `file::`, `saved::`, `display_name::`, `role::`, `billing::`, `model::`
**Pattern:** Per-field boolean presence control
**Purpose:** Each param independently controls whether one output line appears in text output. Shared params (`sub::`, `tier::`, `expires::`, `org::`, `display_name::`, `role::`, `billing::`, `model::`) work identically across both commands.

| Parameter | Type | Default | Commands | Controls |
|-----------|------|---------|----------|----------|
| [`active::`](params.md#parameter--15-active) | `bool` | `1` | `.accounts` only | Active/inactive status line |
| [`account::`](params.md#parameter--6-account) | `bool` | `1` | `.credentials.status` only | Active account name line |
| [`sub::`](params.md#parameter--7-sub) | `bool` | `1` | Both | Subscription type line |
| [`tier::`](params.md#parameter--8-tier) | `bool` | `1` | Both | Rate-limit tier line |
| [`token::`](params.md#parameter--9-token) | `bool` | `1` | `.credentials.status` only | Token status line |
| [`expires::`](params.md#parameter--10-expires) | `bool` | `1` | Both | Token expiry duration line |
| [`email::`](params.md#parameter--11-email) | `bool` | `1` | `.credentials.status` only | Email address line |
| [`org::`](params.md#parameter--12-org) | `bool` | `1` | Both | Organisation name line |
| [`file::`](params.md#parameter--13-file) | `bool` | `0` | `.credentials.status` only | Credentials file path line (opt-in) |
| [`saved::`](params.md#parameter--14-saved) | `bool` | `0` | `.credentials.status` only | Saved account count line (opt-in) |
| [`display_name::`](params.md#parameter--16-display_name) | `bool` | `0` | Both | Display name line (opt-in) |
| [`role::`](params.md#parameter--17-role) | `bool` | `0` | Both | Organisation role line (opt-in) |
| [`billing::`](params.md#parameter--18-billing) | `bool` | `0` | Both | Billing type line (opt-in) |
| [`model::`](params.md#parameter--19-model) | `bool` | `0` | Both | Active model line (opt-in) |

**Used By (2 commands):** [`.accounts`](commands.md#command--3-accounts), [`.credentials.status`](commands.md#command--10-credentialsstatus)

**Typical Patterns:**

```bash
# Default: all on-by-default fields
clp .accounts
clp .credentials.status

# Compact: suppress less-essential fields
clp .accounts sub::0 tier::0 org::0
clp .credentials.status email::0 org::0

# Debug .credentials.status: add file path and account count
clp .credentials.status file::1 saved::1

# Bare names only (.accounts)
clp .accounts active::0 sub::0 tier::0 expires::0 org::0

# Token-only (.credentials.status)
clp .credentials.status account::0 sub::0 tier::0 expires::0 email::0 org::0
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
| `email::` | Yes — Email: line (`.credentials.status`) | Yes |
| `org::` | Yes — Org: line | Yes |
| `file::` | Yes — File: line (`.credentials.status`) | Yes |
| `saved::` | Yes — Saved: line (`.credentials.status`) | Yes |
| `display_name::` | Yes — Display: line | Yes |
| `role::` | Yes — Role: line | Yes |
| `billing::` | Yes — Billing: line | Yes |
| `model::` | Yes — Model: line | Yes |
| `format::` | No — controls serialisation format, not field selection | No (Output Control) |
| `verbosity::` | No — controls detail density across all fields | No (Output Control) |

All members pass. No false inclusions.

**Why NOT `format::` and `verbosity::`**

- **`format::`** — selects serialisation (text vs JSON), not field inclusion. `format::json` always serialises all fields regardless of field-presence params — the two axes are independent.
- **`verbosity::`** — controls per-field detail density (labels, extra context). Neither `.accounts` nor `.credentials.status` uses `verbosity::` — each field either appears or not.

**Cross-References**

- [params.md](params.md) — individual field-presence parameter specifications
- [commands.md](commands.md#command--3-accounts) — `.accounts` command spec
- [commands.md](commands.md#command--10-credentialsstatus) — `.credentials.status` command spec
- [parameter_interactions.md](parameter_interactions.md) — `format::json` override rule for field-presence params
