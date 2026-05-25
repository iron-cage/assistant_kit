# Parameter Group :: 1. Output Control

**Parameters:** `verbosity::`

**Pattern:** Output verbosity and detail level control

**Purpose:** Controls how much information each read command outputs, from machine-readable minimal to full-field verbose.

**Used By:** `.status`, `.list`, `.show`, `.search`, `.projects` (5 commands total)

**Semantic Coherence Test:**
- "Does `verbosity::` control output detail level?" → YES

**Why NOT `entries::` and `metadata::`:**
- `entries::` controls *what content* is shown (all entries vs summary), not *how much detail* per line
- `metadata::` toggles content suppression — a mode switch, not a verbosity adjustment
- Different semantic purpose: display mode vs output density

**Why NOT `sessions::` (bool):**
- `sessions::` controls whether sessions are shown at all — an on/off toggle for the entire session display tier
- Different semantic level: tier visibility vs density of output

**Parameter Details:**

| Parameter | Type | Description | Alias |
|-----------|------|-------------|-------|
| `verbosity::` | [`VerbosityLevel`](../type/12_verbosity_level.md) | Output detail: 0=silent, 1=normal, 2=detailed, 3=verbose | `v` |

**Examples:**
```bash
.status v::2
.list verbosity::3
.search query::error v::0
```

### Referenced Commands

| # | Command | Membership | Excluded Params |
|---|---------|------------|-----------------|
| 1 | [`.status`](../command/01_status.md) | Full | — |
| 2 | [`.list`](../command/02_list.md) | Full | — |
| 3 | [`.show`](../command/03_show.md) | Full | — |
| 5 | [`.search`](../command/05_search.md) | Full | — |
| 7 | [`.projects`](../command/07_projects.md) | Full | — |

### Referenced Parameters

| # | Parameter | Type | Default | Role in Group |
|---|-----------|------|---------|---------------|
| 19 | [`verbosity::`](../param/19_verbosity.md) | [`VerbosityLevel`](../type/12_verbosity_level.md) | 1 | Output detail level |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |
