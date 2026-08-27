# Storage: Support Directories

### Scope

- **Purpose**: Document the operational support directories in `~/.claude/` that store debug logs, task tracking, shell environment, session metadata, and command definitions.
- **Responsibility**: Authoritative instance for the support directories — purpose, format, growth characteristics, and maintenance guidance for each, together with the provenance evidence that each name is one the `claude` binary itself creates.
- **In Scope**: All non-`projects/` subdirectories of `~/.claude/` created by the binary: `debug/`, `tasks/`, `shell-snapshots/`, `session-env/`, `commands/`, `sessions/`, `agents/`, `skills/`, `hooks/`, `cache/`, `plans/`, `plugins/`, `backups/`, `paste-cache/`, `ide/`, `chrome/`, `jobs/`, `daemon/`, `workflows/`, `routines/`, `rules/`, `output-styles/`, `file-history/`, `logs/`, `statsig/`, and the legacy `todos/`.
- **Out of Scope**: `projects/` (conversation storage, → [001_projects_directory.md](001_projects_directory.md)); global root files (→ [003_root_files.md](003_root_files.md)); file format internals (→ [`../format/`](../format/readme.md)).

### Structure

Every name below is confirmed against the v2.1.220 binary, not merely observed on one
machine — see § Provenance. **No size appears in this tree**: an earlier revision quoted
`debug/ 429MB`, `todos/ 63MB`, `shell-snapshots/ 45MB` as if they were properties of the
format. They are properties of one machine on one day. On the machine used for this
revision `debug/` is 648K and the 63MB belongs to `tasks/`, which did not exist under that
name. Sizes still appear further down, but only inside an explicit "on the surveyed
machine" clause — never as a characteristic of the directory. Measure your own with
`du -sh ~/.claude/*`.

```
~/.claude/
├── tasks/                # Per-session task JSON files (was todos/ — see § Rename below)
├── shell-snapshots/      # Session shell environment captures
├── session-env/          # Session metadata (empty directories)
├── commands/             # Custom slash command definitions (.md)
├── sessions/             # Session tracking metadata
├── agents/               # Agent configuration and state
├── skills/               # User-defined skill definitions
├── hooks/                # Hook script storage
├── debug/                # Debug log files
├── cache/                # General-purpose cache
├── paste-cache/          # Pasted-content cache, swept by age
├── plans/                # Saved plans (.md)
├── plugins/              # Installed plugin trees
├── backups/              # Backup copies
├── file-history/         # Per-file edit history
├── ide/                  # IDE integration state
├── chrome/               # Chrome/browser integration state
├── jobs/                 # Background job state
├── daemon/               # Daemon state
├── workflows/            # Workflow definitions
├── routines/             # Routine definitions
├── rules/                # Rule definitions
├── output-styles/        # Output style definitions
├── logs/                 # Logs (cleanup-swept)
├── statsig/              # Feature-flag SDK state (cleanup-swept)
└── todos/                # LEGACY — superseded by tasks/; still cleanup-swept
```

Not every name appears on every machine — a directory is created lazily, on first use of
the feature that needs it. Absence is not evidence against the name; presence of a name
*not* in this list is evidence it belongs to something other than the `claude` binary.

### Rename: `todos/` → `tasks/`

The binary carries both names, in three distinct roles, which together establish a rename
with retained backward compatibility rather than two coexisting directories:

| Binary string | Role |
|---------------|------|
| `join(home(),"tasks")` (2×) | `tasks/` is the live directory the binary reads and writes |
| `"tasks" in e \|\| "todos" in e` | A schema predicate accepting **either** key — the compatibility shim |
| `for(let n of["todos","statsig","logs"])` | `todos/` appears only in a *cleanup sweep* list, never in a path join |

No `join(home(),"todos")` construction exists. Documentation that lists `todos/` as a live
directory is describing a version older than 2.1.220. The exact release that renamed it is
not established here — the changelog in [`../version/`](../version/readme.md) does not
mention either name, so treat "pre-2.1.220" as the only supported claim.

### Provenance

Directory names were confirmed by string-scanning the installed binary, the same method
used for env vars in the behavior collection's E72/E76. Two independent patterns, each
with a fabricated negative control returning 0:

```bash
V=~/.local/share/claude/versions/$(claude --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')

# 1. path-join construction — strongest signal, proves the name is used AS a directory
grep -aoE 'join\([A-Za-z_$0-9]{1,8}\(\),"tasks"' "$V" | wc -l      # → 2
grep -aoE 'join\([A-Za-z_$0-9]{1,8}\(\),"NEVER_REAL_XYZ"' "$V" | wc -l   # → 0  (control)

# 2. the maintenance array that enumerates home subdirectories in one literal
grep -aoE '.{80}"shell-snapshots","session-env".{200}' "$V"
```

