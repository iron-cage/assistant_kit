# root — Root Namespace Commands

### Scope

- **Purpose**: Reference for root-namespace clv commands.
- **Responsibility**: Command syntax, parameters, exit codes, and cross-references for `.help` and `.status`.
- **In Scope**: `.help`, `.status`.
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
| 1 | [`.status`](#command--2-status) | First operational check after reviewing help |
| 2 | [`.version.show`](version.md#command--3-version-show) | Confirms installed version |
| 3 | [`.processes`](processes.md#command--7-processes) | Lists running sessions |
| 4 | [`.settings.show`](settings.md#command--9-settings-show) | Inspects current settings |

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

Aggregates the three most important operational signals — installed version, running session count, and active account — into a single composited view. Use this as the first check when diagnosing a Claude Code environment.

-- **Parameters:** v::, format::
-- **Exit Codes:** 0 (success) | 2 (runtime error)

**Syntax:**

```sh
clv.status [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Output detail level |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |

**Algorithm (3 steps):**
1. Invoke `claude --version` to detect the currently installed binary version.
2. Scan `/proc/*/cmdline` for running Claude Code processes and count matches.
3. Render aggregated status view (version, process count, active account) in the requested format.

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

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.help`](#command--1-help) | Command reference and usage listing |
| 2 | [`.version.show`](version.md#command--3-version-show) | Version-only view without full status |
| 3 | [`.processes`](processes.md#command--7-processes) | Process-only view |
| 4 | [`.settings.show`](settings.md#command--9-settings-show) | Settings-only view |

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
