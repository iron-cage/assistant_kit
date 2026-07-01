# Behavior B32: CLAUDE.md @-Reference Path Acceptance Filter

### Scope

- **Purpose**: Document the path format rules enforced by the `iy4()` function when parsing `@`-references from CLAUDE.md content — which path prefixes are accepted, which are silently rejected, and how accepted paths are resolved by the `C9()` resolver.
- **Responsibility**: Authoritative instance for behavior B32 — defines which @-reference path formats the claude binary accepts, the C9 path resolver tilde expansion behavior, and the silent rejection of environment variable syntax (`$VAR`, `%VAR%`).
- **In Scope**: @-reference regex parsing; path prefix filter logic (accepted: `./`, `~/`, `/`, `[a-zA-Z0-9._-]`; rejected: `$`, `%`, `@`, `#`, `^`, `&`, `*`, `(`, `)`); C9 path resolver behavior; fragment stripping; code block @-ref exclusion; whitespace requirement before `@`.
- **Out of Scope**: Content transformations after path resolution (→ [B34](034_b34_claudemd_content_pipeline.md)); size limits and I/O error handling (→ [B33](033_b33_claudemd_loading_limits.md)); file type extension whitelist (→ [B33](033_b33_claudemd_loading_limits.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 99% | **Tier**: UNVERIFIED | **Since**: v2.1.74 | **Evidence**: E58

The CLAUDE.md @-reference parser (`iy4()`) applies a strict path format filter before any file I/O. Paths not matching the filter are silently discarded — no error, no warning, no logging to the model or UI.

**@-reference syntax requirements:**
- Must be preceded by whitespace or appear at line start: regex `/(?:^|\s)@((?:[^\s\\]|\\ )+)/g`
- Must NOT be inside a fenced code block or inline code span — `code` and `codespan` token types are entirely skipped by the parser
- Fragment suffix (`#section`) is stripped before filter evaluation; if nothing remains after stripping, the reference is discarded

**Accepted path prefixes (pass the filter):**
- `./` — relative to parent file's directory
- `~/` — tilde-expanded to `os.homedir()` by the C9 resolver
- `~` alone — resolved to `os.homedir()` exactly
- `/` (absolute, not lone `/`) — normalized by the C9 resolver
- First character in `[a-zA-Z0-9._-]` — bare names like `RTK.md`, `subdir/file.md`

**Rejected path prefixes (silently discarded, no fallback):**
- `$` — environment variable syntax: `@$GENAI/…`, `@$HOME/…`, `@$PRO/…`; dollar sign fails the `[a-zA-Z0-9._-]` character check and matches none of the explicit prefix tests
- `%` — Windows env var syntax: `@%USERPROFILE%/…`; blacklisted by `/^[#%^&*()]+/` regex
- `#`, `^`, `&`, `*`, `(`, `)` — all in the same blacklist pattern
- `@` — double-at (`@@something`): explicitly rejected by `j.startsWith("@")` check
- Empty after fragment strip — e.g., `@#only-fragment` discards to nothing

**C9 path resolver behavior (confirmed from binary, pos 108,423,272):**
- `K === "~"` → `os.homedir().normalize("NFC")`
- `K.startsWith("~/")` → `path.join(os.homedir(), K.slice(2)).normalize("NFC")`
- `path.isAbsolute(O)` → `path.normalize(O).normalize("NFC")`
- Relative path → `path.resolve(baseDir, O).normalize("NFC")` (baseDir = `path.dirname(parent_file)`)
- Null bytes in path → throws `Error("Path contains null bytes")` — hard failure, does not silently return null

**Practical implication for CLAUDE.md authors:**

| Reference | Result | Reason |
|-----------|--------|--------|
| `@$GENAI/governance/foo.md` | 0 tokens | `$` rejected by path filter |
| `@$PRO/any/path.md` | 0 tokens | `$` rejected |
| `@~/pro/genai/governance/foo.md` | Loads | `~/` accepted, tilde expanded |
| `@/home/user1/pro/genai/foo.md` | Loads | Absolute `/` accepted |
| `@RTK.md` | Loads | Bare name, `R` in `[a-zA-Z0-9._-]` |
| `` `@file.md` `` (in inline code) | 0 tokens | Code span skipped entirely |

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E58 | B32 | Code | Binary analysis — `strings /home/user1/.local/share/claude/versions/2.1.74` — v2.1.74 session (2026-06-29) | `iy4()` at line 492301 of strings output; `C9()` at binary offset 108,423,272 | `iy4()` path filter extracted: `j.startsWith("./") \|\| j.startsWith("~/") \|\| (j.startsWith("/") && j !== "/") \|\| (!j.startsWith("@") && !j.match(/^[#%^&*()]+/) && j.match(/^[a-zA-Z0-9._-]/))`. `C9()` resolver extracted: `if(K==="~")return Uo_.homedir().normalize("NFC"); if(K.startsWith("~/"))return SZ.join(Uo_.homedir(),K.slice(2)).normalize("NFC"); if(SZ.isAbsolute(O))return SZ.normalize(O).normalize("NFC"); return SZ.resolve(q,O).normalize("NFC")`. Regex confirmed: `/(?:^|\s)@((?:[^\s\\]|\\ )+)/g`. Code/codespan skip confirmed: `if(H.type==="code"\|\|H.type==="codespan")continue`. Fragment strip: `let w=j.indexOf("#"); if(w!==-1)j=j.substring(0,w)`. |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, overview table, invalidation tests |
| behavior | [033_b33_claudemd_loading_limits.md](033_b33_claudemd_loading_limits.md) | B33: silent failure modes and size limits after path resolution |
| behavior | [034_b34_claudemd_content_pipeline.md](034_b34_claudemd_content_pipeline.md) | B34: content transformations and access control once file is loaded |
| behavior | [030_b30_subagent_context_inheritance.md](030_b30_subagent_context_inheritance.md) | B30: subagents inherit full CLAUDE.md context — what gets loaded reaches subagents |
| fault | [../fault/readme.md](../fault/readme.md) | Q6: `$VAR` @-reference prefix silently rejected — quirk entry referencing this behavior |
