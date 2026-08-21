# Parameter :: 40. `agents::`

### Scope

- **Purpose**: Specify the `agents::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `agents::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`), session family contract (→ `../../invariant/002_session_family.md`).

Agent fold-in toggle for [`.cost`](../command/15_cost.md): whether each conversation's row aggregates its family's agent sessions or the root session alone.

**Type:** Boolean

**Fundamental Type:** Integer (`0`/`1`)

**Constraints:**
- Accepts only `0` or `1`; any other value is an argument error (`agents must be 0 or 1`), validated before any storage access despite the default (Finding #010 convention — defaults do not exempt a parameter from explicit validation)

**Default:** `1` — agent sessions folded in. A conversation's true cost includes the agent work it spawned, so fold-in is the default and exclusion is the opt-out.

**Commands:** [`.cost`](../command/15_cost.md) — the only command registering this parameter.

**Purpose:** At `1`, the row aggregates the root plus every agent session in its family per the [Session Family invariant](../../invariant/002_session_family.md) (both storage layouts), and the `Agents` column counts the folded files. At `0`, the row is the root session alone and `Agents` shows `0`. Deliberately named `agents::`, NOT a reuse of [`agent::`](01_agent.md): that existing parameter is a session-type *filter* (main-vs-agent, with `0`/`1` selecting which kind to list) on `.list`/`.projects` — here `0`/`1` means exclude/include subordinate sessions in an aggregate, a different semantic that overloading one name would conflate. The plural form marks the aggregate-membership meaning.

**Examples:**
```bash
# Default: conversation totals include agent sessions
.cost

# Root session alone — how much was the main thread vs its agents?
.cost agents::0

# Compose with session_ids:: for a roots-only comparison
.cost session_ids::aaaa1111,bbbb2222 agents::0
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Integer | Only `0` or `1` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 15 | [`.cost`](../command/15_cost.md) | `1` (fold in) | `Agents` column counts folded agent session files |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
