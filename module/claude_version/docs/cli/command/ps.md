# ps — Process Namespace Commands

### Scope

- **Purpose**: Reference for process-namespace clv commands.
- **Responsibility**: Command syntax, parameters, exit codes, and cross-references for `.ps` and `.ps.kill`.
- **In Scope**: `.ps`, `.ps.kill`.
- **Out of Scope**: Version commands (→ [version.md](version.md)), settings commands (→ [settings.md](settings.md)).

---

### Command :: 7. `.ps`

List all running Claude Code processes detected via `/proc` scanning. Reports a rich table with PID, elapsed time, CPU%, RAM, state, mode, working directory, and active task. Returns exit 0 even if no processes are found (empty list is a valid result).

-- **Parameters:** v::, format::
-- **Exit Codes:** 0 (always)

**Syntax:**

```sh
clv.ps [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Output detail level |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |

**Algorithm (2 steps):**
1. Scan `/proc/*/cmdline` for entries where `basename(argv[0]) == "claude"`.
2. Render the list of matching processes as a table (v::1) or bare PID list (v::0) in the requested format.

**Sample text output (v::1):**

```
#  PID     Elapsed  CPU%  RAM      State    Mode    Path                Task
1  287807  0:42     0.1   45.2 MB  running  normal  ~/pro/lib/yrd_…     implement ps rename
2  299134  2:17     0.0   38.7 MB  running  normal  ~/pro/lib/yrd_…     —
```

Columns: index, PID, elapsed wall-clock, CPU%, resident RAM, state (running/stopped), mode (normal/watch/dry), working directory (shortened), JSONL task preview.

**Examples:**

```sh
clv.ps
clv.ps format::json
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

### Referenced Command Group

Evaluated against `.ps.kill` (algorithm step 1 below notes "same discovery as `.ps`") under the strict [command_group](../command_group/readme.md) identity test — does not qualify. `ps_routine()` (`src/commands/process.rs`) shares no routine with `ps_kill_routine()` (`src/commands/process.rs`); both call `find_claude_processes()` from the separate `claude_runner_core` crate, which is external-library sharing (also used by `status_routine()`), not one routine invoking the other. `.ps.kill` also adds `dry::`/`force::`/`pid::` with no `.ps` equivalent. See [`command_group/readme.md`](../command_group/readme.md) Evaluated, Not Qualifying for the full analysis.

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.ps.kill`](#command-8-pskill) | Terminates the listed processes |
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

### Command :: 8. `.ps.kill`

Terminate Claude Code processes. Without `pid::`: terminates all running Claude Code processes. With `pid::PID`: terminates one specific process (after validating it is a Claude Code process). Normal mode: SIGTERM, wait 2 seconds, then SIGKILL survivors. Force mode (`force::1`): SIGKILL directly. Performs 500ms post-kill verification to confirm termination.

**Invocation invariant:** This command must be explicitly invoked by the user. It is never called automatically by `.version.guard`, `.version.install`, or any scheduled path. Automatic flows (guard, install, daemon watch) interact with running processes exclusively via `hot_swap_binary()` — not via kill signals.

-- **Parameters:** pid::, dry::, force::, v::, format::
-- **Exit Codes:** 0 (success) | 1 (invalid params, or `pid::` names a non-claude process) | 2 (signal delivery failed)

**Syntax:**

```sh
clv.ps.kill [pid::PID] [dry::1] [force::1] [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`pid::`](../param/17_pid.md) | u64 | — | No | Kill one specific process; validates it is a claude process |
| [`dry::`](../param/02_dry.md) | bool | false | No | Preview which processes would be killed |
| [`force::`](../param/03_force.md) | bool | false | No | SIGKILL directly, skipping SIGTERM |
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Output detail level |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |

**Algorithm — bulk kill (no `pid::`, 4 steps):**
1. Scan `/proc` for running Claude Code PIDs (same discovery as `.ps`).
2. Send SIGTERM to all discovered PIDs; wait 2 seconds for graceful exit (skip if `force::1`).
3. SIGKILL any processes still alive after the grace period (or SIGKILL all immediately if `force::1`).
4. Wait 500ms, then verify all target PIDs have exited from `/proc`; report termination result.

**Algorithm — targeted kill (`pid::PID`, 4 steps):**
1. Validate that PID is a running Claude Code process (scan `/proc`); exit 1 if not found or not a claude process.
2. Send SIGTERM to the target PID; wait 2 seconds for graceful exit (skip if `force::1`).
3. SIGKILL the target PID if still alive after the grace period (or SIGKILL immediately if `force::1`).
4. Wait 500ms, then verify the target PID has exited from `/proc`; report termination result.

**Examples:**

```sh
clv.ps.kill dry::1              # preview without sending signals
clv.ps.kill                     # SIGTERM → 2s wait → SIGKILL survivors (all)
clv.ps.kill pid::287807         # target one specific process
clv.ps.kill pid::287807 dry::1  # preview targeted kill
clv.ps.kill force::1            # SIGKILL immediately (all)
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
| 1 | [`pid::`](../param/17_pid.md) |
| 2 | [`dry::`](../param/02_dry.md) |
| 3 | [`force::`](../param/03_force.md) |
| 4 | [`v::`](../param/04_v.md) |
| 5 | [`format::`](../param/05_format.md) |

### Referenced Command Group

Evaluated against `.ps` (see step 1 above: "same discovery as `.ps`") under the strict [command_group](../command_group/readme.md) identity test — does not qualify. `ps_kill_routine()` (`src/commands/process.rs`) shares no routine with `ps_routine()` (`src/commands/process.rs`); the shared discovery call is to `find_claude_processes()` in the separate `claude_runner_core` crate, not a call from one routine into the other. `.ps.kill` also adds `dry::`/`force::`/`pid::` with no `.ps` equivalent, and the output shapes differ categorically (kill-confirmation vs. process listing). See [`command_group/readme.md`](../command_group/readme.md) Evaluated, Not Qualifying for the full analysis.

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.ps`](#command-7-ps) | Lists processes before termination |
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
