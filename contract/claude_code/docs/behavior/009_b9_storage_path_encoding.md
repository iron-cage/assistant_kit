# Behavior B9: Storage Path Encoding

### Scope

- **Purpose**: Document that Claude Code stores project sessions under `~/.claude/projects/{path-encoded}/` using `/`→`-` path encoding.
- **Responsibility**: Authoritative instance for behavior B9 — defines the encoding rules, certainty level, and supporting evidence.
- **In Scope**: Path encoding rule (`/`→`-`, prefixed with `-`); project directory naming convention; examples.
- **Out of Scope**: UUID projects (web/IDE sessions have UUID-named directories, not path-encoded); project directory growth (→ [B6](006_b6_session_accumulation.md)); storage root layout (→ [`../storage/`](../storage/readme.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 95% | **Tier**: VALIDATED | **Since**: pre-v1.0 | **Evidence**: E8, E19

Claude Code stores project sessions at `~/.claude/projects/{path-encoded}/` where the encoded name is derived from the working directory path:

**Encoding rules:**
1. Prefix with `-` (hyphen)
2. Replace all `/` with `-`
3. Preserve spaces and other characters

**Examples:**
- `/home/user/project` → `-home-user-project`
- `/home/user/my project/code` → `-home-user-my project-code`

This encoding is deterministic and reversible. The leading `-` distinguishes CLI path projects from UUID-named web/IDE projects.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E8 | B9 | Observation | Live storage | `~/.claude/projects/` | Project directory names match `/`→`-` encoding of working directory paths |
| E19 | B9 | Test | `../../tests/behavior/b09_storage_path.rs` | `b9_project_dir_names_follow_encoding_convention` | Real project directory names start with `-` (encoded leading `/`) and decode to existing paths |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| storage | [`../storage/001_projects_directory.md`](../storage/001_projects_directory.md) | Projects directory path encoding detail and UUID vs path project types |
| test | `../../tests/behavior/b09_storage_path.rs` | Invalidation test |
