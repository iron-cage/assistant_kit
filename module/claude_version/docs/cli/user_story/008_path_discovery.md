# Discover on-disk paths clv manages

**Persona:** developer
**Goal:** Run one command to see every filesystem path clv reads from or writes to — settings, versions directory, binary symlink, caches — without reading source code or guessing.
**Benefit:** Confirms exactly where clv-managed state lives on disk, for scripting, debugging, or manual inspection.
**Priority:** Medium

### Acceptance Criteria

- [ ] `clv .paths` lists all known paths with labels in a single view.
- [ ] `clv .paths key::versions_dir` shows the single resolved path for that key.
- [ ] `clv .paths v::0` outputs plain unlabeled paths suitable for piping.
- [ ] `clv .paths format::json` returns the same paths as a JSON object for scripting.
- [ ] Unresolvable paths (e.g., `project_settings` outside a project) are labeled "(none found)" at v::1/v::2, and omitted entirely at v::0.

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.paths`](../command/paths.md#command-16-paths) | Reports all clv-managed filesystem paths |

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [text](../format/01_text.md) | Default human-readable output |
| 2 | [json](../format/02_json.md) | Machine-readable output for scripting |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Output Control](../param_group/01_output_control.md) | Controls verbosity and format of path output |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`v::`](../param/04_v.md) | Controls label/omission behavior for unresolvable paths |
| 2 | [`format::`](../param/05_format.md) | Selects text or JSON rendering |
| 3 | [`key::`](../param/06_key.md) | Selects a single path to report |

### Workflow Steps

**Step 1 — List all known paths:**

```bash
clv .paths
# settings:               /home/user/.claude/settings.json
# project_settings:       (none found)
# versions_dir:           /home/user/.local/share/claude/versions
# binary_symlink:         /home/user/.local/bin/claude
# version_history_cache:  /home/user/.claude/.transient/version_history_cache.json
```

**Step 2 — Get a single path for scripting:**

```bash
clv .paths key::versions_dir v::0
# /home/user/.local/share/claude/versions
```

**Step 3 — Get machine-readable output:**

```bash
clv .paths format::json
# {"settings":"/home/user/.claude/settings.json","project_settings":null,"versions_dir":"/home/user/.local/share/claude/versions","binary_symlink":"/home/user/.local/bin/claude","version_history_cache":"/home/user/.claude/.transient/version_history_cache.json"}
```
