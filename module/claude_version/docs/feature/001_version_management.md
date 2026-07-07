# Feature: Version Management

### Scope

- **Purpose**: Document the Claude Code version installation, inspection, and guard commands.
- **Responsibility**: Describe version install, show, list, guard, history, alias resolution, hot-swap behavior, and idempotency rules.
- **In Scope**: `.version.install`, `.version.show`, `.version.list`, `.version.guard`, `.version.history`, version aliases, hot-swap, preferred version persistence.
- **Out of Scope**: 8-layer version lock design (→ `pattern/001_version_lock.md`), process listing (→ `feature/002_process_lifecycle.md`).

### Design

**Version commands:**

- `.version.show` — prints the currently installed Claude Code version
- `.version.list` — lists available version aliases with their pinned semver values
- `.version.install` — installs a specified version via the official Anthropic installer (`curl -fsSL https://claude.ai/install.sh | bash -s -- {version}`)
- `.version.guard` — detects drift from the preferred version and reinstalls if needed
- `.version.history` — fetches recent release history from the GitHub Releases API

**Version aliases:** Three named aliases with compile-time pinned semver values:

| Alias | Pinned Value | Description |
|-------|-------------|-------------|
| `latest` | *(installer resolves)* | Most recent published release |
| `stable` | `2.1.78` | Pinned recommended release |
| `month` | `2.1.74` | ~1 month old release |

Aliases are resolved to their semver before passing to the installer. `latest` is passed as-is.

**Idempotency:** `.version.install` skips re-installation if the installed version already matches the resolved semver. The guard compares against the resolved semver, not the alias name. Override with `force::1`. The guard is always skipped for `latest` (always re-install to get newest).

**Hot-swap:** When Claude Code processes are running during `.version.install`, the old binary is removed before installation begins. Running sessions keep their open file descriptor (Unix semantics) and continue unaffected. New sessions use the newly installed binary.

**Preferred version persistence:** After every successful `.version.install` (including idempotent early-return), two keys are written to `settings.json`:
- `preferredVersionSpec` — the alias or semver requested
- `preferredVersionResolved` — concrete semver at install time, or `null` for `latest`; advisory for alias specs (the guard re-resolves `preferredVersionSpec` through the current alias table at guard time and uses that as the target); authoritative only for concrete semver specs

**Version guard:** `.version.guard` reads the preferred version from settings and:
1. No preference → defaults to `stable`
2. Preference is `latest` → verifies auto-update config, fixes if wrong
3. For alias specs: re-resolves `preferredVersionSpec` through the current alias table; uses the result as the target semver (`preferredVersionResolved` is not used as the target for alias specs)
4. Installed version matches resolved target → exits 0
5. Drift detected → reinstalls target version

Optional `version::SPEC` overrides the stored preference for a single invocation without writing to `settings.json`.

**Watch mode:** `interval::N` (N > 0) loops every N seconds. On drift, reinstalls automatically. Install errors in watch mode are logged to stderr and the loop continues. `interval::0` (default) is one-shot mode.

**Release history:** `.version.history` fetches from the GitHub Releases API (`anthropics/claude-code`). Response cached in `~/.claude/.transient/` for 1 hour. `count::N` limits output (default 10). `count::0` produces empty output, exits 0. Verbosity: `v::0` (bare version+date), `v::1` (version+date+summary), `v::2` (full changelog).

### Features

| File | Relationship |
|------|-------------|
| [feature/004_dry_run.md](004_dry_run.md) | dry::1 preview mode for .version.install and .version.guard |
| [feature/005_cli_design.md](005_cli_design.md) | CLI routing, parameter parsing, exit codes |

### Runtime Files

| File | Relationship |
|------|-------------|
| [runtime_file/001_version_history_cache.md](../runtime_file/001_version_history_cache.md) | Cache file written by .version.history |
| [runtime_file/002_versions_directory.md](../runtime_file/002_versions_directory.md) | Directory created/purged/locked by .version.install and .version.guard |
| [runtime_file/003_binary_symlink.md](../runtime_file/003_binary_symlink.md) | Symlink retargeted by .version.install and .version.guard |

### Patterns

| File | Relationship |
|------|-------------|
| [pattern/001_version_lock.md](../pattern/001_version_lock.md) | 8-layer lock applied after successful install |
| [pattern/002_parameter_trace.md](../pattern/002_parameter_trace.md) | Unconditional stderr trace on 6 of the 10 mutating functions this feature calls |
| [../../../../contract/claude_code/docs/pattern/001_version_pinning.md](../../../../contract/claude_code/docs/pattern/001_version_pinning.md) | Official upstream pinning landscape this feature's `.version.install`/`.version.guard` operate within |

### Pitfalls

| File | Relationship |
|------|-------------|
| [pitfall/002_symlink_retarget.md](../pitfall/002_symlink_retarget.md) | Symlink retarget bypass that .version.install mitigates |

### Sources

| File | Relationship |
|------|-------------|
| `../../src/commands/version.rs` | Version command routines |

### Provenance

| Source | Notes |
|--------|-------|
| `spec.md` (deleted) | FR-12 through FR-21, Command Inventory (commands 3-6, 12), Parameter Inventory |

### Tests

| File | Relationship |
|------|-------------|
| [tests/docs/feature/001_version_management.md](../../tests/docs/feature/001_version_management.md) | Feature test spec |
