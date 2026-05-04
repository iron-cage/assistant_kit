# Dictionary

### Commands

| Term | Definition |
|------|------------|
| `.status` | Overview command showing version, session count, and active account |
| `.version.show` | Print the currently installed Claude Code version |
| `.version.install` | Install a Claude Code version via official installer (`curl -fsSL https://claude.ai/install.sh`) |
| `.version.list` | List all named version aliases and their resolution targets |
| `.processes` | List running Claude Code processes detected by scanning `/proc` |
| `.processes.kill` | Terminate processes: SIGTERM -> 2s -> SIGKILL (normal) or SIGKILL (force) |
| `.settings.show` | Print all key-value pairs from `~/.claude/settings.json` |
| `.settings.get` | Read a single setting by key |
| `.settings.set` | Write a single setting atomically via temp-file rename |

### Modes

| Term | Definition |
|------|------------|
| dry-run (`dry::1`) | Preview mode: prints `[dry-run] would ...` without executing side effects |
| force mode (`force::1`) | Bypass safety guards: idempotency check (`.version.install`) or graceful shutdown (`.processes.kill`) |

### Types

| Term | Definition |
|------|------------|
| VerbosityLevel | Output detail: 0=minimal, 1=normal (default), 2=verbose |
| OutputFormat | Display encoding: `text` (human-readable) or `json` (machine-readable); case-sensitive |
| VersionSpec | Release target: `stable`, `month`, `latest`, or semver string (e.g., `1.2.3`) |
| SettingsKey | JSON object key in `~/.claude/settings.json`; dot is literal, not a path separator |
| SettingsValue | Value auto-typed for JSON: `"true"`/`"false"` -> bool, numbers -> number, else -> string |

### Architecture

| Term | Definition |
|------|------------|
| type inference | Settings value auto-typing via `infer_type()`: bool -> number -> string cascade |
| atomic write | Settings written via temp-file rename (`settings.json.tmp` -> `settings.json`) to prevent corruption |
| version alias | Named reference (`stable`, `month`, `latest`) resolving to a specific version string |
| active account | Current account marker stored in `$PRO/.persistent/claude/credential/_active` (or `$HOME/.persistent/...`) |
| `/proc` scanning | Process detection via reading `/proc/{pid}/cmdline` for `basename == "claude"` |
| signal sequence | Normal kill: SIGTERM -> 2 second wait -> SIGKILL survivors |
| last-wins | When a parameter appears multiple times, the last occurrence takes effect |
| CmdError | Two-variant error enum: `Usage` (exit 1) and `Runtime` (exit 2) |
