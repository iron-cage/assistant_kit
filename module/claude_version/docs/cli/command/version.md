# version — Version Namespace Commands

### Scope

- **Purpose**: Reference for version-namespace clv commands.
- **Responsibility**: Command syntax, parameters, exit codes, and cross-references for all `.version.*` commands.
- **In Scope**: `.version.show`, `.version.install`, `.version.guard`, `.version.list`, `.version.history`.
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
| 3 | [`.version.list`](#command-6-versionlist) | Lists aliases that may resolve to installed version |
| 4 | [`.version.history`](#command-12-versionhistory) | Shows release history for version selection |
| 5 | [`.status`](root.md#command-2-status) | Includes version in broader environment snapshot |

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

Download and install a Claude Code version via the official installer (curl). Supports hot-swap and 8-layer version locking (Layers 1–4, 6, and 8 prevent unwanted version changes via auto-update, manual update, or channel drift; Layer 5 stores the preferred version as a recovery signal for `.version.guard`; Layer 7 enforces a minimum-version floor). Accepts named aliases (`stable`, `latest`, `month`) and semver strings. Already-at-target is a no-op (exit 0) unless `force::1` is set.

-- **Parameters:** version::, dry::, force::, v::, format::
-- **Exit Codes:** 0 (success) | 1 (invalid version spec) | 2 (installer failure)

**Syntax:**

```sh
clv.version.install [version::VER] [dry::1] [force::1] [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`version::`](../param/01_version.md) | [`VersionSpec`](../type/03_version_spec.md) | stable | No | Version to install |
| [`dry::`](../param/02_dry.md) | bool | false | No | Preview install command without executing |
| [`force::`](../param/03_force.md) | bool | false | No | Bypass idempotency check |
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Output detail level |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |

**Algorithm (5 steps):**
1. Resolve `version::` alias (`stable`, `latest`, `month`) or validate the semver string against known patterns.
2. Compare resolved target against installed version; exit 0 (no-op) if equal and `force::0`.
3. Apply all 5 version-lock layers (chmod 555, symlink guard, preference storage, etc.).
4. Execute the official curl installer for the resolved version.
5. Verify installed version matches target; store preferred version for `.version.guard` recovery.

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
| 4 | [`v::`](../param/04_v.md) |
| 5 | [`format::`](../param/05_format.md) |

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.version.show`](#command-3-versionshow) | Verifies installed version after install |
| 2 | [`.version.guard`](#command-5-versionguard) | Guards against drift from newly installed version |
| 3 | [`.version.list`](#command-6-versionlist) | Lists version aliases before selecting a target |
| 4 | [`.version.history`](#command-12-versionhistory) | Shows release history for version selection |

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

List all named version aliases (`stable`, `month`, `latest`) with their currently pinned values. These are compile-time constants; they do not query the network.

-- **Parameters:** v::, format::
-- **Exit Codes:** 0 (always)

**Syntax:**

```sh
clv.version.list [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Output detail level |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |

**Algorithm (2 steps):**
1. Load the compile-time version alias table (`stable`, `month`, `latest` → pinned semver values).
2. Render the alias-to-version mapping in the requested format.

**Examples:**

```sh
clv.version.list
clv.version.list format::json
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
| 1 | [`.version.show`](#command-3-versionshow) | Shows which alias is currently installed |
| 2 | [`.version.install`](#command-4-versioninstall) | Installs one of the listed version aliases |
| 3 | [`.version.guard`](#command-5-versionguard) | Guards against drift from a listed alias |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |

---

**Category:** version
**Complexity:** 2
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low

---

### Command :: 12. `.version.history`

Fetch and display recent Claude Code release history from the GitHub Releases API (`anthropics/claude-code`). Use this to see what changed across recent versions, find when a specific fix landed, or review the full changelog for any release. Response is cached locally for 1 hour.

-- **Parameters:** count::, v::, format::
-- **Exit Codes:** 0 (success) | 2 (network failure or HOME unset)

**Syntax:**

```sh
clv.version.history [count::N] [v::N] [format::FMT]
```

**Parameters:**

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| [`count::`](../param/09_count.md) | u64 | 10 | No | Number of recent releases to show |
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | 1 | No | Output detail level |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | text | No | Output format |

**Algorithm (3 steps):**
1. Check local 1-hour cache for GitHub Releases API response; fetch from `anthropics/claude-code` releases endpoint if stale or absent.
2. Select the `count::N` most recent releases from the response payload.
3. Render each release (tag, date, changelog summary) in the requested format.

**Examples:**

```sh
# Default: 10 most recent releases with one-line summaries
clv.version.history

# Show 3 most recent releases
clv.version.history count::3

# Minimal output: version and date only
clv.version.history v::0

# Full changelog per release
clv.version.history count::1 v::2

# JSON format for scripting
clv.version.history format::json count::5
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
| 1 | [`count::`](../param/09_count.md) |
| 2 | [`v::`](../param/04_v.md) |
| 3 | [`format::`](../param/05_format.md) |

### Related Commands

| # | Command | Relationship |
|---|---------|-------------|
| 1 | [`.version.show`](#command-3-versionshow) | Checks which release from history is installed |
| 2 | [`.version.install`](#command-4-versioninstall) | Installs a release from history |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |

---

**Category:** version
**Complexity:** 3
**API Requirement:** Read
**Idempotent:** Yes
**Risk Level:** Low
