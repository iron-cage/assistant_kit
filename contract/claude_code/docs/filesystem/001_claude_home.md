# Filesystem: Claude Home

### Scope

- **Purpose**: Document all filesystem paths within the `~/.claude/` configuration root accessed by claude_version.
- **Responsibility**: Authoritative instance for the `~/.claude/` cluster — every path, its access type, the commands that use it, and its purpose.
- **In Scope**: All paths under `~/.claude/` including settings, credentials, version markers, projects, sessions, session-env, stats-cache, the `.transient/` cache, and downloads — plus `$HOME/.claude.json`, which is *adjacent to* rather than inside the root but is documented here because that is where readers look for it (see § The `.claude.json` Trap).
- **Out of Scope**: Paths outside `~/.claude/` (→ [002_local_install.md](002_local_install.md), [003_credential_store.md](003_credential_store.md), [004_proc_system.md](004_proc_system.md)); storage organization and containment model (→ [`../storage/`](../storage/readme.md)).

### Paths

| Path | Type | Access | Used By | Purpose |
|------|------|--------|---------|---------|
| `~/.claude/` | dir | R | all commands | Configuration root; base for all `ClaudePaths` methods |
| `~/.claude/settings.json` | file | R/W | `.settings.*`, `.version.install`, `.version.guard`, `.status` | User settings; flat JSON with nested object preservation |
| `~/.claude/settings.json.tmp` | file | W | `.settings.set`, `.version.install`, `.version.guard` | Atomic write staging; renamed to `settings.json` on success |
| `~/.claude/.credentials.json` | file | R/W | `.credentials.status`, `.credentials.check`, `.account.save`, `.account.switch` | Active OAuth token; read for status display, overwritten atomically |
| `~/.claude/version-markers.json` | file | R/W | `.version.*` | User-defined version alias markers (`markers_file()`) |
| `~/.claude/projects/` | dir | R | (reserved) | Conversation history root |
| `~/.claude/sessions/` | dir | R | (reserved) | Session records |
| `~/.claude/session-env/` | dir | R | (reserved) | Per-session environment records |
| `~/.claude/stats-cache.json` | file | R | (reserved) | Usage statistics cache |
| `~/.claude/.transient/version_history_cache.json` | file | R/W | `.version.*` | Cached version history (`version_history_cache_path()`) |
| `~/.claude/downloads/` | dir | ❓ | ❓ unattributed | See § Unattributed Paths — no evidence links it to either side |
| `$HOME/.claude.json` | file | R | `.credentials.status`, `.account.*` | OAuth account metadata. **Adjacent to `~/.claude/`, not inside it** — see § The `.claude.json` Trap |

### The `.claude.json` Trap

`.claude.json` sits at `$HOME/.claude.json`, one level **above** the configuration root — it
is a sibling of `~/.claude/`, not a member of it. `~/.claude/.claude.json` does not exist.

The near-identical names make `base().join( ".claude.json" )` read as obviously correct
while resolving one directory too deep. That exact construction shipped and was corrected
as `Fix(BUG-270)` — see [`../../../../module/claude_profile/src/commands/credentials.rs`](../../../../module/claude_profile/src/commands/credentials.rs).
Earlier revisions of this document and of [readme.md](readme.md) carried the same wrong path
after the code had already been fixed.

The failure is silent in the read direction: `read_to_string(...).unwrap_or_default()` on a
nonexistent path yields `""`, which parses as "no account metadata" rather than an error.

Its two most-read fields are **nested**, not top-level:

```
$HOME/.claude.json
└── oauthAccount            # 19 keys
    ├── emailAddress
    ├── organizationName
    ├── displayName
    ├── organizationRole
    └── billingType  …
```

An earlier revision described this file as simply "provides `emailAddress` and
`organizationName`", which reads as top-level. Neither key exists at the top level; the
surveyed file has 75 top-level keys and neither is among them.

Verify both halves of the trap yourself:

```bash
ls -la ~/.claude.json          # exists — the real file
ls -la ~/.claude/.claude.json  # No such file or directory — the wrong path

python3 -c "import json;d=json.load(open('$HOME/.claude.json'));\
print('top-level:', 'emailAddress' in d);\
print('nested   :', 'emailAddress' in d.get('oauthAccount',{}))"
# → top-level: False / nested: True
```

### Unattributed Paths

`~/.claude/downloads/` was previously documented as installer staging, "Used By: installer
(`install.sh`)", holding `claude-{ver}-{platform}`. Neither side supports that:

| Check | Result |
|-------|--------|
| References in this repo's source (`*.rs`, `*.sh`) | 0 |
| Quoted-literal scan of the v2.1.220 binary | 0 (§ Provenance in [`../storage/002_support_directories.md`](../storage/002_support_directories.md)) |
| Contents on the surveyed machine | empty directory |

The directory exists, but nothing observed here creates or writes it, and no downloaded
binary was ever seen in it. Recorded as ❓ Uncertain rather than restated as fact — the
same treatment `cld-timeout-config.json` receives in
[`../storage/003_root_files.md`](../storage/003_root_files.md) § Not Claude Code's.

By contrast `.transient/` **is** now attributed — to this repo's own `clv` tooling, not to
the `claude` binary — via `version_history_cache_path()`.

### Resolution

`~/.claude/` resolves via `ClaudePaths::new()` from the `HOME` environment variable. Returns `None` if `HOME` is unset.

```
~/.claude/          = $HOME/.claude/        ClaudePaths::base()
$HOME/.claude.json  = base().parent()/.claude.json    ← NOT base().join()
```

Every other sub-path is `base().join( … )`. `claude_json_file()` is the sole exception and
the only method that walks *up* from the base. The authoritative implementation is
`ClaudePaths` in `claude_core/src/paths.rs`; `claude_profile/src/paths.rs` is a re-export,
not a second implementation. All commands that access these paths must go through it.

**Divergence: `CLAUDE_CONFIG_DIR` is not honoured by this resolution.** `ClaudePaths::new()`
reads `HOME` and appends `.claude` unconditionally. The `claude` binary, by contrast,
relocates its entire tree when `CLAUDE_CONFIG_DIR` is set — see
[`../param/154_config_dir.md`](../param/154_config_dir.md). Under that variable the two
disagree about where configuration lives, and the disagreement is silent in the read
direction: this repo's tooling reads `$HOME/.claude/`, finds the old tree or nothing at
all, and reports it as current. Recorded as a known divergence, not a resolved behavior.

```bash
# Reproduce the disagreement without changing anything:
CLAUDE_CONFIG_DIR=/tmp/alt-claude claude --version   # binary would use /tmp/alt-claude
# ClaudePaths::new() still yields $HOME/.claude — see claude_core/src/paths.rs:61-65
```

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Filesystem master index: full directory tree, path reference table |
| storage | [`../storage/003_root_files.md`](../storage/003_root_files.md) | Root-level files: settings.json, .credentials.json, history.jsonl |
| filesystem | [003_credential_store.md](003_credential_store.md) | Per-account credential files (separate from `~/.claude/.credentials.json`) |
| settings | [`../settings/001_global_settings.md`](../settings/001_global_settings.md) | settings.json write protocol and key table |
| formats | [`../format/002_credentials.md`](../format/002_credentials.md) | `.credentials.json` format: `claudeAiOauth` structure |
| source | `../../../../module/claude_core/src/paths.rs` | `ClaudePaths` — authoritative path implementation, including `claude_json_file()` |
| source | `../../../../module/claude_profile/src/paths.rs` | Re-export of `ClaudePaths`; not a second implementation |
