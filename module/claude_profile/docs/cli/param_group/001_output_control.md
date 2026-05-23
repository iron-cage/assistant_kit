# Group :: 1. Output Control

**Parameters:** `format::`
**Pattern:** Read-only output formatting
**Purpose:** Controls presentation layer for commands that display information without modifying state.

| Parameter | Type | Description |
|-----------|------|-------------|
| [`format::`](../param/002_format.md) | [`OutputFormat`](../type/002_output_format.md) | Output format: `text`, `json`, or `table` (`.accounts` only) |

**Used By:** [`.accounts`](../command/001_account.md#command--3-accounts), [`.token.status`](../command/005_token.md#command--7-tokenstatus), [`.paths`](../command/004_paths.md#command--8-paths), [`.usage`](../command/006_usage.md#command--9-usage), [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus), [`.account.limits`](../command/001_account.md#command--11-accountlimits) — 6 commands

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

- **`name::`** — Identifies a target entity, not output style.
- **`threshold::`** — Modifies classification logic, not how results are displayed.
- **`dry::`** — Controls whether mutation happens, not how output is formatted.
- **Field Presence params** — Control which individual output lines appear, not how output is serialized.

**Cross-References**

- [../004_parameter_interactions.md](../004_parameter_interactions.md) — `format::json` override rules

**Notes**

- `format::json` overrides field-presence params — see [004_parameter_interactions.md](../004_parameter_interactions.md#interaction--2-formatjson-overrides-field-presence-params) for the authoritative rule.
- `format::table` ignores field-presence params and uses fixed columns — see [004_parameter_interactions.md](../004_parameter_interactions.md#interaction--3-formattable-ignores-field-presence-params). Only accepted by `.accounts`.
- Commands not in this group (`.account.save`, `.account.use`, `.account.delete`, `.account.relogin`) produce fixed single-line confirmation messages not affected by formatting parameters.
