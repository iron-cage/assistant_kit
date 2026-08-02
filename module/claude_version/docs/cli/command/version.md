# version — Version Namespace Commands

### Scope

- **Purpose**: Reference for version-namespace clv commands.
- **Responsibility**: Command syntax, parameters, exit codes, and cross-references for all `.version.*` commands.
- **In Scope**: `.version.show`, `.version.install`, `.version.guard`, `.version.list` (alias and release-history listing via `mode::`), `.version.paths` (filesystem path discovery).
- **Out of Scope**: Root commands (→ [root.md](root.md)), process commands (→ [processes.md](processes.md)), settings commands (→ [settings.md](settings.md)).

---

### Command :: 3. `.version.show`

Print the currently installed Claude Code version by querying `claude --version`. Use this to verify what is currently installed before upgrading or troubleshooting.

-- **Parameters:** v::, format::
-- **Exit Codes:** 0 (success) | 2 (binary not found)

**Syntax:**

```sh
clv.version.show [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Output detail level |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |

**Algorithm (2 steps):**
1. Invoke `claude --version` to detect the installed binary version string.
2. Render the version string in the requested format.

**Examples:**

```sh
clv.version.show
clv.version.show format::json
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

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.version.install`](#command-4-versioninstall) | Installs the version currently displayed |
| 2 | [`.version.guard`](#command-5-versionguard) | Restores preferred version if drift detected |
| 3 | [`.version.list`](#command-6-versionlist) | Lists aliases or release history relevant to the installed version |
| 4 | [`.status`](root.md#command-2-status) | Includes version in broader environment snapshot |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 2 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |

---

**Category:** version
**Complexity:** 2
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low

---

### Command :: 4. `.version.install`

Download and install a Claude Code version via the official installer (curl). Supports hot-swap and 8-layer version locking (Layers 1–4, 6, and 8 prevent unwanted version changes via auto-update, manual update, or channel drift; Layer 5 stores the preferred version as a recovery signal for `.version.guard`; Layer 7 enforces a minimum-version floor). Accepts named aliases (`stable`, `latest`, `month`) and semver strings. Already-at-target is a no-op (exit 0) unless `force::1` is set. `record_only::1` persists the resolved preference to `settings.json` without invoking the installer at all — see Algorithm below.

-- **Parameters:** version::, dry::, force::, record_only::, v::, format::
-- **Exit Codes:** 0 (success) | 1 (invalid version spec, or `record_only::1`+`dry::1` both set) | 2 (installer failure)

**Syntax:**

```sh
clv.version.install [version::VER] [dry::1] [force::1] [record_only::1] [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`version::`](../param/01_version.md) | [`VersionSpec`](../type/03_version_spec.md) | stable | No | Version to install |
| [`dry::`](../param/02_dry.md) | bool | false | No | Preview install command without executing |
| [`force::`](../param/03_force.md) | bool | false | No | Bypass idempotency check |
| [`record_only::`](../param/15_record_only.md) | bool | false | No | Persist preference only; skip the installer (mutually exclusive with `dry::`) |
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Output detail level |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |

**Algorithm (7 steps):**
0. If `record_only::1` and `dry::1` are both set, reject immediately (exit 1, `ArgumentMissing`) — the two are mutually exclusive.
1. Resolve `version::` alias (`stable`, `latest`, `month`) or validate the semver string against known patterns.
2. Compare resolved target against installed version; exit 0 (no-op) if equal and `force::0` — the preferred version is still stored on this path.
3. Store the preferred version spec and resolved value in `settings.json`. Recorded before the lock mechanism is applied (step 6) so that a crash partway through install leaves a true, not false, mismatch signal in `.status`'s `Lock:` section — see `tests/docs/cli/command/02_status.md` IT-24–IT-27 and TC-530.
4. Hot-swap the running binary if any Claude Code process is active, then unlock the versions directory so the installer can write to it.
5. Execute the official curl installer for the resolved version; purge stale cached binaries afterward for pinned installs (skipped for `latest`, so version history remains available for rollback).
6. Apply the lock mechanism for pinned installs — `autoUpdates`, `autoUpdatesChannel`, `minimumVersion`, `env.DISABLE_AUTOUPDATER`, `env.DISABLE_UPDATES`, and `chmod 555` on the versions directory — or leave unlocked (`autoUpdates` true, the other 4 keys removed, `chmod 755`) for `latest`.