The second command returns the authoritative list in a single string:
`["shell-snapshots","session-env","plugins","hooks","skills","workflows","commands","agents","routines","rules","output-styles","scheduled_tasks.json","launch.json","CLAUDE.md","projects","daemon.json","policy-limits.json","backups"]`,
immediately preceded by `resolve(home(),"jobs")` and `resolve(home(),"daemon")`.

**Do not use a bare substring count.** `grep -ac ide "$V"` returns 5010 because `ide`
occurs inside `provide`, `identifier`, and `width`; `grep -ac tasks` returns 555 for
similar reasons. Both patterns above anchor on syntax that a coincidental substring
cannot satisfy. This is the same masking failure recorded as Fix(A5) in
[`../../tests/behavior/b14_agent_meta_json.rs`](../../tests/behavior/b14_agent_meta_json.rs).

**Names observed on disk but NOT attributable to the binary**: `downloads/`, `docs/`, and
`.transient/` each return 0 for both patterns above. They exist in the surveyed
`~/.claude/` but no evidence here ties them to `claude` — they may come from a plugin,
another tool, or a version other than the one scanned. Recorded as unattributed rather
than silently listed as Claude Code's.

One of the three has since been attributed, and to *this repo* rather than to the binary:
`.transient/` holds `version_history_cache.json`, written by `clv` via
`version_history_cache_path()` in `claude_version_core/src/version.rs`. The 0 score against
the binary was correct — the directory is simply not Claude Code's. See
[`../filesystem/001_claude_home.md`](../filesystem/001_claude_home.md) § Unattributed Paths,
which applies the same treatment to `downloads/`. `docs/` remains unattributed on both sides.

### Contents

#### debug/ — Debug Logs

**Purpose**: Debug output from Claude Code operations.
**Format**: Plain text; one `[DEBUG] message` line per log entry.
**Growth**: Continuous append during operation. Can grow to 100MB+ per file over time.
**Maintenance**: Safe to delete entirely. No impact on conversations or settings.

Content types: setting file watching, plugin loading, LSP server init, shell snapshot creation, process lifecycle events.

See [`../format/003_debug_log.md`](../format/003_debug_log.md) for format spec.

#### tasks/ — Task Tracking

**Purpose**: Store per-session task lists.
**File organization**: One **directory** per session UUID, not one file:
`tasks/{session-uuid}/{n}.json` — one JSON file per task, numbered from `1` — plus a
0-byte `.lock` sibling in every session directory.
**Format**: a single JSON **object** per file with `id`, `subject`, `description`,
`status`, `blocks`, `blockedBy`, and optional `activeForm`. Not a JSON array, and there is
no `content` field — both belonged to the superseded `todos/` layout.
**Growth**: one file per task, rewritten in place on status change; sibling tasks are not
touched. Typically the largest support directory after `projects/` — on the surveyed
machine 72MB across 250 session directories, 17822 task files, 250 `.lock` files.
**Maintenance**: Can be deleted if corresponding sessions are no longer needed.

Formerly `todos/`; see § Rename above for the binary evidence. The supersession is total
— path, granularity, container type, and field names all changed — and is recorded in full
by [`../format/005_task.md`](../format/005_task.md). A consumer written against the old
shape fails *silently*: globbing `~/.claude/todos/*.json` simply matches nothing.

Check the layout yourself: `ls ~/.claude/tasks | head -3` shows UUID directories, and
`ls ~/.claude/tasks/$(ls ~/.claude/tasks | head -1)` shows `1.json 2.json … .lock`.

#### shell-snapshots/ — Shell Environment Captures

**Purpose**: Preserve shell environment for session restoration.
**File naming**: UUID matches session ID: `shell-snapshots/{session-uuid}.sh`.
**Format**: Executable bash script; functions base64-encoded to preserve complex syntax.
**Growth**: One file per CLI session with shell context. Size: 5KB–500KB per snapshot.
**Maintenance**: Old snapshots can be deleted safely; only affects ability to restore old sessions.

See [`../format/004_shell_snapshot.md`](../format/004_shell_snapshot.md) for format spec.

#### session-env/ — Session Metadata

**Purpose**: Store session-specific metadata.
**Current status**: Empty directories named by session UUID. No files observed — directories exist as placeholders.
**Growth**: One empty directory per session (minimal disk impact).

