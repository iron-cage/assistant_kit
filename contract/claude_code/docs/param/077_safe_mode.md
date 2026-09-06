# safe_mode

Disables bundled skills and experimental features for safety-constrained deployments.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--safe-mode` |
| Env Var | `CLAUDE_CODE_SAFE_MODE` |
| Config Key | — |

### Type

bool

### Default

off / false

### Since

v2.1.169 (2026-06-08)

### Description

When set, Claude Code disables bundled skills, experimental agent features, and other
non-essential capabilities. Intended for sandboxed or safety-constrained environments
where only core functionality should be active.

Distinct from `CLAUDE_CODE_SANDBOX_MODE` (param 53) — ❌ refuted, 0 occurrences in
the v2.1.220 binary; see [056_sandbox_mode.md](056_sandbox_mode.md). `CLAUDE_CODE_SAFE_MODE`
is a higher-level "disable extras" toggle, unrelated to sandbox permissions.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [078_disable_bundled_skills.md](078_disable_bundled_skills.md) | Bundled skills toggle (companion) |
