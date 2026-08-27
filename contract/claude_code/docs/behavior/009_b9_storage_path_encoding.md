# Behavior B9: Storage Path Encoding

### Scope

- **Purpose**: Document that Claude Code stores project sessions under `~/.claude/projects/{path-encoded}/`, and state the current encoding rule precisely.
- **Responsibility**: Authoritative instance for behavior B9 — defines the encoding rules, certainty level, and supporting evidence.
- **In Scope**: Current encoding rule (every non-alphanumeric → `-`); the long-path truncation-and-hash rule; the superseded `/`-only rule and the legacy directories it produced; project directory naming convention.
- **Out of Scope**: UUID projects (web/IDE sessions have UUID-named directories, not path-encoded); project directory growth (→ [B6](006_b6_session_accumulation.md)); storage root layout (→ [`../storage/`](../storage/readme.md)); relocating the storage root (→ `CLAUDE_CONFIG_DIR`, [B23](023_b23_session_dir_override.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 95% | **Tier**: UNVERIFIED (test does not assert the rule — see below) | **Since**: pre-v1.0; current rule differs from the original | **Evidence**: E8, E19, E74, E75

Claude Code stores project sessions at `~/.claude/projects/{path-encoded}/`, where the encoded name derives from the working directory path.

**Current encoding rule (v2.1.220):**
1. Replace **every non-alphanumeric character** with `-` — including `/`, `_`, `.`, and spaces. A leading `/` therefore produces the leading `-`; it is not a separately applied prefix rule.
2. If the converted name exceeds **200 characters**, truncate to 200 and append a hash of the full path, keeping the directory name inside filesystem limits.

**Examples (current rule):**
- `/home/user/project` → `-home-user-project`
- `/home/user/my project/code` → `-home-user-my-project-code`
- `/home/user1/pro/lib/yrd_core/family_ai/claude_runner` → `-home-user1-pro-lib-yrd-core-family-ai-claude-runner`

**The encoding is NOT reversible.** Because `/`, `_`, `.`, and space all collapse to the same `-`, the original path cannot be recovered from the directory name. Any consumer that decodes by replacing `-` with `/` will produce a wrong path for every source path containing an underscore, dot, or space. This corrects a claim in the previous revision, which described the encoding as "deterministic and reversible" — it is deterministic but strictly lossy.

**Superseded rule.** Earlier Claude Code versions replaced only `/` and preserved `_`. Both encodings coexist on long-lived machines: this machine holds 978 project directories, of which 8 preserve underscores (last written 2026-06-29 through 2026-07-16) while every current-era directory converts them (E75). A consumer scanning real storage must tolerate both forms. The exact version at which the rule changed is not established here — only that it changed between 2026-07-16 and v2.1.220.

**Known test gap.** `b09_storage_path.rs` is tiered VALIDATED in the master index but does not assert this rule. It asserts only that *at least one* directory name starts with `-`, then attempts a `-`→`/` round-trip that it explicitly allows to fail: the final comment reads "Still pass: the naming convention (starts with `-`) was confirmed above." An encoding change of exactly the kind documented here would leave the test green. The tier is therefore corrected to UNVERIFIED until the test asserts the character class rather than the leading character.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E8 | B9 | Observation | Live storage | `~/.claude/projects/` | Project directory names match `/`→`-` encoding of working directory paths |
| E19 | B9 | Test | `../../tests/behavior/b09_storage_path.rs` | `b9_project_dir_names_follow_encoding_convention` | Asserts only that at least one project directory name starts with `-`; the `-`→`/` round-trip is best-effort and the test passes even when every decode fails. Does not assert the character class, so it cannot detect an encoding-rule change. |
| E74 | B9 | Doc | Official Claude Code documentation (code.claude.com/docs/en/sessions § Where transcripts are stored) | Storage path paragraph | "By default, Claude Code stores transcripts as JSONL at `~/.claude/projects/<project>/<session-id>.jsonl`, where `<project>` is your working directory path with non-alphanumeric characters replaced by `-`. For a working directory whose converted name exceeds 200 characters, Claude Code truncates the name to 200 characters and appends a hash of the full path, so the directory name stays within filesystem limits." |
| E75 | B9 | Experiment | Live storage survey — `~/.claude/projects/` (2026-08-27, v2.1.220) | 978 project directories | Character census: 0 directory names contain a space, 0 contain a dot, 8 contain an underscore. The 8 underscore-preserving names were last written 2026-06-29 → 2026-07-16 (legacy rule); every current-era name converts underscores. Direct current-version confirmation: this session's cwd `/home/user1/pro/lib/yrd_core/family_ai/claude_runner/module/claude_runner/docs` writes its transcript to `-home-user1-pro-lib-yrd-core-family-ai-claude-runner-module-claude-runner-docs`, converting `yrd_core`→`yrd-core`, `family_ai`→`family-ai`, `claude_runner`→`claude-runner`. Forward-encoding `sed 's/[^a-zA-Z0-9]/-/g'` over real source directories reproduces existing project directory names exactly. Longest name observed: 168 chars, so the 200-char truncation rule is unexercised on this machine. |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| storage | [`../storage/001_projects_directory.md`](../storage/001_projects_directory.md) | Projects directory path encoding detail and UUID vs path project types |
| behavior | [023_b23_session_dir_override.md](023_b23_session_dir_override.md) | Refuted `CLAUDE_CODE_SESSION_DIR` override; names `CLAUDE_CONFIG_DIR` as the real way to relocate this path |
| test | `../../tests/behavior/b09_storage_path.rs` | Invalidation test |
