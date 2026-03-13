# Commands

### All Commands (12 total)

| # | Command | Description | Params | Example |
|---|---------|-------------|--------|---------|
| 1 | `.help` | Display command reference and usage | 0 | `cm .help` |
| 2 | `.status` | Show installation state, session count, and preferred version | 2 | `cm .status format::json` |
| 3 | `.version.show` | Print installed Claude Code version | 2 | `cm .version.show` |
| 4 | `.version.install` | Install a Claude Code version via official installer | 5 | `cm .version.install version::stable` |
| 5 | `.version.guard` | Check for version drift and restore preferred version | 5 | `cm .version.guard` |
| 6 | `.version.list` | List named version aliases with pinned values | 2 | `cm .version.list` |
| 7 | `.processes` | List running Claude Code processes | 2 | `cm .processes` |
| 8 | `.processes.kill` | Terminate all Claude Code processes | 2 | `cm .processes.kill dry::1` |
| 9 | `.settings.show` | Print all settings | 2 | `cm .settings.show format::json` |
| 10 | `.settings.get` | Read a single setting by key | 3 | `cm .settings.get key::theme` |
| 11 | `.settings.set` | Write a single setting atomically | 3 | `cm .settings.set key::theme value::dark` |
| 12 | `.version.history` | Show release history with changelogs from GitHub | 3 | `cm .version.history count::5` |

---

### Command :: 1. `.help`

Display the full command listing, all parameters, and usage examples. Triggered
by the `.help` command, empty argv, or `.help` appearing anywhere in argv.

**Syntax:**

```sh
cm .help
cm            # empty argv also shows help
cm .          # bare dot is a help alias
cm .status .help   # .help anywhere in argv triggers help (FR-02)
```

**Parameters:** none

**Exit Codes:** 0 (always)

---

### Command :: 2. `.status`

Aggregates the three most important operational signals -- installed version,
running session count, and active account -- into a single composited view.
Use this as the first check when diagnosing a Claude Code environment.

**Syntax:**

