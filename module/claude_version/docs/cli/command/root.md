# root — Root Namespace Commands

### Scope

- **Purpose**: Reference for root-namespace clv commands.
- **Responsibility**: Command syntax, parameters, exit codes, and cross-references for `.help`, `.status`, and `.runtime_files`.
- **In Scope**: `.help`, `.status`, `.runtime_files`.
- **Out of Scope**: Version commands (→ [version.md](version.md)), process commands (→ [processes.md](processes.md)), settings commands (→ [settings.md](settings.md)).

---

### Command :: 1. `.help`

Display the full command listing, all parameters, and usage examples. Triggered by the `.help` command, empty argv, or `.help` appearing anywhere in argv. Overrides any other command or parameters when present.

-- **Exit Codes:** 0 (always)

**Syntax:**

```sh
clv.help
clv             # empty argv also shows help
clv.            # bare dot is a help alias
clv.status .help  # .help anywhere in argv triggers help (FR-02)
```

**Parameters:** none

**Algorithm (3 steps):**
1. Scan argv for `.help` anywhere, or detect empty argv or bare `.` as the sole token.
2. If any trigger is present, bypass the unilang pipeline entirely (intercept before parsing).
3. Render grouped command listing (4 categories: version management, settings & config, process lifecycle, status), shared parameters, and usage examples via `cli_fmt::CliHelpTemplate` to stdout and exit 0.

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.status`](#command-2-status) | First operational check after reviewing help |
| 2 | [`.version.show`](version.md#command-3-versionshow) | Confirms installed version |
| 3 | [`.processes`](processes.md#command-7-processes) | Lists running sessions |
| 4 | [`.settings.show`](settings.md#command-9-settingsshow) | Inspects current settings |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [001 Environment Check](../user_story/001_environment_check.md) | Developer (new machine) |
| 2 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 3 | [003 Process Lifecycle](../user_story/003_process_lifecycle.md) | Developer (unresponsive session) |
| 4 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
| 5 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |
| 6 | [006 Config Management](../user_story/006_config_management.md) | Developer (config management) |

---

**Category:** help
**Complexity:** 0
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low

---

### Command :: 2. `.status`

Aggregates the three most important operational signals — installed version, running session count, and active account — into a single composited view, plus the preferred version spec when one is stored. At `v::2`+ (text) or any `format::json`, also shows a `Lock:` section comparing the 5 pin-related settings keys plus the versions directory's `chmod` mode against what the current pin state implies, flagging any drifted key as `MISMATCH`. If `settings.json` exists but could not be read — malformed JSON, a permission error, or any other I/O error short of the file simply not existing — every row instead reports `UNVERIFIABLE`: an unreadable settings file makes the pin state itself unreliable, so a real MISMATCH/Compliant verdict cannot be asserted against it. Use this as the first check when diagnosing a Claude Code environment.

-- **Parameters:** v::, format::
-- **Exit Codes:** 0 (always — read-only, never fails; see Degradation Semantics in `tests/docs/cli/command/02_status.md`) | 1 (invalid `v::`/`format::` value, or unknown parameter)

**Syntax:**

```sh
clv.status [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Output detail level |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |

**Algorithm (5 steps):**
1. Invoke `claude --version` to detect the currently installed binary version (reports "not found" rather than failing if absent).
2. Scan `/proc/*/cmdline` for running Claude Code processes and count matches.
3. Read the active account and, if stored, the preferred version spec (`preferredVersionSpec`) — shown as a `Preferred:` line only when set.
4. At `v::2`+ (text) or any `format::json`: resolve `autoUpdates`, `autoUpdatesChannel`, `minimumVersion`, `env.DISABLE_AUTOUPDATER`, `env.DISABLE_UPDATES`, and the versions directory's `chmod` mode; compare each against what the current pin state implies and render a `Lock:` section (`"lock"` object in JSON), flagging any drifted key as `MISMATCH`. A `chmod` value of `absent` (versions directory not yet created — e.g. a fresh install) is never flagged as a mismatch, since there is no reliable drift signal to compare. If `settings.json` exists but could not be read (malformed JSON, a permission error, or any other I/O error besides the file simply not existing), all 6 rows report `UNVERIFIABLE` instead — an unreadable settings file makes the pin state itself unreliable, so a MISMATCH/Compliant verdict cannot be trusted either way. Read-only — never mutates settings or file permissions.
5. Render aggregated status view (version, process count, account, optional preferred-version line, Lock: section) in the requested format; exits 1 first if `v::`/`format::` is out of range or an unknown parameter is present.

**Examples:**

```sh
clv.status
clv.status format::json
clv.status v::2
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
| 1 | [`.help`](#command-1-help) | Command reference and usage listing |
| 2 | [`.version.show`](version.md#command-3-versionshow) | Version-only view without full status |
| 3 | [`.processes`](processes.md#command-7-processes) | Process-only view |
| 4 | [`.settings.show`](settings.md#command-9-settingsshow) | Settings-only view |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [001 Environment Check](../user_story/001_environment_check.md) | Developer (new machine) |

---

**Category:** status
**Complexity:** 2
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low

---

### Command :: 15. `.runtime_files`

Enumerate all on-disk paths managed by clv for the current configuration. Output represents paths that WILL exist after relevant commands are run — not only paths currently on disk. Suitable for operator tooling, health checks, and cleanup scripts.

-- **Exit Codes:** 0 (success) | 2 (HOME unset or I/O error)

**Syntax:**

```sh
clv.runtime_files
```

**Parameters:** none

**Algorithm (3 steps):**
1. Read `$HOME` from the environment; exit 2 if absent.
2. Compute all runtime file paths for the current configuration: `$HOME/.claude/.transient/version_history_cache.json`.
3. Print each path as an absolute path, one per line, to stdout; exit 0.

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.version.history`](version.md#command-12-versionhistory) | Creates the version history cache path enumerated here |

---

**Category:** discovery
**Complexity:** 0
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low
