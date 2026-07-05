# processes — Process Namespace Commands

### Scope

- **Purpose**: Reference for process-namespace clv commands.
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

**Algorithm (2 steps):**
1. Scan `/proc/*/cmdline` for entries where `basename(argv[0]) == "claude"`.
2. Render the list of matching PIDs and working directories in the requested format.

**Examples:**

```sh
clv.processes
clv.processes format::json
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
| 1 | [`.processes.kill`](#command-8-processeskill) | Terminates the listed processes |
| 2 | [`.status`](root.md#command-2-status) | Includes process count in broader environment snapshot |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [003 Process Lifecycle](../user_story/003_process_lifecycle.md) | Developer (unresponsive session) |

---

**Category:** process
**Complexity:** 2
**API Requirement:** Read
**Idempotent:** Yes
**Risk Level:** Low

---

### Command :: 8. `.processes.kill`

Terminate all running Claude Code processes. Normal mode: SIGTERM, wait 2 seconds, then SIGKILL survivors. Force mode (`force::1`): SIGKILL directly. Performs 500ms post-kill verification to confirm termination.

**Invocation invariant:** This command must be explicitly invoked by the user. It is never called automatically by `.version.guard`, `.version.install`, or any scheduled path. Automatic flows (guard, install, daemon watch) interact with running processes exclusively via `hot_swap_binary()` — not via kill signals.

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

**Algorithm (4 steps):**
1. Scan `/proc` for running Claude Code PIDs (same discovery as `.processes`).
2. Send SIGTERM to all discovered PIDs; wait 2 seconds for graceful exit (skip if `force::1`).
3. SIGKILL any processes still alive after the grace period (or SIGKILL all immediately if `force::1`).
4. Wait 500ms, then verify all target PIDs have exited from `/proc`; report termination result.

**Examples:**

```sh
clv.processes.kill dry::1    # preview without sending signals
clv.processes.kill           # SIGTERM -> 2s wait -> SIGKILL survivors
clv.processes.kill force::1  # SIGKILL immediately
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
| 1 | [`dry::`](../param/02_dry.md) |
| 2 | [`force::`](../param/03_force.md) |
| 3 | [`v::`](../param/04_v.md) |
| 4 | [`format::`](../param/05_format.md) |

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.processes`](#command-7-processes) | Lists processes before termination |
| 2 | [`.status`](root.md#command-2-status) | Confirms process count after kill |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [003 Process Lifecycle](../user_story/003_process_lifecycle.md) | Developer (unresponsive session) |

---

**Category:** process
**Complexity:** 4
**API Requirement:** Write
**Idempotent:** No
**Risk Level:** High
