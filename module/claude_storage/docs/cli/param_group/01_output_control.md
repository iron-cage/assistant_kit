# Parameter Group :: 1. Output Control

### Scope

- **Purpose**: Specify the Output Control parameter group.
- **Responsibility**: Member parameters, coherence semantics, and command usage for Output Control.
- **In Scope**: Group membership, shared behavior, command interactions.
- **Out of Scope**: Individual parameter specs (→ `param/`), type constraints (→ `type/`).

**Parameters:** `show_stat::`, `show_tokens::`, `show_tree::`

**Pattern:** Per-block boolean toggles for optional output sections

**Purpose:** Controls which optional information blocks are included in command output. Each parameter independently enables one additional section; the default output is always the standard summary.

**Used By:** `.show` (show_stat, show_tokens), `.status` (show_tokens), `.projects` (show_tree) — 3 commands total

**Semantic Coherence Test:**
- "Does each parameter toggle a specific optional output block?" → YES

**Why NOT `show_entries::` and `show_metadata::`:**
- `show_entries::` controls *what content* is shown (all entries vs summary), not an optional extra block
- `show_metadata::` is a mode switch (suppresses content), not an additive block
- Different semantic purpose: display mode vs optional section visibility

**Why NOT `show_sessions::` (bool):**
- `show_sessions::` controls whether the session tier is shown at all — an on/off toggle for the entire session display tier
- Different semantic level: tier visibility vs optional section within a tier

**Parameter Details:**

| Parameter | Type | Description | Commands |
|-----------|------|-------------|----------|
| `show_stat::` | Boolean | Append statistics footer (entry counts, timestamps) | `.show` |
| `show_tokens::` | Boolean | Include token usage section | `.show`, `.status` |
| `show_tree::` | Boolean | Tree-indent agent sessions under root sessions | `.projects` |

**Examples:**
```bash
.show session_id::abc123 show_stat::1
.show session_id::abc123 show_tokens::1
.status show_tokens::1
.projects show_tree::1
```

### Referenced Commands

| # | Command | Parameters |
|---|---------|------------|
| 1 | [`.status`](../command/01_status.md) | `show_tokens::` |
| 3 | [`.show`](../command/03_show.md) | `show_stat::`, `show_tokens::` |
| 7 | [`.projects`](../command/07_projects.md) | `show_tree::` |

### Referenced Parameters

| # | Parameter | Type | Default | Role in Group |
|---|-----------|------|---------|---------------|
| 19 | [`show_stat::`](../param/19_show_stat.md) | Boolean | `0` | Statistics footer in `.show` content mode |
| 23 | [`show_tokens::`](../param/23_show_tokens.md) | Boolean | `0` | Token usage section in `.show` and `.status` |
| 24 | [`show_tree::`](../param/24_show_tree.md) | Boolean | `0` | Tree-indented agent display in `.projects` |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
