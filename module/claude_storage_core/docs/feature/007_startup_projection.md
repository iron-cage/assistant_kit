# Feature: Startup Projection

### Scope

- **Purpose**: Answer "what would a session started in this directory begin with" by reading the same directories Claude Code reads at startup — skills, agents, commands, settings files, and MCP config — before any session exists to ask a transcript instead.
- **Responsibility**: Documents `StartupProjection`, the user/project scope split, symlink-following for skill directories, why settings and clash resolution are deliberately left undecided, and the three named gaps a filesystem read cannot close.
- **In Scope**: `StartupProjection::resolve()`, `StartupProjection::resolve_in()`, `AssetOrigin`, `AssetScope`, `StartupProjection::clashes()`, the skill/agent/command roster rules, settings-file existence listing, `.mcp.json` detection, project-root resolution.
- **Out of Scope**: Settings precedence among the files listed (→ `claude_version_core::config_resolve`), install/uninstall of skills or agents (→ `claude_assets_core`), runtime session state — deferred tools, token usage, invoked skills (→ `ContextFold`, `docs/feature/006_transcript_answer.md`'s sibling, `context.rs`), nested `.claude/skills` deep-scan, bundled skills shipped inside the `claude` binary.

### Design

`ContextFold` answers "what does this session hold" by replaying a transcript — that needs a session to have existed and written one. `StartupProjection` answers the question one step earlier, before any session exists, by reading the directories Claude Code itself reads when one starts. The two are not interchangeable: a fold reports what *was* loaded, a projection reports what *would be*, and where they disagree the fold is right — a session can defer a tool, be started with `--disable-bundled-skills`, or predate a skill's installation.

**Two scopes, never merged.** Everything is read from both `<claude_home>` (user scope) and `<project_root>/.claude/` (project scope, when a project root exists), and every entry keeps its `AssetScope` rather than being collapsed into one roster. A skill directory is identified by containing `SKILL.md` — the marker, not mere presence, is what makes a directory a skill. Agents and commands are simpler: flat `.md` files, named by file stem, no marker needed because there is no ambiguous non-agent content a directory could hold instead.

**Symlinks are followed, deliberately.** Skills are routinely installed as symlinks into a separate assets tree. `is_dir()`/`is_file()` follow links by default, which is required here rather than incidental — a check that did not follow would report every symlinked skill as absent. The path recorded is still the link's own path inside `.claude/`, because that is what Claude Code reads; resolving it to wherever the link points would report a location nothing ever asks for.

**Why clashes are reported, not resolved.** Whether user or project wins when the same name exists in both scopes is not documented anywhere in the contract this crate is written against. Inventing an answer would be indistinguishable, to a caller, from a documented fact — so both entries survive in their roster, and `clashes()` separately names anything appearing under more than one scope, across skills, agents, and commands at once, sorted and deduplicated. A caller that cares is told there is a question; none is silently answered on its behalf.

**Why settings are existence-only.** `settings_files()` lists which of `<project>/.claude/settings.local.json`, `<project>/.claude/settings.json`, and `<claude_home>/settings.json` exist, project scope first — and reads none of their content. Which file's value for a given key actually wins is `claude_version_core::config_resolve`'s decision, already made and already correct; re-deciding it here would be exactly the kind of duplicated-and-liable-to-diverge logic this crate's own "avoid all duplication" rule exists to prevent.

**Project root is the nearest ancestor, not `cwd` alone.** A session started three directories below a project's `.claude/` is still in that project, matching how project settings are already resolved elsewhere in this workspace. The walk includes `cwd` itself before checking any parent, and terminates cleanly at the filesystem root with `None` rather than looping — a directory with no project anywhere above it is a real, user-scoped-only answer, not a failure.

**Nothing here fails.** An absent or unreadable directory contributes nothing to a roster and is not an error, at any level — a machine with no `~/.claude` at all is a fresh install, not a broken one, and reporting that case as an I/O failure would make the ordinary case unreachable.

**Three gaps, named rather than silently absent:**
- Bundled skills ship inside the `claude` binary itself — no filesystem read enumerates them, so a projection's skill list is always a lower bound.
- Nested `.claude/skills` directories, which load when Claude Code is working near them and appear under a directory-qualified name on a clash, require an unbounded tree scan this module does not perform.
- Runtime state — deferred tools, token usage, invoked skills, background tasks — cannot exist before a session does, so it is absent here because the question is unanswerable yet, not because the answer is empty.

### Algorithm

```text
resolve(cwd):
  1. home = scope_for(cwd).claude_home     # CLAUDE_HOME-aware, same as the rest of this crate
  2. return resolve_in(home, cwd)

resolve_in(claude_home, cwd):
  1. project_root = project_root_for(cwd)               # nearest ancestor (cwd included) with .claude/
  2. project_config = project_root / ".claude"           # or None
  3. skills   = directory_assets(claude_home, project_config, "skills",   SkillDir)
  4. agents   = directory_assets(claude_home, project_config, "agents",   MarkdownFile)
  5. commands = directory_assets(claude_home, project_config, "commands", MarkdownFile)
  6. sort each roster by (name, scope)                   # a value, not a directory-iteration artifact
  7. settings_files = [project_config/settings.local.json, project_config/settings.json,
                        claude_home/settings.json].filter(is_file)
  8. mcp_config = project_root.map(|r| r/".mcp.json").filter(is_file)
  9. return StartupProjection { cwd, claude_home, project_root, skills, agents, commands,
                                 settings_files, mcp_config }

project_root_for(cwd):
  1. current = cwd
  2. loop:
       if current/".claude" is a directory → return Some(current)
       current = current.parent()
       if current is None → return None                 # walked past the filesystem root

directory_assets(claude_home, project_config, roster, kind):
  1. found = read_roster(claude_home/roster, kind, User)
  2. if project_config is Some(config): found += read_roster(config/roster, kind, Project)
  3. return found

read_roster(dir, kind, scope):
  1. entries = read_dir(dir)                             # unreadable/absent → empty, not an error
  2. return entries.filter_map(|e| asset_from(e.path(), kind, scope))

asset_from(path, SkillDir, scope):
  1. if !(path/"SKILL.md").is_file() → None              # follows symlinks
  2. return AssetOrigin { name: path.file_name(), path, scope }

asset_from(path, MarkdownFile, scope):
  1. if !path.is_file() or path.extension() != "md" → None
  2. return AssetOrigin { name: path.file_stem(), path, scope }

clashes():
  1. names = [skills, agents, commands].flat_map(clashing_names)
  2. return names.sort().dedup()

clashing_names(roster):
  1. return roster.filter(|one| roster.any(|other| other.name == one.name && other.scope != one.scope))
                  .map(|one| one.name)
```

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| source | `../../src/startup.rs` | Full implementation |
| source | `../../src/scope.rs` | `scope_for()` — `CLAUDE_HOME` resolution this module reuses |
| doc | `../feature/006_transcript_answer.md` | The sibling "what a session has" question, answered from a transcript instead of disk |
| doc | `../../../claude_version/docs/feature/006_config_command.md` | `config_resolve` — settings precedence, deliberately not decided here |
| doc | `../../../claude_assets_core/readme.md` | Skill/agent install and uninstall, out of scope for a read-only projection |

### Sources

| File | Notes |
|------|-------|
| `../../src/startup.rs` | `StartupProjection`, `AssetOrigin`, `AssetScope`, `resolve`, `resolve_in`, `clashes` |
| `../../tests/startup_test.rs` | SP-1–SP-15 against hand-built temp directory trees |

### Tests

| File | Notes |
|------|-------|
| `../../tests/startup_test.rs` | SP-1–SP-15: user-scope and project-scope skills (SP-1, SP-2), a cross-scope clash surviving as two entries and one `clashes()` name (SP-3), a symlinked skill directory followed to the link's own path (SP-4), a marker-less directory excluded (SP-5), agents across both scopes (SP-6), non-`.md` entries excluded from commands (SP-7), settings-file existence and project-first ordering (SP-8), `.mcp.json` gated on project root and file presence (SP-9), project root found from a nested `cwd` (SP-10) and absent without panicking when none exists (SP-11), `clashes()` deduped and sorted across rosters (SP-12), deterministic roster sort (SP-13), a wholly absent `claude_home` reporting empty rather than erroring (SP-14), and `resolve()`'s `CLAUDE_HOME` wiring (SP-15) |
