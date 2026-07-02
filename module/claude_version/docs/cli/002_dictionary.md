# Dictionary

### Scope

- **Purpose**: Domain vocabulary for the clv CLI.
- **Responsibility**: Term definitions for commands, modes, types, and architecture concepts.
- **In Scope**: All domain terms used across clv documentation.
- **Out of Scope**: Command reference (→ `command/readme.md`), parameter reference (→ `param/readme.md`).

### Commands

| Term | Definition |
|------|------------|
| `.help` | Display full command listing, all parameters, and usage examples; triggered by `.help`, empty argv, or bare `.` |
| `.status` | Overview command showing version, session count, and active account |
| `.version.show` | Print the currently installed Claude Code version |
| `.version.install` | Install a Claude Code version via official installer (`curl -fsSL https://claude.ai/install.sh`) |
| `.version.guard` | Check for version drift and restore preferred version; supports one-shot and watch-mode (`interval::N`) |
| `.version.list` | List all named version aliases and their resolution targets |
| `.version.history` | Fetch and display recent release history from GitHub Releases API; cached locally for 1 hour |
| `.processes` | List running Claude Code processes detected by scanning `/proc` |
| `.processes.kill` | Terminate processes: SIGTERM -> 2s -> SIGKILL (normal) or SIGKILL (force) |
| `.settings.show` | Print all key-value pairs from `~/.claude/settings.json` |
| `.settings.get` | Read a single setting by key |
| `.settings.set` | Write a single setting atomically via temp-file rename |
| `.config` | Inspect or modify settings with 4-layer resolution (env var → project → user → catalog default); supports show-all, get, set, and unset modes |

### Modes

| Term | Definition |
|------|------------|
| dry-run (`dry::1`) | Preview mode: prints `[dry-run] would ...` without executing side effects |
| force mode (`force::1`) | Bypass safety guards: idempotency check (`.version.install`) or graceful shutdown (`.processes.kill`) |

### Types

| Term | Definition |
|------|------------|
| ConfigKey | Name of a config key in the 4-layer resolution chain — either a known catalog key (with env var mapping and catalog default) or an arbitrary user-defined key |
| ConfigScope | Write target for `.config`: `user` (default, `~/.claude/settings.json`) or `project` (`{cwd}/.claude/settings.json`) |
| OutputFormat | Display encoding: `text` (human-readable) or `json` (machine-readable); case-sensitive |
| SettingsKey | JSON object key in `~/.claude/settings.json`; dot is literal, not a path separator |
| SettingsValue | Value auto-typed for JSON: `"true"`/`"false"` -> bool, numbers -> number, else -> string |
| VersionSpec | Release target: `stable`, `month`, `latest`, or semver string (e.g., `1.2.3`) |
| VerbosityLevel | Output detail: 0=minimal, 1=normal (default), 2=verbose |

### Architecture

| Term | Definition |
|------|------------|
| 4-layer resolution | Config resolution order: env var > project config > user config > catalog default |
| `/proc` scanning | Process detection via reading `/proc/{pid}/cmdline` for `basename == "claude"` |
| active account | Current account marker stored in `$HOME/.persistent/claude/credential/_active` |
| atomic write | Settings written via temp-file rename (`settings.json.tmp` -> `settings.json`) to prevent corruption |
| CmdError | Two-variant error enum: `Usage` (exit 1) and `Runtime` (exit 2) |
| last-wins | When a parameter appears multiple times, the last occurrence takes effect |
| signal sequence | Normal kill: SIGTERM -> 2 second wait -> SIGKILL survivors |
| type inference | Settings value auto-typing: bool -> number -> string cascade |
| version alias | Named reference (`stable`, `month`, `latest`) resolving to a specific version string |
