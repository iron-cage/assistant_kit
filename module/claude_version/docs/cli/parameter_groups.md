# Parameter Groups

### All Groups (3 total)

| # | Group | Parameters | Purpose |
|---|-------|------------|---------|
| 1 | Output Control | 3 | Control output appearance and volume |
| 2 | Execution Control | 2 | Control mutation behavior |
| 3 | Settings Identity | 2 | Identify settings target |

---

### Group :: 1. Output Control

Control how command output appears. Both parameters affect display without
changing behavior.

**Coherence test:** "Does this parameter control output appearance?" — YES for both.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| [`v::`](params.md#parameter--4-v) | [`VerbosityLevel`](types.md#type--1-verbositylevel) | Detail level (0=minimal, 1=normal, 2=verbose) |
| [`format::`](params.md#parameter--5-format) | [`OutputFormat`](types.md#type--2-outputformat) | Display encoding (text or json) |
| [`count::`](params.md#parameter--10-count) | u64 | Entry limit (`.version.history` only; default 10) |

**Used by:** [`.status`](commands.md#command--2-status), [`.version.show`](commands.md#command--3-version-show), [`.version.install`](commands.md#command--4-version-install), [`.version.list`](commands.md#command--6-version-list), [`.version.guard`](commands.md#command--5-version-guard), [`.version.history`](commands.md#command--12-version-history), [`.processes`](commands.md#command--7-processes), [`.processes.kill`](commands.md#command--8-processes-kill), [`.settings.show`](commands.md#command--9-settings-show), [`.settings.get`](commands.md#command--10-settings-get)

**Why NOT in this group:**
- `dry::`: controls execution, not display
- `key::`: identifies what to read, not how to display

**Typical usage:**

```sh
cm .status v::0 format::json
cm .processes format::json v::2
```

---

### Group :: 2. Execution Control

Control whether and how mutation commands execute. Both parameters modify
the execution mode of destructive operations.

**Coherence test:** "Does this parameter control mutation execution mode?" — YES for both.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| [`dry::`](params.md#parameter--2-dry) | bool | Preview without executing |
| [`force::`](params.md#parameter--3-force) | bool | Bypass safety guards |

**Used by:** [`.version.install`](commands.md#command--4-version-install) (both), [`.version.guard`](commands.md#command--5-version-guard) (both), [`.processes.kill`](commands.md#command--8-processes-kill) (both), [`.settings.set`](commands.md#command--11-settings-set) (`dry::` only)

**Partial implementor:** `.settings.set` implements `dry::` only (no `force::`).

**Why NOT in this group:**
- `version::`: specifies *what* to install, not *whether* to install
- `v::`: controls display, not execution
- `interval::`: controls guard *frequency*, not execution mode

**Typical usage:**

```sh
cm .version.install dry::1          # preview
cm .version.install force::1        # bypass idempotency
cm .version.guard dry::1 force::1   # preview forced guard
cm .processes.kill dry::1 force::1   # preview forced kill
```

---

### Group :: 3. Settings Identity

Identify the settings entry being operated on. Both parameters specify
the target of a settings read or write.

**Coherence test:** "Does this parameter identify the settings entry?" — YES for both.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| [`key::`](params.md#parameter--6-key) | [`SettingsKey`](types.md#type--4-settingskey) | Entry name |
| [`value::`](params.md#parameter--7-value) | [`SettingsValue`](types.md#type--5-settingsvalue) | Entry value (type-inferred) |

**Used by:** [`.settings.get`](commands.md#command--10-settings-get) (`key::` only), [`.settings.set`](commands.md#command--11-settings-set) (both)

**Partial implementor:** `.settings.get` implements `key::` only (read operation — no `value::`).

**Why NOT in this group:**
- `version::`: specifies installation target, not settings target
- `dry::`: controls execution mode, not target identification

**Typical usage:**

```sh
cm .settings.get key::theme
cm .settings.set key::theme value::dark
```
