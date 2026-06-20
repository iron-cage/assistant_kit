# Behavior B24: From-PR Resumes PR-Linked Session

### Scope

- **Purpose**: Document that `--from-pr [value]` resumes a session previously linked to a GitHub pull request.
- **Responsibility**: Authoritative instance for behavior B24 — defines the behavior statement, certainty level, and supporting evidence.
- **In Scope**: `--from-pr [value]` flag; interactive picker (no argument); direct PR number/URL resume; PR-context requirement.
- **Out of Scope**: `--resume`/`-r` for UUID-based resume (→ [B19](019_b19_resume_flag.md)); how PR-linked sessions are originally created.

### Behavior

**Status**: 🎯 Observed | **Certainty**: 75% | **Tier**: FLAG-VFY | **Since**: pre-v1.0 | **Evidence**: E45, E46

`--from-pr` without a value opens an interactive picker listing sessions previously associated with GitHub pull requests. `--from-pr <PR-number-or-URL>` resumes the session linked to that PR directly.

Requires the original session to have been started with PR context (e.g., via `gh pr checkout` workflow). This is the inverse of PR-linked session creation: it finds the stored session by PR identity rather than by UUID or mtime.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E45 | B24 | Observation | `claude --help` live output | `--from-pr` flag entry | Help text documents `--from-pr [value]` flag for resuming sessions linked to GitHub pull requests |
| E46 | B24 | Test | `../../tests/behavior/b24_from_pr_flag.rs` | `b24_from_pr_flag_documented_in_help` | `claude --help` output contains `--from-pr` flag |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [019_b19_resume_flag.md](019_b19_resume_flag.md) | `--resume`/`-r` UUID-based session resume |
| test | `../../tests/behavior/b24_from_pr_flag.rs` | Invalidation test |
