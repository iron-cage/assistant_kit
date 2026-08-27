# session_dir

> ❌ **Refuted — this parameter does not exist.** Retained to record the error and point at the real mechanism. See [B23](../behavior/023_b23_session_dir_override.md).

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | ~~`CLAUDE_CODE_SESSION_DIR`~~ — not read by the binary |
| Config Key | — |

### Type

path

### Default

n/a — the variable has no effect

### Since

Never. Documented here from an unverified assumption; refuted against v2.1.220.

### Description

The previous revision of this doc claimed: *"Overrides the directory where session `.jsonl` files are stored for the current invocation… Useful for redirecting session storage to a custom location in CI or multi-user environments."*

**That is false.** The literal string `CLAUDE_CODE_SESSION_DIR` occurs **0 times** in the v2.1.220 binary and appears in no official Claude Code documentation. Setting it is a no-op. This workspace's own bug records reached the same conclusion independently (BUG-490, BUG-493: "claude ≥2.x ignores the `CLAUDE_CODE_SESSION_DIR` override entirely") without this contract doc being updated to match.

**Use instead:**

| Goal | Documented mechanism |
|------|---------------------|
| Move storage off `~/.claude` | `CLAUDE_CONFIG_DIR` |
| Name the `<project>` directory yourself | `CLAUDE_CODE_PROJECT_DIR_NAME` (requires v2.1.234+; absent from v2.1.220) |
| Suppress transcript writes in all modes | `CLAUDE_CODE_SKIP_PROMPT_HISTORY` |
| Suppress writes for one non-interactive run | `--no-session-persistence` |

`CLAUDE_CONFIG_DIR` relocates the entire config directory, not the session subdirectory alone, so it is not a drop-in substitute for what this doc described.

**Verify:**

```bash
V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_SESSION_DIR "$V"   # → 0  (the claim)
grep -ac CLAUDE_CONFIG_DIR       "$V"   # → 28 (positive control)
grep -ac TOTALLY_FAKE_VAR_XYZ    "$V"   # → 0  (negative control)
```

The positive and negative controls matter: without them, a `0` proves only that the scan found nothing, not that it would have found something real.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [055_resume.md](055_resume.md) | Resume sessions (reads from this dir) |
| doc | [058_session_id.md](058_session_id.md) | Session ID stored in this dir |
| doc | [../storage/001_projects_directory.md](../storage/001_projects_directory.md) | Project session storage layout |
| behavior | [../behavior/023_b23_session_dir_override.md](../behavior/023_b23_session_dir_override.md) | Refutation record with the full disconfirming evidence (E43, E44, E72, E73) |
| behavior | [../behavior/009_b9_storage_path_encoding.md](../behavior/009_b9_storage_path_encoding.md) | The default path this variable was believed to override |