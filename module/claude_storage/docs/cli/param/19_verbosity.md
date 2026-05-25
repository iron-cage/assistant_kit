# Parameter :: 19. `verbosity::`

Output detail level controlling information density.

**Type:** [`VerbosityLevel`](../type/12_verbosity_level.md)

**Fundamental Type:** Integer wrapper (0-5 range)

**Constraints:**
- Valid range: `0`-`5` inclusive
- Error on out-of-range: `"verbosity must be 0-5, got {value}"`
- Error on non-integer: `"verbosity must be an integer 0-5, got {value}"`

**Default:** `1`

**Alias:** `v`

**Commands:** `.status`, `.list`, `.show`, `.search`, `.projects`

**Purpose:** Controls how much information each command outputs. Level `0` is minimal/machine-readable; level `1` is the standard summary; level `2` adds details; level `3` shows all fields; levels `4-5` are reserved.

**Examples:**
```bash
# Valid values
verbosity::0    # Minimal / machine-readable
verbosity::1    # Standard summary (default)
v::2            # Detailed (using alias)
v::3            # Verbose with all fields

# Invalid values
verbosity::6    # "verbosity must be 0-5, got 6"
verbosity::abc  # "verbosity must be an integer 0-5, got abc"
```

**Group:** [Output Control](../param_group/01_output_control.md)

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`VerbosityLevel`](../type/12_verbosity_level.md) | Integer wrapper | Integer | Range `0`–`5`; out-of-range rejected |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Full | *(sole member)* |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.status`](../command/01_status.md) | `1` | Controls output density; `0` is machine-readable |
| 2 | [`.list`](../command/02_list.md) | `1` | Controls output density |
| 3 | [`.show`](../command/03_show.md) | `1` | Controls output density |
| 5 | [`.search`](../command/05_search.md) | `1` | Controls output density |
| 7 | [`.projects`](../command/07_projects.md) | `1` | Controls output density; `0` is machine-readable |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |
