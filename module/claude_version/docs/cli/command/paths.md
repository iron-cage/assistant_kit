# paths — Path Discovery Command

### Scope

- **Purpose**: Reference for the `.paths` clv command.
- **Responsibility**: Command syntax, parameters, exit codes, examples, and cross-references for `.paths`.
- **In Scope**: `.paths` (show-all / single-key modes).
- **Out of Scope**: Runtime file lifecycle and ownership (→ `../runtime_file/`), unlabeled pipeline-only path enumeration (→ [`.runtime_files`](root.md#command--15-runtime_files)).

---

### Command :: 16. `.paths`

Report filesystem paths clv reads from or writes to: settings files, the versions directory, the binary symlink, and internal caches. Read-only — does not create, modify, or delete any file. Complements `.runtime_files` (unlabeled, pipeline-only, versions_dir/binary_symlink/history-cache subset) by adding labels, descriptions, and the externally-owned settings paths.

The operating mode is determined by whether `key::` is provided:

| Mode | Parameters | Behavior |
|------|------------|----------|
| show-all | (none) | All known paths, one per line, labeled |
| single | `key::K` | One resolved path for the given key |

-- **Parameters:** key::, format::, v::
-- **Exit Codes:** 0 (success) | 1 (invalid `key::` value) | 2 (HOME unset)

**Syntax:**

```sh
clv.paths [key::K] [format::FMT] [v::N]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`key::`](../param/06_key.md) | [`PathKey`](../type/09_path_key.md) | — | No | Specific path key for single-path mode |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Detail level: 0=plain paths only, 1=labeled, 2=labeled+description |

**`key::` values:**

| Value | Path |
|-------|------|
| absent | Show all paths |
| `settings` | `~/.claude/settings.json` |
| `project_settings` | `<cwd>/.claude/settings.json` (nearest project config) |
| `versions_dir` | `~/.local/share/claude/versions` |
| `binary_symlink` | `~/.local/bin/claude` |
| `version_history_cache` | `~/.claude/.transient/version_history_cache.json` |

**Algorithm (show-all, 3 steps):**
1. Resolve all 5 known paths via `ClaudeVersionPaths`.
2. At v::0, drop any path that did not resolve (e.g., no project config found for `project_settings`); at v::1/v::2, keep it with a "(none found)" placeholder.
3. Render the path table in requested format and verbosity.

**Algorithm (single-path, 3 steps):**
1. Validate `key::K` against the `PathKey` enum; exit 1 if unrecognized.
2. Resolve the requested path via `ClaudeVersionPaths`.
3. Render the single path (or placeholder, per verbosity) in requested format.

**Examples:**

```sh
# Show all known clv-managed paths
clv.paths

# Single path for scripting
clv.paths key::versions_dir v::0

# Machine-readable output
clv.paths format::json
clv.paths key::settings format::json

# Verbose output with descriptions
clv.paths v::2
```

**Sample text output (v::1, `clv.paths`):**

```
settings:               /home/user/.claude/settings.json
project_settings:       (none found)
versions_dir:           /home/user/.local/share/claude/versions
binary_symlink:         /home/user/.local/bin/claude
version_history_cache:  /home/user/.claude/.transient/version_history_cache.json
```

**Sample text output (v::0, `clv.paths key::versions_dir`):**

```
/home/user/.local/share/claude/versions
```

**Sample text output (v::2, `clv.paths key::binary_symlink`):**

```
binary_symlink:  /home/user/.local/bin/claude
  Hot-swap target; retargeted by .version.install to activate a version
```

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [text](../format/01_text.md) | Default human-readable output |
| 2 | [json](../format/02_json.md) | Machine-readable structured output |

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|-----------|----------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Partial | `count::` |

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 1 | [`v::`](../param/04_v.md) |
| 2 | [`format::`](../param/05_format.md) |
| 3 | [`key::`](../param/06_key.md) |

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.runtime_files`](root.md#command--15-runtime_files) | Unlabeled pipeline-only subset (versions_dir, binary_symlink, version_history_cache only) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [008 Path Discovery](../user_story/008_path_discovery.md) | Developer (path discovery and scripting) |

---

**Category:** paths
**Complexity:** 6
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** None (read-only)
