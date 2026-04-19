# Parameters

### All Parameters (10 total)

| # | Parameter | Type | Default | Valid Values | Description | Used In |
|---|-----------|------|---------|--------------|-------------|---------|
| 1 | `version::` | [`VersionSpec`](types.md#type--3-versionspec) | stable | stable, latest, month, semver | Version to install or guard | 2 cmds |
| 2 | `dry::` | bool | false | 0, 1 | Preview without executing | 4 cmds |
| 3 | `force::` | bool | false | 0, 1 | Bypass guards | 3 cmds |
| 4 | `v::` | [`VerbosityLevel`](types.md#type--1-verbositylevel) | 1 | 0 to 2 | Output detail level | 10 cmds |
| 5 | `format::` | [`OutputFormat`](types.md#type--2-outputformat) | text | text, json | Output format | 10 cmds |
| 6 | `key::` | [`SettingsKey`](types.md#type--4-settingskey) | **(required)** | Any text | Settings key | 2 cmds |
| 7 | `value::` | [`SettingsValue`](types.md#type--5-settingsvalue) | **(required)** | Any text | Settings value (type-inferred) | 1 cmd |
| 8 | `interval::` | u64 | 0 | Non-negative integer | Guard check interval (seconds) | 1 cmd |
| 9 | `count::` | u64 | 10 | Non-negative integer | Number of history entries to show | 1 cmd |
| 10 | `.help` | bool | false | present/absent | Show help and exit | 12 cmds |

**Groups:** Parameters 4–5 form [Output Control](parameter_groups.md#group--1-output-control). Parameters 2–3 form [Execution Control](parameter_groups.md#group--2-execution-control). Parameters 6–7 form [Settings Identity](parameter_groups.md#group--3-settings-identity).

---

### Parameter :: 1. `version::`

Specify which Claude Code version to install or guard against. Accepts named aliases
(`stable`, `latest`, `month`) or semver strings (e.g., `1.2.3`).
On `.version.guard`, the value overrides the stored preference for a single invocation
without writing to `settings.json`.

- **Type:** [`VersionSpec`](types.md#type--3-versionspec)
- **Default:** `stable`
- **Commands:** [`.version.install`](commands.md#command--4-version-install), [`.version.guard`](commands.md#command--5-version-guard)
- **Validation:** rejects 4-part semver (e.g., `1.2.3.4`), leading zeros (e.g., `01.02.03`), empty value

```sh
cm .version.install version::stable
cm .version.install version::1.2.3
cm .version.guard version::stable dry::1
cm .version.guard version::month
```

---

### Parameter :: 2. `dry::`

Preview the action that would be performed without executing side effects.
Output prefixed with `[dry-run] would ...`.

- **Type:** bool
- **Default:** false (0)
- **Commands:** [`.version.install`](commands.md#command--4-version-install), [`.version.guard`](commands.md#command--5-version-guard), [`.processes.kill`](commands.md#command--8-processes-kill), [`.settings.set`](commands.md#command--11-settings-set)
- **Group:** [Execution Control](parameter_groups.md#group--2-execution-control)
- **Validation:** strictly `0` or `1`; `true`, `yes`, `TRUE` etc. rejected with exit 1

```sh
cm .version.install dry::1
cm .version.guard dry::1
cm .processes.kill dry::1
cm .settings.set key::theme value::dark dry::1
```

---

### Parameter :: 3. `force::`

Bypass safety guards. For `.version.install`/`.version.guard`: skip "already installed"
idempotency check. For `.processes.kill`: SIGKILL directly (no SIGTERM).

- **Type:** bool
- **Default:** false (0)
- **Commands:** [`.version.install`](commands.md#command--4-version-install), [`.version.guard`](commands.md#command--5-version-guard), [`.processes.kill`](commands.md#command--8-processes-kill)
- **Group:** [Execution Control](parameter_groups.md#group--2-execution-control)
- **Validation:** strictly `0` or `1`; `true`, `yes`, `TRUE` etc. rejected with exit 1

```sh
cm .version.install force::1          # reinstall even if current
cm .version.guard force::1            # reinstall even if matching
cm .processes.kill force::1            # SIGKILL immediately
```

---

### Parameter :: 4. `v::`

Control output detail level.

- **Type:** [`VerbosityLevel`](types.md#type--1-verbositylevel)
- **Default:** 1 (normal)
- **Commands:** [`.status`](commands.md#command--2-status), [`.version.show`](commands.md#command--3-version-show), [`.version.install`](commands.md#command--4-version-install), [`.version.list`](commands.md#command--6-version-list), [`.version.guard`](commands.md#command--5-version-guard), [`.version.history`](commands.md#command--12-version-history), [`.processes`](commands.md#command--7-processes), [`.processes.kill`](commands.md#command--8-processes-kill), [`.settings.show`](commands.md#command--9-settings-show), [`.settings.get`](commands.md#command--10-settings-get)
- **Group:** [Output Control](parameter_groups.md#group--1-output-control)
- **Validation:** must be 0, 1, or 2; out of range -> exit 1
- **Last-wins:** when repeated, the last value takes effect

```sh
cm .status v::0                       # minimal
cm .status v::2                       # verbose
```

---

### Parameter :: 5. `format::`

Select output format. Case-sensitive: `text` and `json` only.

- **Type:** [`OutputFormat`](types.md#type--2-outputformat)
- **Default:** text
- **Commands:** [`.status`](commands.md#command--2-status), [`.version.show`](commands.md#command--3-version-show), [`.version.install`](commands.md#command--4-version-install), [`.version.list`](commands.md#command--6-version-list), [`.version.guard`](commands.md#command--5-version-guard), [`.version.history`](commands.md#command--12-version-history), [`.processes`](commands.md#command--7-processes), [`.processes.kill`](commands.md#command--8-processes-kill), [`.settings.show`](commands.md#command--9-settings-show), [`.settings.get`](commands.md#command--10-settings-get)
- **Group:** [Output Control](parameter_groups.md#group--1-output-control)
- **Validation:** `text` or `json` only; `TEXT`, `Json` etc. -> exit 1

```sh
cm .status format::json
cm .settings.show format::text
```

---

### Parameter :: 6. `key::`

Identify the settings entry to read or write. Required for `.settings.get`
and `.settings.set`.

- **Type:** [`SettingsKey`](types.md#type--4-settingskey)
- **Default:** **(required)**
- **Commands:** [`.settings.get`](commands.md#command--10-settings-get), [`.settings.set`](commands.md#command--11-settings-set)
- **Group:** [Settings Identity](parameter_groups.md#group--3-settings-identity)
- **Validation:** must not be empty; `key::` (empty) -> exit 1

```sh
cm .settings.get key::theme
cm .settings.set key::theme value::dark
```

---

### Parameter :: 7. `value::`

The value to write. Type-inferred: `"true"`/`"false"` -> JSON bool,
integer/float -> JSON number, otherwise -> JSON string. Required for
`.settings.set`.

- **Type:** [`SettingsValue`](types.md#type--5-settingsvalue)
- **Default:** **(required)**
- **Command:** [`.settings.set`](commands.md#command--11-settings-set)
- **Group:** [Settings Identity](parameter_groups.md#group--3-settings-identity)
- **Validation:** must not be empty; `value::` (empty) -> exit 1

```sh
cm .settings.set key::theme value::dark      # -> "dark" (string)
cm .settings.set key::timeout value::30      # -> 30 (number)
cm .settings.set key::autoUpdate value::true  # -> true (bool)
```

---

### Parameter :: 8. `interval::`

Controls the check frequency for `.version.guard`. When `0` (default), the
guard runs once and exits. When `N > 0`, the guard loops every `N` seconds
until interrupted.

- **Type:** u64 (unsigned integer, seconds)
- **Default:** 0 (one-shot)
- **Command:** [`.version.guard`](commands.md#command--5-version-guard)
- **Validation:** must be a non-negative integer

```sh
cm .version.guard interval::0      # one-shot (default)
cm .version.guard interval::60     # check every 60 seconds
cm .version.guard interval::3600   # check every hour
```

---

### Parameter :: 9. `count::`

Limit the number of releases shown by `.version.history`. Default is 10,
showing the most recent releases first.

- **Type:** u64 (unsigned integer)
- **Default:** 10
- **Command:** [`.version.history`](commands.md#command--12-version-history)
- **Group:** [Output Control](parameter_groups.md#group--1-output-control)
- **Validation:** must be a non-negative integer; values exceeding available releases return all available

```sh
cm .version.history count::1       # most recent release only
cm .version.history count::3       # 3 most recent releases
cm .version.history count::0       # empty output (valid, exit 0)
```

---

### Parameter :: 10. `.help`

Display help listing and exit. Overrides any command or other parameters when
present anywhere in argv.

- **Type:** bool (standalone)
- **Default:** false
- **Commands:** all 12 commands (universal override)

```sh
cm .help
cm .version.install .help    # still shows help, ignores install
```

### Quick Reference

**Required parameters:** `key::` (`.settings.get`/`.settings.set`), `value::` (`.settings.set`).

**Most used parameters:** `v::` (10 commands), `format::` (10 commands), `dry::` (4 commands), `force::` (3 commands), `key::` (2 commands).

**Commands by parameter count:** 0 params = 1 command (`.help`), 2 params = 5 commands, 3 params = 3 commands, 4 params = 1 command (`.processes.kill`), 5 params = 1 command (`.version.install`), 6 params = 1 command (`.version.guard`).
