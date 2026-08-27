# config_dir

Relocates the entire `~/.claude/` configuration and state tree.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CONFIG_DIR` |
| Config Key | — |

### Type

string (directory path)

### Default

`~/.claude`

### Since

≤v2.1.111 (2026-04-16) — [`../version/028_v2_1_111.md`](../version/028_v2_1_111.md) already treats it as established. No introduction entry exists in the 2.1.74–2.1.220 window, so the variable predates it.

### Description

The single highest-leverage environment variable in this collection: it moves the whole configuration and state tree — settings, projects, history, credentials, shell snapshots, tasks, workflows, lock files — to a different root.

**It is the replacement for the parameter this collection once wrongly documented as `--session-dir`.** See [`057_session_dir.md`](057_session_dir.md), which records that refutation. There is no per-session directory override; `CLAUDE_CONFIG_DIR` relocating the whole tree is the actual mechanism.

**Coverage was incremental — several subsystems ignored it at first.** Each of these was a *fix*, meaning the subsystem previously wrote to `~/.claude/` regardless:

| Version | Subsystem that had been ignoring it |
|---------|-------------------------------------|
| v2.1.111 | `/setup-vertex` and `/setup-bedrock` displaying the wrong `settings.json` path |
| v2.1.136 | The workflow save dialog showing `~/.claude/workflows/` for user-scope saves |
| v2.1.208 | IDE shell-integration lock files |

The lesson generalizes: on any given version, assume some path may still be hardcoded, and verify rather than presume full coverage.

**It is the positive control for binary string-scans in this collection.** At 28 occurrences in v2.1.220 it is reliably present, which is why evidence blocks throughout `docs/` pair a claim's grep against `CLAUDE_CONFIG_DIR` (expect non-zero) and `TOTALLY_FAKE_VAR_XYZ` (expect zero).

### Verification

```bash
V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CONFIG_DIR    "$V"   # → 28
grep -ac TOTALLY_FAKE_VAR_XYZ "$V"   # → 0  (negative control)

# Observe the relocation end-to-end:
export CLAUDE_CONFIG_DIR=/tmp/claude-config-probe
claude --version >/dev/null 2>&1
ls -la /tmp/claude-config-probe       # → the tree is created here, not in ~/.claude
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [057_session_dir.md](057_session_dir.md) | Refuted `--session-dir`; this is the real mechanism |
| doc | [../storage/readme.md](../storage/readme.md) | `~/.claude/` storage architecture this variable relocates |
| doc | [../behavior/009_b9_storage_path_encoding.md](../behavior/009_b9_storage_path_encoding.md) | Project-directory encoding under the config root |
