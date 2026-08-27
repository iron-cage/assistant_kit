# Behavior B23: CLAUDE_CODE_SESSION_DIR Overrides Storage Directory — REFUTED

### Scope

- **Purpose**: Record that the hypothesised `CLAUDE_CODE_SESSION_DIR` env var does not exist in the `claude` binary, and name the mechanism that actually redirects session storage.
- **Responsibility**: Authoritative instance for behavior B23 — retained as a refuted hypothesis with the disconfirming evidence, per the collection's no-silent-deletion policy.
- **In Scope**: Disconfirmation of `CLAUDE_CODE_SESSION_DIR`; the real redirect mechanism (`CLAUDE_CONFIG_DIR`); why the NEG-ONLY tier could not catch this.
- **Out of Scope**: `--no-session-persistence` that disables storage entirely (→ [B22](022_b22_no_session_persistence.md)); default path encoding (→ [B9](009_b9_storage_path_encoding.md)).

### Behavior

**Status**: ❌ Refuted | **Certainty**: 95% refuted | **Tier**: NEG-ONLY (insufficient — see below) | **Refuted at**: v2.1.220 | **Evidence**: E43, E44, E72, E73

**The original hypothesis was:** *"`CLAUDE_CODE_SESSION_DIR` env var overrides session storage directory — when set, session `.jsonl` files are read from and written to the specified path instead of the default `~/.claude/projects/{encoded-path}/`."*

**That hypothesis is refuted.** The literal string `CLAUDE_CODE_SESSION_DIR` does not occur anywhere in the v2.1.220 binary (0 occurrences across 271 MB), and it appears in no official Claude Code documentation. A variable the binary never names cannot be one the binary reads.

**The mechanism that actually exists** is `CLAUDE_CONFIG_DIR` (28 occurrences in the same binary, and officially documented as the supported way to "Move storage off `~/.claude`"). It relocates the whole config directory, not the session subdirectory alone, so it is not a drop-in substitute for what B23 described. Two adjacent officially documented controls complete the picture:

| Goal | Documented mechanism | Present in v2.1.220 |
|------|---------------------|---------------------|
| Move storage off `~/.claude` | `CLAUDE_CONFIG_DIR` | Yes (28) |
| Name the `<project>` directory yourself | `CLAUDE_CODE_PROJECT_DIR_NAME` | No — requires v2.1.234 |
| Suppress transcript writes in all modes | `CLAUDE_CODE_SKIP_PROMPT_HISTORY` | Yes (9) |
| Suppress writes for one non-interactive run | `--no-session-persistence` (→ [B22](022_b22_no_session_persistence.md)) | Yes |

**Why the test tier could not catch this — and this is the general lesson.** The NEG-ONLY tier asserts only that the binary "does not explicitly reject" the env var at startup. An env var the binary has never heard of is *also* not explicitly rejected, so a nonexistent variable and a working one produce byte-identical test results. NEG-ONLY cannot distinguish *accepted* from *silently ignored* from *does not exist* — the tier legend states the first two, and this refutation adds the third. Every remaining NEG-ONLY behavior carries the same blind spot; see [B11](011_b11_auto_continue_env.md), refuted by the same method.

**Consumer impact:** `../param/057_session_dir.md` documents this variable as a real parameter, and any workspace code setting it is setting a no-op. Both need correcting against `CLAUDE_CONFIG_DIR`.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E43 | B23 | Doc | `../param/057_session_dir.md` | Description | Documents `CLAUDE_CODE_SESSION_DIR` env var that overrides session storage directory |
| E44 | B23 | Test | `../../tests/behavior/b23_session_dir_override.rs` | `b23_session_dir_env_var_not_rejected` | Binary does not explicitly reject `CLAUDE_CODE_SESSION_DIR` env var at startup — passes identically for a nonexistent variable, which is why it did not catch this refutation |
| E72 | B23, B11 | Experiment | Binary string scan — `grep -ac <VAR> ~/.local/share/claude/versions/2.1.220` (2026-08-27) | v2.1.220 native binary, 271,825,824 bytes | Occurrence counts: `CLAUDE_CODE_SESSION_DIR` = 0, `CLAUDE_CODE_AUTO_CONTINUE` = 0. Positive controls in the same scan: `CLAUDE_CONFIG_DIR` = 28, `CLAUDE_CODE_SKIP_PROMPT_HISTORY` = 9, `CLAUDE_CODE_ENTRYPOINT` = 41, `CLAUDECODE` = 20, `cleanupPeriodDays` = 12. Negative control `TOTALLY_FAKE_VAR_XYZ` = 0. Method control: `CLAUDE_CODE_PROJECT_DIR_NAME` = 0, which is the expected result since official docs state it requires v2.1.234 — confirming the scan reports absence correctly rather than under-matching. |
| E73 | B23 | Doc | Official Claude Code documentation (code.claude.com/docs/en/sessions § Where transcripts are stored) | Configuration table | Lists the supported controls for transcript location and retention: `CLAUDE_CONFIG_DIR` to "Move storage off `~/.claude`", `CLAUDE_CODE_PROJECT_DIR_NAME` to name the project directory (v2.1.234+), `cleanupPeriodDays` for the 30-day retention, `CLAUDE_CODE_SKIP_PROMPT_HISTORY` to suppress transcript writes in all modes, and `--no-session-persistence` for one non-interactive run. `CLAUDE_CODE_SESSION_DIR` appears nowhere in official documentation. |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [009_b9_storage_path_encoding.md](009_b9_storage_path_encoding.md) | Default path encoding that this env var overrides |
| behavior | [011_b11_auto_continue_env.md](011_b11_auto_continue_env.md) | `CLAUDE_CODE_AUTO_CONTINUE` env var (related env var, same NEG-ONLY pattern) |
| behavior | [022_b22_no_session_persistence.md](022_b22_no_session_persistence.md) | `--no-session-persistence` (disables rather than redirects) |
| params | `../param/057_session_dir.md` | Canonical parameter definition |
| test | `../../tests/behavior/b23_session_dir_override.rs` | Invalidation test (NEG-ONLY) |
