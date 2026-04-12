# Parameter Groups

Shared parameters reused across command roots. Groups emerge from semantic coherence — parameters that together control the same operational concern.

See [params.md](params.md) for individual parameter specs and [commands.md](commands.md) for per-command usage.

## Overview

| # | Group | Parameters | Used By |
|---|-------|-----------|---------|
| 1 | [Output Control](#output-control) | `verbosity::` | 6 commands |
| 2 | [Project Scope](#project-scope) | `project::` | 5 commands |
| 3 | [Session Identification](#session-identification) | `session_id::` | 2 commands |
| 4 | [Session Filter](#session-filter) | `session::`, `agent::`, `min_entries::` | 2 commands |
| 5 | [Scope Configuration](#scope-configuration) | `scope::`, `path::` | 6 commands |

---

## Output Control

**Parameters:** `verbosity::`

**Pattern:** Output verbosity and detail level control

**Purpose:** Controls how much information each read command outputs, from machine-readable minimal to full-field verbose.

**Used By:** `.status`, `.list`, `.show`, `.show.project`, `.search`, `.projects` (6 commands total)

**Semantic Coherence Test:**
- "Does `verbosity::` control output detail level?" → YES

**Why NOT `entries::` and `metadata::`:**
- `entries::` controls *what content* is shown (all entries vs summary), not *how much detail* per line
- `metadata::` toggles content suppression — a mode switch, not a verbosity adjustment
- Different semantic purpose: display mode vs output density

**Why NOT `sessions::` (bool):**
- `sessions::` controls project list view expansion (whether sessions appear), not detail granularity
- Its domain is scope of listing, not density of output

**Parameter Details:**

| Parameter | Type | Description | Alias |
|-----------|------|-------------|-------|
| `verbosity::` | [`VerbosityLevel`](types.md#verbositylevel) | Output detail: 0=silent, 1=normal, 2=detailed, 3=verbose | `v` |

**Examples:**
```bash
.status v::2
.list verbosity::3
.search query::error v::0
```

---

## Project Scope

**Parameters:** `project::`

**Pattern:** Project-level scope restriction

**Purpose:** Restricts an operation to a specific project, identified by multiple accepted formats.

**Used By:** `.show`, `.show.project`, `.count`, `.search`, `.export` (5 commands total)

**Semantic Coherence Test:**
- "Does `project::` control which project is operated on?" → YES

**Why NOT `path::` (in `.list`):**
- `path::` in `.list` is a substring filter on project *listing* — it affects which projects are shown, not which single project is the scope
- Different semantic purpose: filter expression vs scope pin

**Why NOT `session_id::`:**
- `session_id::` identifies a session within a project, not the project itself
- Different semantic level: sub-project identifier vs project identifier

**Parameter Details:**

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `project::` | [`ProjectId`](types.md#projectid) | Project identifier (path, encoded ID, UUID, or Path(...) form) | current dir |

**Accepted formats:**
```bash
project::/home/user1/pro/lib/consumer         # Absolute path
project::-home-user1-pro-lib-consumer         # Path-encoded ID
project::8d795a1c-c81d-4010-8d29-b4e678272419  # UUID
project::Path("/home/user1/pro/lib/consumer") # Path(...) from .list output
```

---

## Session Identification

**Parameters:** `session_id::`

**Pattern:** Direct session access by exact identifier

**Purpose:** Identifies a specific session by its filename stem for single-session operations (display or export). When used without an accompanying `project::` parameter, `session_id::` triggers a global search across all projects — the first project containing a matching session is used.

**Used By:** `.show`, `.export` (2 commands total)

**Semantic Coherence Test:**
- "Does `session_id::` identify a specific session for direct access?" → YES

**Why NOT `session::` (filter):**
- `session::` is a substring filter for *narrowing a listing* — it affects which sessions appear in results
- `session_id::` identifies *exactly one* session for a direct operation
- Different semantic purpose: filter expression vs direct identifier

**Parameter Details:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id::` | [`SessionId`](types.md#sessionid) | optional in `.show`, required in `.export` | Session filename stem (without `.jsonl`) |

**Examples:**
```bash
.show session_id::-default_topic
.export session_id::-default_topic output::conversation.md
```

---

## Session Filter

**Parameters:** `session::`, `agent::`, `min_entries::`

**Pattern:** Session listing narrowing by session properties

**Purpose:** Together these three parameters filter which sessions appear in a listing — by ID pattern, by session type, and by minimum size.

**Used By (full implementors):** `.list`, `.projects` (2 commands total)

**Partial implementors:**
- `.count` (`session::` only — as exact `SessionId`, not substring filter): scopes entry counting to a session
- `.search` (`session::` only — as exact `SessionId`, not substring filter): restricts search to a session

Note: In `.count` and `.search`, `session::` behaves as a `SessionId` (exact match), not as a `SessionFilter` (substring). The group semantics (substring filtering of session listings) apply only to `.list` and `.projects`.

**Semantic Coherence Test:**
- "Does `session::` control which sessions appear in listing?" → YES (by ID substring) — in `.list` and `.projects`
- "Does `agent::` control which sessions appear in listing?" → YES (by session type)
- "Does `min_entries::` control which sessions appear in listing?" → YES (by size threshold)

**Why NOT `sessions::` (bool):**
- `sessions::` controls whether sessions are shown at all — an on/off toggle for the entire session display tier
- These three parameters determine *which* sessions appear, assuming session display is enabled
- Different semantic level: tier visibility vs session predicate

**Why NOT `verbosity::`:**
- `verbosity::` controls how much information appears per session, not which sessions appear
- Different semantic purpose: output density vs session selection predicate

**Auto-enable behavior:** In `.list`, providing any of `session::`, `agent::`, or `min_entries::` automatically enables `sessions::1`. Override with `sessions::0`.

**Parameter Details:**

| Parameter | Type | Description | Side Effect |
|-----------|------|-------------|-------------|
| `session::` | [`SessionFilter`](types.md#sessionfilter) | Filter sessions by ID substring | Auto-enables `sessions::1` |
| `agent::` | Boolean | `0`=main only, `1`=agent only, unset=all | Auto-enables `sessions::1` |
| `min_entries::` | [`EntryCount`](types.md#entrycount) | Minimum entry count threshold | Auto-enables `sessions::1` |

**Examples:**
```bash
.list session::commit
.list agent::1
.list agent::0 min_entries::5
.list session::feature agent::0 min_entries::10
```

---

## Scope Configuration

**Parameters:** `scope::`, `path::`

**Pattern:** Discovery scope boundary and anchor

**Purpose:** Together these control the session discovery strategy: `scope::` selects the discovery algorithm and `path::` provides the filesystem anchor for scope resolution.

**Used By:** `.list` (scope:: only — path:: is PathSubstring in this command), `.count`, `.search`, `.show`, `.export`, `.projects` (6 commands total)

**Note on `.list` membership:** `.list` is a partial member — it accepts `scope::` for discovery boundary control, but its `path::` parameter remains a PathSubstring filter (not a StoragePath anchor); cwd is used as the implicit scope anchor in `.list`.

**Semantic Coherence Test:**
- "Does `scope::` control how session discovery is bounded?" → YES
- "Does `path::` control where session discovery is anchored?" → YES

**Why NOT `session::`, `agent::`, `min_entries::`:**
- Those parameters filter *which sessions* appear after discovery
- These parameters control *what gets discovered* (where and how)
- Different semantic layer: discovery configuration vs result filtering

**Why NOT `verbosity::`:**
- `verbosity::` controls output detail, not discovery behavior
- Different semantic purpose: output density vs discovery configuration

**Scope × Path interaction:**

| Scope | Path semantics |
|-------|----------------|
| `local` | Starting directory to look up (default: cwd) |
| `relevant` | Starting point for ancestor walk (default: cwd) |
| `under` | Root of subtree to descend (required when non-cwd) |
| `global` | Ignored (all projects regardless of path) |

**Mirrors kbase `DiscoveryConfig`:** The `scope` + `path` pair directly mirrors kbase's `DiscoveryConfig` group (scope, path, depth, role). This intentional alignment creates a consistent mental model across tools.

**Parameter Details:**

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `scope::` | [`ScopeValue`](types.md#scopevalue) | Discovery strategy: `local`\|`relevant`\|`under`\|`global` | `under` |
| `path::` | [`StoragePath`](types.md#storagepath) | Filesystem anchor for scope resolution | cwd |

**Examples:**
```bash
.projects scope::local
.projects scope::relevant
.projects scope::under path::/home/user1/pro
.projects scope::global
```
