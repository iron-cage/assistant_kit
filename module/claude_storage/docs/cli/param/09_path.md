# Parameter :: 9. `path::`

### Scope

- **Purpose**: Specify the `path::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `path::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Path argument. Semantics differ by command — see command sections for exact behavior.

**Type:** [`StoragePath`](../type/10_storage_path.md) or [`PathSubstring`](../type/04_path_substring.md) depending on command

**Fundamental Type:** String

**Constraints:** Command-dependent (see table below)

**Default:** Command-dependent

**Commands:** `.status`, `.list` (deprecated), `.projects`, `.count`, `.show`, `.search`, `.export`, `.project.path`, `.project.exists`, `.session.dir`, `.session.ensure`, `.tail`, `.usage`, `.rollup`, `.session.path` — registered.

**Per-command semantics:**

| Command | Type | Default | Semantics |
|---------|------|---------|-----------|
| `.status` | StoragePath | `~/.claude/` | Storage root override |
| `.list` (deprecated) | PathSubstring | — | Filter projects by path substring (case-insensitive); this role moved to `.projects`' new [`filter::`](29_filter.md) parameter — NOT to `.projects`' own `path::` below, which keeps its distinct StoragePath meaning |
| `.projects` | StoragePath | cwd | Scope anchor path (implemented) |
| `.count` | String | `~/.claude/` | Storage root override — not a scope anchor; `.count` narrows via `scope::` alone (no `path::` anchor role) |
| `.search` | StoragePath | cwd | Scope anchor path for `scope::`-resolved project search when `project::` is absent (implemented) |
| `.show` | StoragePath | cwd | Scope anchor path for `scope::`-resolved session lookup when `session_id::` is given without `project::` (implemented) |
| `.export` | StoragePath | cwd | Scope anchor path for `scope::`-resolved source-session lookup when `project::` is absent (implemented) |
| `.project.path` | StoragePath | cwd | Directory to compute storage path for |
| `.project.exists` | StoragePath | cwd | Directory to check for history |
| `.session.dir` | StoragePath | cwd | Base directory |
| `.session.ensure` | StoragePath | cwd | Base directory |
| `.tail` | StoragePath | cwd | Directory to resolve project from |
| `.usage` | StoragePath | cwd | Scope anchor path (implemented) |
| `.rollup` | StoragePath | cwd | Scope anchor path — identical role to `.usage` (implemented) |
| `.session.path` | StoragePath | cwd | Base directory whose storage holds the session file (canonicalized before encoding) |

**Purpose:** Provides a path context appropriate to each command. In `.project.exists`, `.project.path`, `.session.dir`, `.session.ensure`, and `.session.path`, it is a filesystem path to process. In `.list` (deprecated), it was a substring filter on project paths — that role is now `.projects`' [`filter::`](29_filter.md). In `.status` and `.count`, it overrides the storage root entirely. In `.projects`, `.search`, `.show`, `.export`, [`.usage`](../command/13_usage.md), and [`.rollup`](../command/14_rollup.md), it anchors the scope discovery when paired with `scope::` (all six implemented).

**Examples:**
```bash
# .status: storage root override
.status path::~/.claude/

# .list (deprecated): path substring filter — use .projects filter:: instead
.list path::assistant          # Matches all projects with "assistant" in path

# .project.exists: directory check
.project.exists path::/home/user/project

# .project.path: storage path computation
.project.path path::/home/user/project

# .session.dir / .session.ensure: base directory (cwd when omitted)
.session.dir path::/home/user/project
.session.ensure path::/home/user/project

# .projects: scope anchor (one of four commands implementing this role)
.projects scope::under path::/home/alice/projects

# .search / .show / .export: scope anchor for scope::-resolved lookups
.search query::error scope::under path::/home/alice/projects
.show session_id::abc123 scope::under path::/home/alice/projects
.export session_id::abc123 output::out.md scope::under path::/home/alice/projects

# .count: storage root override (not a scope anchor — .count narrows via scope:: alone)
.count path::/alt/storage/root

# .tail: directory to resolve project from
.tail path::/home/alice/projects/my-app

# .rollup: scope anchor, identical role to .usage
.rollup scope::under path::/home/alice/projects
```

**Group (scope anchor context):** [Scope Configuration](../param_group/05_scope_configuration.md) — `path::` acts as the scope anchor paired with `scope::` in `.projects`, `.search`, `.show`, `.export`, [`.usage`](../command/13_usage.md), and [`.rollup`](../command/14_rollup.md) (all implemented). `.count` is a Partial member of this group via `scope::` alone — its own `path::` keeps a separate storage-root-override role, not the anchor role. `path::`'s role in `.status`, `.list` (deprecated), `.project.exists`, `.project.path`, `.session.dir`, and `.session.ensure` is independent and not part of this group.

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`StoragePath`](../type/10_storage_path.md) | String (filesystem path) | String | Filesystem path; `~` expansion supported |
| [`PathSubstring`](../type/04_path_substring.md) | String | String | Historical: in `.list` (deprecated) only. Current substring-filter role lives on `.projects`' [`filter::`](29_filter.md), a distinct parameter — not `path::`. |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Partial — anchor role implemented via `.projects`, `.search`, `.show`, `.export`, `.usage`, `.rollup` | `scope::` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.status`](../command/01_status.md) | `~/.claude/` | Storage root override |
| 2 | [`.list`](../command/02_list.md) (deprecated) | — | Historical PathSubstring role; superseded by `.projects`' [`filter::`](29_filter.md) |
| 3 | [`.show`](../command/03_show.md) | cwd | Scope anchor path — implemented |
| 4 | [`.count`](../command/04_count.md) | `~/.claude/` | Storage root override, not a scope anchor — `.count` narrows via `scope::` alone |
| 5 | [`.search`](../command/05_search.md) | cwd | Scope anchor path — implemented |
| 6 | [`.export`](../command/06_export.md) | cwd | Scope anchor path — implemented |
| 7 | [`.projects`](../command/07_projects.md) | cwd | Scope anchor path — implemented |
| 8 | [`.project.path`](../command/08_project_path.md) | cwd | Directory to compute storage path for |
| 9 | [`.project.exists`](../command/09_project_exists.md) | cwd | Directory to check for history |
| 10 | [`.session.dir`](../command/10_session_dir.md) | cwd | Base directory |
| 11 | [`.session.ensure`](../command/11_session_ensure.md) | cwd | Base directory |
| 12 | [`.tail`](../command/12_tail.md) | cwd | Directory to resolve project from |
| 13 | [`.usage`](../command/13_usage.md) | cwd | Scope anchor path — implemented |
| 14 | [`.rollup`](../command/14_rollup.md) | cwd | Scope anchor path — implemented |
| 15 | [`.session.path`](../command/15_session_path.md) | cwd | Base directory whose storage holds the session file |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
| 3 | [Export Session for Review](../user_story/003_export_session_for_review.md) | developer |
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |
| 5 | [Resume Claude Session](../user_story/005_resume_claude_session.md) | developer |
| 6 | [Quick Context Refresh](../user_story/006_quick_context_refresh.md) | developer |
