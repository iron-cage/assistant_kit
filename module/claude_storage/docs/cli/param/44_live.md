# Parameter :: 44. `live::`

### Scope

- **Purpose**: Specify the `live::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `live::`.
- **In Scope**: Value constraints, default behavior, command interactions, the unavailable-detection response.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`), how liveness is inferred (→ `../../algorithm/002_session_liveness.md`).

Attached-process filter for [`.projects`](../command/07_projects.md): narrows the listing to projects that do — or do not — have a Claude Code process running against them right now.

**Type:** Boolean

**Fundamental Type:** Integer (`0`/`1`)

**Constraints:**
- Accepts only `0` or `1`; any other value is an argument error, rejected before any storage access (Finding #010 convention)
- Unset is a third state, distinct from `0`: no filtering at all

**Default:** unset — every scoped project is listed, live or not.

**Commands:** [`.projects`](../command/07_projects.md) — the only command registering this parameter.

**Purpose:** A global store reaches hundreds of projects (914 measured, of which 38 were live) while the answer to "what am I actually running" is a couple of dozen rows. `live::1` is the narrowing that makes `scope::global` usable for that question.

**Scope of the filter is the project, not the conversation.** Filtering sessions here would desynchronize every per-project count from what actually renders — the issue-034 class of defect the command already guards against. Which *conversation* is being driven is answered by marking it in `detail::sessions`, not by hiding its siblings.

**When detection is unavailable.** Liveness is read from the process table and can only report positives (→ [`../../algorithm/002_session_liveness.md`](../../algorithm/002_session_liveness.md)). Inside a container, or on a platform without `/proc`, nothing is visible even while sessions are running. `live::1` therefore never returns a silently empty list: finding no attached process at all produces an explicit note saying so and naming the reason, because "nothing is running" and "this host cannot see it" are not distinguishable from the inside. `live::0` under the same conditions correctly returns everything, and — since no row is live — the `STATUS` column is simply absent.

**With `ids::1`.** The same project-level verdict applies, as a predicate on the one project `project::` names: `live::1` emits its conversation IDs only if something is attached, `live::0` only if nothing is. Suppression yields no lines, or `0` under `count::1`. The unavailable-detection case cannot be answered in prose here — a scripting mode's stdout has to stay parseable — so `live::1` with no attached process visible anywhere **exits non-zero with the explanation on stderr** rather than emitting an empty list a caller would read as fact. Probing is skipped entirely when `live::` is unset, keeping the plain `ids::1` path free of a process-table walk.

**Examples:**
```bash
# What is running right now, anywhere in the store
.projects scope::global live::1

# …and which conversation each one is driving
.projects scope::global live::1 detail::sessions

# The inverse: everything with nothing attached
.projects live::0

# Compose with any other filter
.projects scope::global live::1 filter::assistant
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Integer | Only `0` or `1`; unset means no filtering |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 7 | [`.projects`](../command/07_projects.md) | unset (no filter) | Project-scoped; pairs with the `STATUS` column and `detail::sessions` tags |

### Referenced Algorithm
| # | Algorithm | Relationship |
|---|-----------|--------------|
| 002 | [Session Liveness](../../algorithm/002_session_liveness.md) | Supplies the per-project verdict this parameter filters on |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