`record_only::1` short-circuits between steps 1 and 2: after resolving the version
spec, it performs step 3's `settings.json` write directly — unconditionally,
without the step 2 idempotency comparison — and returns. Steps 2, 4, 5, and 6
never run: no hot-swap, no `curl`, no lock mechanism applied. `force::1` has
nothing to bypass in this path and is silently ignored rather than rejected.

**Examples:**

```sh
# Install the pinned stable version (default)
clv.version.install

# Dry-run shows all 5 lock layers
clv.version.install version::stable dry::1

# Idempotent skip: already at target, stores preference and exits 0
clv.version.install version::stable

# Force reinstall even if already at target version
clv.version.install force::1

# Install latest (no version pin — resolves dynamically)
clv.version.install version::latest

# Record "month" as preferred without downloading/installing
clv.version.install version::month record_only::1

# Rejected: record_only:: and dry:: are mutually exclusive
clv.version.install version::month record_only::1 dry::1
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
| 2 | [Execution Control](../param_group/02_execution_control.md) | Full | — |

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 1 | [`version::`](../param/01_version.md) |
| 2 | [`dry::`](../param/02_dry.md) |
| 3 | [`force::`](../param/03_force.md) |
| 4 | [`record_only::`](../param/15_record_only.md) |
| 5 | [`v::`](../param/04_v.md) |
| 6 | [`format::`](../param/05_format.md) |

### Referenced Command Group

Evaluated against `.version.guard` (which invokes install logic on drift; see step 4 above) under the strict [command_group](../command_group/readme.md) identity test — does not qualify. `version_install_routine()` (`src/commands/version.rs:75`) and `version_guard_routine()` (`src/commands/version.rs:240`) never call each other directly; what they share is `perform_install()` and `validate_version_spec()`, both imported from the separate `claude_version_core` crate, not one routine calling the other. Parameter sets also differ (`.version.guard` adds `interval::` for watch mode, with no `.version.install` equivalent). See [`command_group/readme.md`](../command_group/readme.md) Evaluated, Not Qualifying for the full analysis.

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.version.show`](#command-3-versionshow) | Verifies installed version after install |
| 2 | [`.version.guard`](#command-5-versionguard) | Guards against drift from newly installed version |
| 3 | [`.version.list`](#command-6-versionlist) | Lists version aliases or release history before selecting a target |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 2 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |

---

**Category:** version
**Complexity:** 5
**API Requirement:** Write
**Idempotent:** Yes
**Risk Level:** High

---

### Command :: 5. `.version.guard`

Check for version drift and restore the preferred version if it was changed. Operates in one-shot mode by default. Pass `interval::N` for watch mode that checks every N seconds until interrupted. In watch mode, transient install errors (e.g. `ETXTBSY`) are logged to stderr and do not terminate the loop; one-shot mode still propagates errors normally.

-- **Parameters:** version::, dry::, force::, interval::, v::, format::
-- **Exit Codes:** 0 (success/restored) | 2 (runtime error)
-- **Modes:** one-shot, watch

**Syntax:**

```sh
clv.version.guard [version::SPEC] [dry::1] [force::1] [interval::N] [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`version::`](../param/01_version.md) | [`VersionSpec`](../type/03_version_spec.md) | *(stored preference)* | No | Override preferred version for this invocation only |
| [`dry::`](../param/02_dry.md) | bool | false | No | Preview without side effects |
| [`force::`](../param/03_force.md) | bool | false | No | Reinstall even if version matches |
| [`interval::`](../param/08_interval.md) | u64 | 0 | No | Seconds between checks; 0 = one-shot |
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Output detail level |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |

**Algorithm (6 steps):**
1. Read stored preferred version from settings (or apply `version::` override for this run only).
2. Invoke `claude --version` to detect the currently installed version.
3. Compare installed vs. preferred; skip restore if equal and `force::0`.
4. If drift detected (or `force::1`): invoke `.version.install` logic for the preferred version.
5. Verify post-install version matches preferred; report restore result.
6. In watch mode (`interval::N>0`): sleep N seconds, loop back to step 2; log transient errors to stderr without terminating.

**Watch Mode Log Format:**

In watch mode, each check emits one line to stderr:

```
{date} · {time} · ok · {detail} · next check in {duration}
{date} · {time} · error · {message} · next check in {duration}
```

| Field | Meaning |
|-------|---------|
| `{date}` | `YYYY-MM-DD`, UTC |
| `{time}` | `HH:MM:SS`, UTC |
| `ok` / `error` | Outcome of this check |
| `{detail}` | Check result text (e.g. `version 2.1.197 matches preferred v2.1.197`); omitted when the result is the bare terse `ok` (`v::0`) |
| `{duration}` | The `interval::` value formatted as `Ns` or `Nm` |

Example:

```
2026-07-05 · 16:58:29 · ok · version 2.1.197 matches preferred v2.1.197 · next check in 30s
```

This compact format applies to `format::text` (the default). Under `format::json`, watch mode instead prints each iteration's check result verbatim as one JSON line, without the dot-separated wrapper — e.g. `{"status":"ok","installed":"2.1.197","preferred":"v2.1.197"}` — so JSON consumers get parseable output rather than a JSON blob embedded inside `{detail}`.

**Examples:**

```sh
# One-shot: check and restore if drifted
clv.version.guard

# Dry-run preview
clv.version.guard dry::1

# Override preference for this run only (no settings.json change)
clv.version.guard version::stable dry::1

# Watch mode: check every 60 seconds
clv.version.guard interval::60

# Force reinstall regardless of drift
clv.version.guard force::1
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
| 2 | [Execution Control](../param_group/02_execution_control.md) | Full | — |

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 1 | [`version::`](../param/01_version.md) |
| 2 | [`dry::`](../param/02_dry.md) |
| 3 | [`force::`](../param/03_force.md) |
| 4 | [`interval::`](../param/08_interval.md) |
| 5 | [`v::`](../param/04_v.md) |
| 6 | [`format::`](../param/05_format.md) |

### Referenced Command Group

Evaluated against `.version.install` (see step 4 above: "invoke `.version.install` logic for the preferred version") under the strict [command_group](../command_group/readme.md) identity test — does not qualify. `version_guard_routine()` (`src/commands/version.rs:240`) shares no routine with `version_install_routine()` (`src/commands/version.rs:75`); both call `perform_install()`/`validate_version_spec()` from the separate `claude_version_core` crate, which is external-library sharing, not one routine invoking the other. `.version.guard` also adds `interval::` (watch mode) with no `.version.install` equivalent. See [`command_group/readme.md`](../command_group/readme.md) Evaluated, Not Qualifying for the full analysis.

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.version.install`](#command-4-versioninstall) | Performs the install step when drift detected |
| 2 | [`.version.show`](#command-3-versionshow) | Verifies version after restoration |
| 3 | [`.version.list`](#command-6-versionlist) | Lists aliases that guard can target |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 2 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |

---

**Category:** version
**Complexity:** 6
**API Requirement:** Read
**Idempotent:** Yes
**Risk Level:** High

---

### Command :: 6. `.version.list`

List available version information: named version aliases with their pinned values (`mode::aliases`, default), or recent Claude Code release history from the GitHub Releases API (`mode::history`). Alias listing is a compile-time constant lookup — no network. History listing fetches from `anthropics/claude-code` releases (response cached locally for 1 hour). Use history mode to see what changed across recent versions, find when a specific fix landed, or review the full changelog for any release.

-- **Parameters:** mode::, count::, v::, format::
-- **Exit Codes:** 0 (success, both modes — `mode::history` falls back to a compiled-in offline snapshot with a stderr advisory when the live fetch and cache both fail) | 2 (`mode::history`: HOME unset)

**Syntax:**

```sh
clv.version.list [mode::MODE] [count::N] [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`mode::`](../param/14_mode.md) | [`ListMode`](../type/10_list_mode.md) | aliases | No | Select alias listing (local) or release-history listing (network) |
| [`count::`](../param/09_count.md) | u64 | 10 | No | Number of recent releases to show (`mode::history` only; ignored under `mode::aliases`) |
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Output detail level |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |

**Algorithm, `mode::aliases` (2 steps):**
1. Load the compile-time version alias table (`stable`, `month`, `latest` → pinned semver values).
2. Render the alias-to-version mapping in the requested format.

**Algorithm, `mode::history` (3 steps):**
1. Check local 1-hour cache for GitHub Releases API response; fetch from `anthropics/claude-code` releases endpoint if stale or absent. If both the cache and the live fetch fail, fall back to a compiled-in `VERSION_HISTORY` snapshot (versions 2.1.74-2.1.220) and print a stderr advisory — HOME being unset is the only condition that still exits non-zero.
2. Select the `count::N` most recent releases from the response payload.
3. Render each release (tag, date, changelog summary) in the requested format.

**Examples:**

```sh
# Default: alias listing
clv.version.list
clv.version.list format::json

# Release history
clv.version.list mode::history
clv.version.list mode::history count::3
clv.version.list mode::history v::0
clv.version.list mode::history count::1 v::2
clv.version.list mode::history format::json count::5
```

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [text](../format/01_text.md) | Default human-readable output |
| 2 | [json](../format/02_json.md) | Machine-readable structured output |

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|-----------|----------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Full | — |

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 1 | [`mode::`](../param/14_mode.md) |
| 2 | [`count::`](../param/09_count.md) |
| 3 | [`v::`](../param/04_v.md) |
| 4 | [`format::`](../param/05_format.md) |

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.version.show`](#command-3-versionshow) | Shows which alias is currently installed |
| 2 | [`.version.install`](#command-4-versioninstall) | Installs one of the listed aliases or a version found in history |
| 3 | [`.version.guard`](#command-5-versionguard) | Guards against drift from a listed alias |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 2 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |

---

**Category:** version
**Complexity:** 4
**API Requirement:** Read
**Idempotent:** Yes
**Risk Level:** Low

---

### Command :: 12. `.version.history` (retired)

**Retired** — merged into [`.version.list`](#command-6-versionlist) as `mode::history`. All release-history behavior, parameters, and examples now live under Command 6. This entry is preserved only to keep the global command numbering stable; do not implement or reference `.version.history` as a standalone command.

---

### Command :: 16. `.version.paths`

Report filesystem paths clv reads from or writes to: settings files, the versions directory, the binary symlink, and internal caches. Read-only — does not create, modify, or delete any file. Complements `.runtime_files` (unlabeled, pipeline-only, reports only the version-history-cache path) by adding labels and descriptions, plus the versions directory, binary symlink, and settings paths that `.runtime_files` does not report.

The operating mode is determined by whether `key::` is provided:

| Mode | Parameters | Behavior |
|------|------------|----------|
| show-all | (none) | All known paths, one per line, labeled |
| single | `key::K` | One resolved path for the given key |

-- **Parameters:** key::, format::, v::
-- **Exit Codes:** 0 (success) | 1 (invalid `key::` value) | 2 (HOME unset)

**Syntax:**

```sh
clv.version.paths [key::K] [format::FMT] [v::N]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`key::`](../param/06_key.md) | [`PathKey`](../type/09_path_key.md) | — | No | Specific path key for single-path mode |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Detail level: 0=plain paths only; 1=labeled (show-all mode only — single-key mode stays plain until v::2); 2=labeled+description |

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
2. Text format: at v::0, drop any path that did not resolve (e.g., no project config found for `project_settings`); at v::1/v::2, keep it with a "(none found)" placeholder. JSON format: always includes all 5 keys, using `null` for any unresolved path, regardless of `v::`.
3. Render the path table in requested format and verbosity.

**Algorithm (single-path, 3 steps):**
1. Validate `key::K` against the `PathKey` enum; exit 1 if unrecognized.
2. Resolve the requested path via `ClaudeVersionPaths`.
3. Render the single path (or placeholder, per verbosity) in requested format.

**Examples:**

```sh
# Show all known clv-managed paths
clv.version.paths

# Single path for scripting
clv.version.paths key::versions_dir v::0

# Machine-readable output
clv.version.paths format::json
clv.version.paths key::settings format::json

# Verbose output with descriptions
clv.version.paths v::2
```

**Sample text output (v::1, `clv.version.paths`):**

```
settings:               /home/user/.claude/settings.json
project_settings:       (none found)
versions_dir:           /home/user/.local/share/claude/versions
binary_symlink:         /home/user/.local/bin/claude
version_history_cache:  /home/user/.claude/.transient/version_history_cache.json
```

**Sample text output (v::0, `clv.version.paths key::versions_dir`):**

```
/home/user/.local/share/claude/versions
```

**Sample text output (v::2, `clv.version.paths key::binary_symlink`):**

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

### Referenced Command Group

Evaluated against `.runtime_files` under the strict [command_group](../command_group/readme.md) identity test (same routine function, same parameter set) — does not qualify. `paths_routine()` and `runtime_files_routine()` are separate functions with no call between them; `.version.paths` accepts 3 parameters (`key::, format::, v::`) while `.runtime_files` accepts none, so the parameter sets are not the same set differing only by default. See [`command_group/readme.md`](../command_group/readme.md) Evaluated, Not Qualifying for the full analysis.

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.runtime_files`](root.md#command-15-runtime_files) | Unlabeled, pipeline-only; reports only the version-history-cache path (a subset of `.version.paths`'s 5) |

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
