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

**Commands:** `.status`, `.list`, `.projects`, `.count`, `.search`, `.show`, `.export`, `.project.path`, `.project.exists`, `.session.dir`, `.session.ensure`

**Per-command semantics:**

| Command | Type | Default | Semantics |
|---------|------|---------|-----------|
| `.status` | StoragePath | `~/.claude/` | Storage root override |
| `.list` | PathSubstring | — | Filter projects by path substring (case-insensitive) |
| `.projects` | StoragePath | cwd | Scope anchor path |
| `.count` | StoragePath | cwd | Scope anchor path |
| `.search` | StoragePath | cwd | Scope anchor path |
| `.show` | StoragePath | cwd | Scope anchor path |
| `.export` | StoragePath | cwd | Scope anchor path |
| `.project.path` | StoragePath | cwd | Directory to compute storage path for |
| `.project.exists` | StoragePath | cwd | Directory to check for history |
| `.session.dir` | StoragePath | cwd | Base directory |
| `.session.ensure` | StoragePath | cwd | Base directory |

**Purpose:** Provides a path context appropriate to each command. In `.project.exists`, `.project.path`, `.session.dir`, and `.session.ensure`, it is a filesystem path to process. In `.list`, it is a substring filter on project paths. In `.projects`, `.count`, `.search`, `.show`, and `.export`, it anchors the scope discovery when paired with `scope::`.

**Examples:**
```bash
# .status: storage root override
.status path::~/.claude/

# .list: path substring filter
.list path::assistant          # Matches all projects with "assistant" in path

# .project.exists: directory check
.project.exists path::/home/user/project

# .project.path: storage path computation
.project.path path::/home/user/project

# .session.dir / .session.ensure: base directory (cwd when omitted)
.session.dir path::/home/user/project
.session.ensure path::/home/user/project

# .projects / .count / .search / .show / .export: scope anchor
.projects scope::under path::/home/alice/projects
.count scope::under path::/home/alice/projects
.search query::error scope::under path::/home/alice/projects
```

**Group (scope anchor context):** [Scope Configuration](../param_group/05_scope_configuration.md) — `path::` acts as the scope anchor paired with `scope::` in `.projects`, `.count`, `.search`, `.show`, and `.export`; its role in `.status`, `.list`, `.project.exists`, `.project.path`, `.session.dir`, and `.session.ensure` is independent and not part of this group.

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`StoragePath`](../type/10_storage_path.md) | String (filesystem path) | String | Filesystem path; `~` expansion supported |
| [`PathSubstring`](../type/04_path_substring.md) | String | String | In `.list` only: substring filter on project paths |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.status`](../command/01_status.md) | `~/.claude/` | Storage root override |
| 2 | [`.list`](../command/02_list.md) | — | PathSubstring type: substring filter on project paths |
| 3 | [`.show`](../command/03_show.md) | cwd | Scope anchor path |
| 4 | [`.count`](../command/04_count.md) | cwd | Scope anchor path |
| 5 | [`.search`](../command/05_search.md) | cwd | Scope anchor path |
| 6 | [`.export`](../command/06_export.md) | cwd | Scope anchor path |
| 7 | [`.projects`](../command/07_projects.md) | cwd | Scope anchor path |
| 8 | [`.project.path`](../command/08_project_path.md) | cwd | Directory to compute storage path for |
| 9 | [`.project.exists`](../command/09_project_exists.md) | cwd | Directory to check for history |
| 10 | [`.session.dir`](../command/10_session_dir.md) | cwd | Base directory |
| 11 | [`.session.ensure`](../command/11_session_ensure.md) | cwd | Base directory |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Full | `scope::` |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
| 3 | [Export Session for Review](../user_story/003_export_session_for_review.md) | developer |
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |
| 5 | [Resume Claude Session](../user_story/005_resume_claude_session.md) | developer |
