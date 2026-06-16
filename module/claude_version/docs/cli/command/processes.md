# processes — Process Namespace Commands

### Scope

- **Purpose**: Reference for process-namespace clvcommands.
- **Responsibility**: Command syntax, parameters, exit codes, and cross-references for `.processes` and `.processes.kill`.
- **In Scope**: `.processes`, `.processes.kill`.
- **Out of Scope**: Version commands (→ [version.md](version.md)), settings commands (→ [settings.md](settings.md)).

---

### Command :: 7. `.processes`

List all running Claude Code processes detected via `/proc` scanning. Reports PIDs and working directories. Returns exit 0 even if no processes are found (empty list is a valid result).

-- **Parameters:** v::, format::
-- **Exit Codes:** 0 (always)

**Syntax:**

```sh
clv.processes [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Output detail level |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |

**Examples:**

```sh
clv.processes
clv.processes format::json
```

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [text](../format/001_text.md) | Default human-readable output |
| 2 | [json](../format/002_json.md) | Machine-readable structured output |

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|-----------|----------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Partial | `count::` |

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.processes.kill`](#command--8-processes-kill) | Terminates the listed processes |
| 2 | [`.status`](root.md#command--2-status) | Includes process count in broader environment snapshot |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 3 | [003 Process Lifecycle](../user_story/003_process_lifecycle.md) | Developer (unresponsive session) |

---

**Category:** process
**Complexity:** 2
**API Requirement:** Read
**Idempotent:** Yes
**Risk Level:** Low

---

### Command :: 8. `.processes.kill`

Terminate all running Claude Code processes. Normal mode: SIGTERM, wait 2 seconds, then SIGKILL survivors. Force mode (`force::1`): SIGKILL directly. Performs 500ms post-kill verification to confirm termination.

-- **Parameters:** dry::, force::, v::, format::
-- **Exit Codes:** 0 (success) | 2 (signal delivery failed)

**Syntax:**

```sh
clv.processes.kill [dry::1] [force::1] [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`dry::`](../param/02_dry.md) | bool | false | No | Preview which processes would be killed |
| [`force::`](../param/03_force.md) | bool | false | No | SIGKILL directly, skipping SIGTERM |
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Output detail level |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |

**Examples:**

```sh
clv.processes.kill dry::1    # preview without sending signals
clv.processes.kill           # SIGTERM -> 2s wait -> SIGKILL survivors
clv.processes.kill force::1  # SIGKILL immediately
```

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [text](../format/001_text.md) | Default human-readable output |
| 2 | [json](../format/002_json.md) | Machine-readable structured output |

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|-----------|----------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Partial | `count::` |
| 2 | [Execution Control](../param_group/02_execution_control.md) | Full | — |

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.processes`](#command--7-processes) | Lists processes before termination |
| 2 | [`.status`](root.md#command--2-status) | Confirms process count after kill |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 3 | [003 Process Lifecycle](../user_story/003_process_lifecycle.md) | Developer (unresponsive session) |

---

**Category:** process
**Complexity:** 4
**API Requirement:** Write
**Idempotent:** No
**Risk Level:** High
