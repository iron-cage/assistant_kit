# Parameter Groups

| Group | Parameters | Used By |
|-------|------------|---------|
| [Output Control](#group--1-output-control) | `verbosity::`, `format::` | `.account.list`, `.account.status`, `.token.status`, `.paths`, `.usage`, `.account.limits` |
| [Field Presence](#group--2-field-presence) | `account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `org::`, `file::`, `saved::` | `.credentials.status` |

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

**Used By (6 commands):** [`.account.list`](commands.md#command--3-accountlist), [`.account.status`](commands.md#command--4-accountstatus), [`.token.status`](commands.md#command--8-tokenstatus), [`.paths`](commands.md#command--9-paths), [`.usage`](commands.md#command--10-usage), [`.account.limits`](commands.md#command--12-accountlimits)

**Typical Patterns:**

```bash
# Scripting: bare JSON for pipeline consumption
clp .account.list format::json v::0

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

All members pass. No false inclusions.

**Why NOT These Parameters**

- **`name::`** — Identifies a target entity, not output style. Mutation commands (save, switch, delete) don't produce formatted output in the Output Control sense.
- **`threshold::`** — Modifies classification logic (when to report ExpiringSoon), not how results are displayed. A classification parameter, not a presentation parameter.
- **`dry::`** — Controls whether mutation happens, not how output is formatted. Orthogonal concern (execution control vs output control).

**Cross-References**

- [params.md](params.md) — individual parameter specifications
- [types.md](types.md) — `VerbosityLevel`, `OutputFormat` type definitions
- [commands.md](commands.md) — command specifications using this group
- [parameter_interactions.md](parameter_interactions.md) — `format::json` / `verbosity::` interaction rule

**Notes**

- `format::json` overrides `verbosity::` — see [parameter_interactions.md](parameter_interactions.md#interaction--1-formatjson-overrides-verbosity) for the authoritative rule.
- Commands not in this group (`.account.save`, `.account.switch`, `.account.delete`) produce fixed single-line confirmation messages not affected by formatting parameters.
- `.credentials.status` uses `format::` but not `verbosity::` — it belongs to the Output Control group only via `format::`, and uses Field Presence params instead of `verbosity::`.

---

### Group :: 2. Field Presence

**Parameters:** `account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`, `org::`, `file::`, `saved::`
**Pattern:** Per-field boolean presence control
**Purpose:** Each param independently controls whether one output line appears in `.credentials.status` text output. All default to `1` (shown) except `file::` and `saved::` which are opt-in (`0`).

| Parameter | Type | Default | Controls |
|-----------|------|---------|----------|
| [`account::`](params.md#parameter--6-account) | `bool` | `1` | Active account name line |
| [`sub::`](params.md#parameter--7-sub) | `bool` | `1` | Subscription type line |
| [`tier::`](params.md#parameter--8-tier) | `bool` | `1` | Rate-limit tier line |
| [`token::`](params.md#parameter--9-token) | `bool` | `1` | Token status line |
| [`expires::`](params.md#parameter--10-expires) | `bool` | `1` | Token expiry duration line |
| [`email::`](params.md#parameter--11-email) | `bool` | `1` | Email address line |
| [`org::`](params.md#parameter--12-org) | `bool` | `1` | Organization name line |
| [`file::`](params.md#parameter--13-file) | `bool` | `0` | Credentials file path line (opt-in) |
| [`saved::`](params.md#parameter--14-saved) | `bool` | `0` | Saved account count line (opt-in) |

**Used By (1 command):** [`.credentials.status`](commands.md#command--11-credentialsstatus)

**Typical Patterns:**

```bash
# Default: all 7 on-by-default fields
clp .credentials.status

# Compact: suppress email/org (often N/A on individual accounts)
clp .credentials.status email::0 org::0

# Debug: add credentials file path and account count
clp .credentials.status file::1 saved::1

# Token-only: suppress everything except token state
clp .credentials.status account::0 sub::0 tier::0 expires::0 email::0 org::0
```

**Semantic Coherence Test**

> "Does parameter X independently control ONE output field in `.credentials.status`?"

| Parameter | Controls one output field? | In group? |
|-----------|---------------------------|-----------|
| `account::` | Yes — Account: line | Yes |
| `sub::` | Yes — Sub: line | Yes |
| `tier::` | Yes — Tier: line | Yes |
| `token::` | Yes — Token: line | Yes |
| `expires::` | Yes — Expires: line | Yes |
| `email::` | Yes — Email: line | Yes |
| `org::` | Yes — Org: line | Yes |
| `file::` | Yes — File: line | Yes |
| `saved::` | Yes — Saved: line | Yes |
| `format::` | No — controls serialisation format, not field selection | No (Output Control) |
| `verbosity::` | No — controls detail density across all fields | No (Output Control; not on `.credentials.status`) |

All members pass. No false inclusions.

**Why NOT `format::` and `verbosity::`**

- **`format::`** — selects serialisation (text vs JSON), not field inclusion. `format::json` always serialises all fields regardless of field-presence params — the two axes are independent.
- **`verbosity::`** — controls per-field detail density (labels, extra context). `.credentials.status` does not use `verbosity::` because there is no "detail density" distinction — each field either appears or not.

**Cross-References**

- [params.md](params.md#parameter--6-account) — individual field-presence parameter specifications
- [commands.md](commands.md#command--11-credentialsstatus) — `.credentials.status` command spec
- [parameter_interactions.md](parameter_interactions.md) — `format::json` override rule for `.credentials.status`
