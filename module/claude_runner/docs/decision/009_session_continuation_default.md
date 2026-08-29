# Decision: Session Continuation By Default

**ID:** D9 · **Category:** Behavior · **Status:** ✅ Adopted

### Scope

- **Purpose**: Record why `clr` continues the previous session by default instead of requiring `-c`, and how the guard that makes it safe was arrived at.
- **Responsibility**: Rationale for default continuation, the flag-surface consequence, and the two-stage BUG-214 history that corrected the underlying assumption.
- **In Scope**: Why continuation is the default; why `-c`/`--continue` left the public flag list; the `session_exists()` guard and the storage path it must check.
- **Out of Scope**: The behavioral specification itself (→ [`../invariant/001_default_flags.md`](../invariant/001_default_flags.md)); path encoding for the project-specific storage directory (→ [`../algorithm/001_path_encoding.md`](../algorithm/001_path_encoding.md)).

### Decision

Session continuation is on by default. Behavioral specification: [`../invariant/001_default_flags.md`](../invariant/001_default_flags.md).

### Rationale

`clr` adds value over the raw `claude` binary by managing session continuity automatically. Most invocations are continuations of ongoing work. Users who want a genuinely fresh start opt in explicitly with `--new-session`. This also decouples `clr` from external session orchestration.

### Consequence

`-c`/`--continue` was removed from the public flag list as redundant; `--new-session` was added as the only way to disable default continuation. Net: 11 flags → 11 flags.

### History

**Fixed (BUG-214, 2026-05-28; reopened and re-fixed 2026-06-03).** The "most invocations are continuations" assumption was false on first use. When no prior session existed in storage, `-c` caused the `claude` binary to exit immediately with `No conversation found to continue`.

The fix added a `session_exists()` guard in `build_claude_command()`: `-c` is injected only when session storage is non-empty. It took two attempts to get the *storage path* right:

| Attempt | Path checked | Outcome |
|---------|--------------|---------|
| Initial fix | `$HOME/.claude/` | Always non-empty — it holds credentials and config — so the guard never fired and the bug survived |
| Re-fix | `$HOME/.claude/projects/{encoded(cwd)}/`, via `claude_storage_core::continuation::check_continuation()` | Project-specific; empty exactly when there is genuinely nothing to continue |

The lesson the reopen recorded: a non-emptiness guard is only as good as the specificity of the directory it tests. `~/.claude/` is never empty for any installed user, so testing it answered a different question than the one being asked.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Decision collection index |
| invariant | [`../invariant/001_default_flags.md`](../invariant/001_default_flags.md) | Behavioral specification for automatic flag injection and opt-out |
| invariant | [`../invariant/009_session_mismatch_detection.md`](../invariant/009_session_mismatch_detection.md) | Diagnostic warning when `-c` resumes a different session than expected |
| algorithm | [`../algorithm/001_path_encoding.md`](../algorithm/001_path_encoding.md) | `{encoded(cwd)}` — how the project-specific storage path is derived |
| source | `../../src/cli/builder.rs` | `build_claude_command()` — the `session_exists()` guard |
| test | `../../tests/cli_args_ext_test.rs` | Session continuation guard coverage |