```sh
cm .status [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`v::`](params.md#parameter--4-v) | [`VerbosityLevel`](types.md#type--1-verbositylevel) | 1 | Output detail |
| [`format::`](params.md#parameter--5-format) | [`OutputFormat`](types.md#type--2-outputformat) | text | Output format |

**Group:** [Output Control](parameter_groups.md#group--1-output-control)

**Exit Codes:** 0 (success), 2 (runtime error -- HOME missing or settings unreadable)

**Examples:**

```sh
cm .status
cm .status format::json
cm .status v::2
```

---

### Command :: 3. `.version.show`

Print the currently installed Claude Code version by querying
`claude --version`. Use this to verify what is currently installed
before upgrading or troubleshooting.

**Syntax:**

```sh
cm .version.show [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`v::`](params.md#parameter--4-v) | [`VerbosityLevel`](types.md#type--1-verbositylevel) | 1 | Output detail |
| [`format::`](params.md#parameter--5-format) | [`OutputFormat`](types.md#type--2-outputformat) | text | Output format |

**Group:** [Output Control](parameter_groups.md#group--1-output-control)

**Exit Codes:** 0 (success), 2 (runtime error -- binary not found)

---

### Command :: 4. `.version.install`

Download and install a Claude Code version via the official installer (curl).
Supports hot-swap and 5-layer version locking (Layers 1–4 prevent auto-updates; Layer 5 stores the preferred version as a recovery signal for `.version.guard`).
Accepts named aliases (`stable`, `latest`, `month`) and semver strings.

**Syntax:**

```sh
cm .version.install [version::VER] [dry::1] [force::1] [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`version::`](params.md#parameter--1-version) | [`VersionSpec`](types.md#type--3-versionspec) | stable | Version to install |
| [`dry::`](params.md#parameter--2-dry) | bool | false | Preview install command |
| [`force::`](params.md#parameter--3-force) | bool | false | Bypass idempotency check |
| [`v::`](params.md#parameter--4-v) | [`VerbosityLevel`](types.md#type--1-verbositylevel) | 1 | Output detail |
| [`format::`](params.md#parameter--5-format) | [`OutputFormat`](types.md#type--2-outputformat) | text | Output format |

**Groups:** [Execution Control](parameter_groups.md#group--2-execution-control), [Output Control](parameter_groups.md#group--1-output-control)

**Exit Codes:** 0 (success), 1 (invalid version spec), 2 (installer failure)

**Examples:**

```sh
# Install the pinned stable version (default)
cm .version.install

# Dry-run shows all 5 lock layers
cm .version.install version::stable dry::1

# Idempotent skip: already at target, stores preference and exits 0
cm .version.install version::stable

# Force reinstall even if already at target version
cm .version.install force::1

# Install latest (no version pin -- resolves dynamically)
cm .version.install version::latest
```

---

### Command :: 5. `.version.guard`

Check for version drift and restore the preferred version if it was changed.
Operates in one-shot mode by default. Pass `interval::N` for watch mode
that checks every N seconds until interrupted. In watch mode, transient
install errors (e.g. `ETXTBSY`) are logged to stderr and do not terminate
the loop; one-shot mode still propagates errors normally.

**Syntax:**

```sh
cm .version.guard [version::SPEC] [dry::1] [force::1] [interval::N] [v::N]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`version::`](params.md#parameter--1-version) | [`VersionSpec`](types.md#type--3-versionspec) | *(stored preference)* | Override preferred version for this invocation only |
| [`dry::`](params.md#parameter--2-dry) | bool | false | Preview without side effects |
| [`force::`](params.md#parameter--3-force) | bool | false | Reinstall even if version matches |
| [`interval::`](params.md#parameter--9-interval) | u64 | 0 | Seconds between checks; 0 = one-shot |
| [`v::`](params.md#parameter--4-v) | [`VerbosityLevel`](types.md#type--1-verbositylevel) | 1 | Output detail |

**Exit Codes:** 0 (success/restored), 2 (runtime error -- install failed or HOME unset)

**Examples:**

```sh
# One-shot: check and restore if drifted
cm .version.guard

# Dry-run preview
cm .version.guard dry::1

# Override preference for this run only (no settings.json change)
cm .version.guard version::stable dry::1

# Watch mode: check every 60 seconds
cm .version.guard interval::60

# Force reinstall regardless of drift
cm .version.guard force::1
```

---

### Command :: 6. `.version.list`

List all named version aliases (`stable`, `month`, `latest`) with their
currently pinned values. These are compile-time constants; they do not
query the network.

**Syntax:**

```sh
cm .version.list [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`v::`](params.md#parameter--4-v) | [`VerbosityLevel`](types.md#type--1-verbositylevel) | 1 | Output detail |
| [`format::`](params.md#parameter--5-format) | [`OutputFormat`](types.md#type--2-outputformat) | text | Output format |

**Group:** [Output Control](parameter_groups.md#group--1-output-control)

**Exit Codes:** 0 (always)

---

### Command :: 7. `.processes`

List all running Claude Code processes detected via `/proc` scanning.
Reports PIDs and working directories.

**Syntax:**

```sh
cm .processes [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`v::`](params.md#parameter--4-v) | [`VerbosityLevel`](types.md#type--1-verbositylevel) | 1 | Output detail |
| [`format::`](params.md#parameter--5-format) | [`OutputFormat`](types.md#type--2-outputformat) | text | Output format |

**Group:** [Output Control](parameter_groups.md#group--1-output-control)

**Exit Codes:** 0 (always, even if 0 processes found)

---

### Command :: 8. `.processes.kill`

Terminate all running Claude Code processes. Normal mode: SIGTERM, wait
2 seconds, then SIGKILL survivors. Force mode: SIGKILL directly. Performs
500ms post-kill verification to confirm termination.

**Syntax:**

```sh
cm .processes.kill [dry::1] [force::1]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`dry::`](params.md#parameter--2-dry) | bool | false | Preview which processes would be killed |
| [`force::`](params.md#parameter--3-force) | bool | false | SIGKILL directly (skip SIGTERM) |

**Group:** [Execution Control](parameter_groups.md#group--2-execution-control)

**Exit Codes:** 0 (success), 2 (signal delivery failed)

**Examples:**

```sh
cm .processes.kill dry::1    # preview
cm .processes.kill           # SIGTERM -> 2s -> SIGKILL
cm .processes.kill force::1  # SIGKILL immediately
```

---

### Command :: 9. `.settings.show`

Print all key-value pairs from `~/.claude/settings.json`.

**Syntax:**

```sh
cm .settings.show [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`v::`](params.md#parameter--4-v) | [`VerbosityLevel`](types.md#type--1-verbositylevel) | 1 | Output detail |
| [`format::`](params.md#parameter--5-format) | [`OutputFormat`](types.md#type--2-outputformat) | text | Output format |

**Group:** [Output Control](parameter_groups.md#group--1-output-control)

**Exit Codes:** 0 (success), 2 (file unreadable or malformed JSON)

---

### Command :: 10. `.settings.get`

Read a single setting from `~/.claude/settings.json` by key.

**Syntax:**

```sh
cm .settings.get key::<KEY> [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`key::`](params.md#parameter--6-key) | [`SettingsKey`](types.md#type--4-settingskey) | **(required)** | Setting to read |
| [`v::`](params.md#parameter--4-v) | [`VerbosityLevel`](types.md#type--1-verbositylevel) | 1 | Output detail |
| [`format::`](params.md#parameter--5-format) | [`OutputFormat`](types.md#type--2-outputformat) | text | Output format |

**Groups:** [Settings Identity](parameter_groups.md#group--3-settings-identity), [Output Control](parameter_groups.md#group--1-output-control)

**Exit Codes:** 0 (success), 1 (missing `key::`), 2 (key not found or file error)

**Examples:**

```sh
cm .settings.get key::theme
cm .settings.get key::autoUpdate format::json
```

---

### Command :: 11. `.settings.set`

Write a single setting atomically to `~/.claude/settings.json`. The value
is type-inferred: `"true"`/`"false"` -> bool, integer/float -> number,
otherwise -> string. Creates the key if absent (upsert semantics).

**Syntax:**

```sh
cm .settings.set key::<KEY> value::<VALUE> [dry::1]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`key::`](params.md#parameter--6-key) | [`SettingsKey`](types.md#type--4-settingskey) | **(required)** | Setting to write |
| [`value::`](params.md#parameter--7-value) | [`SettingsValue`](types.md#type--5-settingsvalue) | **(required)** | Value to write |
| [`dry::`](params.md#parameter--2-dry) | bool | false | Preview without writing |

**Groups:** [Settings Identity](parameter_groups.md#group--3-settings-identity), [Execution Control](parameter_groups.md#group--2-execution-control)

**Exit Codes:** 0 (success), 1 (missing `key::` or `value::`), 2 (write failure)

**Examples:**

```sh
cm .settings.set key::theme value::dark
cm .settings.set key::timeout value::30       # stored as number
cm .settings.set key::autoUpdate value::true   # stored as bool
cm .settings.set key::theme value::dark dry::1
```

---

### Command :: 12. `.version.history`

Fetch and display recent Claude Code release history from the GitHub Releases
API (`anthropics/claude-code`). Use this to see what changed across recent
versions, find when a specific fix landed, or review the full changelog for
any release. Response is cached locally for 1 hour.

**Syntax:**

```sh
cm .version.history [count::N] [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`count::`](params.md#parameter--10-count) | u64 | 10 | Number of recent releases to show |
| [`v::`](params.md#parameter--4-v) | [`VerbosityLevel`](types.md#type--1-verbositylevel) | 1 | Output detail |
| [`format::`](params.md#parameter--5-format) | [`OutputFormat`](types.md#type--2-outputformat) | text | Output format |

**Group:** [Output Control](parameter_groups.md#group--1-output-control)

**Exit Codes:** 0 (success), 2 (network failure or HOME unset)

**Examples:**

```sh
# Default: 10 most recent releases with one-line summaries
cm .version.history

# Show 3 most recent releases
cm .version.history count::3

# Minimal output: version and date only
cm .version.history v::0

# Full changelog per release
cm .version.history count::1 v::2

# JSON format for scripting
cm .version.history format::json count::5
```