#### commands/ — Command Definitions

**Purpose**: Store custom slash command definitions available as `/{command-name}` in Claude Code sessions.
**File format**: Markdown files (`.md`) — count is user-determined (124 on the surveyed machine).
**Examples**: `commit.md`, `refactor_extracting.md`, `test_clean.md`
**Growth**: Static — only grows when user adds new custom commands.
**Maintenance**: Do not delete unless removing custom commands intentionally.

See [`../format/006_command_definition.md`](../format/006_command_definition.md) for format spec.

#### sessions/ — Session Tracking Metadata

**Purpose**: Store session-level tracking data and metadata independent of conversation content.
**File organization**: Session tracking files keyed by session UUID.
**Growth**: One entry per session.
**Maintenance**: Safe to delete; does not affect conversation history.

#### agents/ — Agent Configuration and State

**Purpose**: Store agent configuration — registered custom agents and their definitions.
**File format**: JSON files defining agent properties (description, prompt, model).
**Growth**: Static — grows only when user registers new agents via `claude agents` subcommand.
**Maintenance**: Safe to delete individual agent configs; agent will need to be re-registered.

#### skills/ — User-Defined Skill Definitions

**Purpose**: Store user-created skills (slash commands) beyond the built-in skill set. Distinct from `commands/` which stores the markdown-format command definitions — `skills/` stores skill metadata and registration.
**Growth**: Static — grows only when user creates new skills.
**Maintenance**: Safe to delete; skills will need to be re-created.

#### hooks/ — Hook Script Storage

**Purpose**: Store hook scripts referenced by the `hooks` settings configuration. Scripts executed at `PreToolUse`, `PostToolUse`, and `UserPromptSubmit` lifecycle events.
**Growth**: Static — grows only when user creates new hook scripts.
**Maintenance**: Do not delete while hooks are active in settings; will cause hook execution failures.

#### Directories confirmed by name but not yet characterized

The sixteen below are confirmed Claude Code directory names by the § Provenance scans, but
their internal formats have not been inspected. They are listed so the collection stops
implying `~/.claude/` has only nine support directories — **naming them is a stronger claim
than describing them, and only the naming is evidenced here.** Anything beyond the middle
column is ❓ Uncertain and marked as such rather than guessed.

| Directory | Evidence | On surveyed machine |
|-----------|----------|---------------------|
| `cache/` | `join(home(),"cache")` ×9 | present, 564K |
| `paste-cache/` | `zc_="paste-cache"`, adjacent to "old paste" cleanup code | present, 3.4M / 190 items |
| `plans/` | `join(home(),"plans")` ×2; read with a `.md` extension filter | present, 172K / 8 items |
| `plugins/` | `join(home(),"plugins")`; also in the maintenance array | present, 7.2M / 6 items |
| `backups/` | `join(home(),"backups")` ×2; also in the maintenance array | present, 2.6M, empty |
| `ide/` | `join(home(),"ide")`, and `join(homedir(),".claude","ide")` | present, 12K / 2 items |
| `chrome/` | `join(home(),"chrome")` | present, 8K / 1 item |
| `sessions/` | `join(home(),"sessions")` ×2 | present, 32K / 7 items |
| `jobs/` | `resolve(home(),"jobs")` | absent |
| `daemon/` | `resolve(home(),"daemon")` | absent (the `-daemon/` on disk is a user directory, not this) |
| `workflows/` | maintenance array | absent |
| `routines/` | maintenance array | absent |
| `rules/` | maintenance array | absent |
| `output-styles/` | maintenance array | absent |
| `file-history/` | `vjt("file-history")`, same helper as `session-env` | absent |
| `logs/`, `statsig/` | cleanup sweep list `["todos","statsig","logs"]` | absent |

Absent-on-disk is expected: these are created lazily on first use of the owning feature.
To check any of them yourself: `ls -la ~/.claude/<name>` and, for provenance,
`grep -aoE 'join\([A-Za-z_$0-9]{1,8}\(\),"<name>"' "$V"` from § Provenance.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Storage master index: full directory structure |
| formats | [`../format/003_debug_log.md`](../format/003_debug_log.md) | Debug log `[DEBUG]` line format |
| formats | [`../format/004_shell_snapshot.md`](../format/004_shell_snapshot.md) | Shell snapshot bash script format |
| formats | [`../format/005_task.md`](../format/005_task.md) | Per-task JSON object format under `tasks/{session}/` |
| formats | [`../format/006_command_definition.md`](../format/006_command_definition.md) | Command definition markdown format |
