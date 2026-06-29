# Behavior B34: CLAUDE.md Content Pipeline — Transformations and Access Control

### Scope

- **Purpose**: Document the content transformations applied to CLAUDE.md files before injection into the system-reminder, and the access control mechanisms that can suppress entire files or the entire claudeMd section.
- **Responsibility**: Authoritative instance for behavior B34 — defines HTML comment stripping (`Kp6`), YAML frontmatter conditional handling (`ly4`), GFM-disabled parsing, User type always-include-external rule, Project/Local external include dialog requirement, `claudeMdExcludes` glob filtering, `tengu_paper_halyard` Statsig flag suppression, and environment variable session-level disablement.
- **In Scope**: HTML comment stripping; YAML `paths:` frontmatter processing; GFM-disabled @-ref parsing; User type unconditional external include; Project/Local external include dialog; `claudeMdExcludes` micromatch filtering; `tengu_paper_halyard` Statsig flag; `CLAUDE_CODE_DISABLE_CLAUDE_MDS` and `CLAUDE_CODE_SIMPLE` env vars; `K2q()` assembly order.
- **Out of Scope**: @-reference path format acceptance rules (→ [B32](032_b32_claudemd_at_ref_path_filter.md)); size limits and I/O error handling during file loading (→ [B33](033_b33_claudemd_loading_limits.md)); subagent context inheritance of the assembled content (→ [B30](030_b30_subagent_context_inheritance.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 99% | **Tier**: UNVERIFIED | **Since**: v2.1.74 | **Evidence**: E60

After a CLAUDE.md file is successfully loaded, its content passes through transformation and suppression layers before being assembled into the `# claudeMd` system-reminder block.

**Content transformations (applied per file):**

| Transform | Function | Effect |
|-----------|----------|--------|
| HTML comment stripping | `Kp6()` | `<!-- ... -->` blocks removed before injection — invisible to model |
| YAML frontmatter | `ly4()` | `paths:` globs used as conditional inclusion rules; frontmatter NOT passed to model as content |
| GFM-disabled parsing | `new $X({gfm: false})` | Marked.js lexes @-references with GFM off — table syntax and other GFM extensions parsed differently during @-ref scanning |
| Code block exclusion | `iy4()` node filter | `@`-references inside fenced code blocks or inline code spans are never parsed as file references |

**Access control (file-level suppression, evaluated before loading):**

- **User type** (`~/.claude/CLAUDE.md`): always `includeExternal=true` — all @-references processed unconditionally; no dialog shown; no approval required
- **Project/Local type**: external @-references (resolving outside the project directory) require explicit user approval via `ClaudeMdExternalIncludesDialog` — silently excluded if approval absent or not yet granted
- **`claudeMdExcludes` setting**: glob patterns in user settings suppress matching file paths before loading; implemented via `micromatch` with `{dot: true}` (matches dotfiles); symlink real paths also checked
- **`tengu_paper_halyard` Statsig flag**: when `true`, `K2q()` silently drops ALL `Project` and `Local` type CLAUDE.md files from the assembled `# claudeMd` block; only `User` and `Managed` type files survive; no in-session notification

**Session-level disablement (entire `# claudeMd` section absent):**

- `CLAUDE_CODE_DISABLE_CLAUDE_MDS` env var: `a$()` sets `K2q()` to null — `# claudeMd` system-reminder entirely omitted
- `CLAUDE_CODE_SIMPLE` env var: same effect as above

**`# claudeMd` assembly order (`K2q` function):**

Files are assembled in this type order: `User` → `Managed` → `Project` → `Local`. Each file wrapped as:
```
Contents of {path} ({type label}):
{content}
```
The `tengu_paper_halyard` check is applied at this assembly step — Project/Local files are skipped in the loop without affecting User/Managed files.

**Consequence for CLAUDE.md authors:**

- HTML comments can be used to annotate CLAUDE.md without leaking content to the model — they are stripped before injection
- YAML frontmatter `paths:` enables conditional rules (only injected when the working directory matches the glob) — the frontmatter block itself never appears in the model's context
- `@`-references placed inside a code fence (documentation of the syntax) will not be followed — only references in plain text or list items are parsed

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E60 | B34 | Code | Binary analysis — `strings /home/user1/.local/share/claude/versions/2.1.74` — v2.1.74 session (2026-06-29) | `K2q()` and `S1()` at lines 492298–492307; `ry4()` (claudeMdExcludes) at line 492301 | `K2q()` assembly confirmed: `let q=Wq("tengu_paper_halyard",!1); for(let K of T){if(q&&(K.type==="Project"\|\|K.type==="Local"))continue; ...}`. Access control: `S1()` passes `includeExternal=!0` for User type unconditionally; Project/Local use approval flag. `ry4()` exclusion check: `_p6.default.isMatch(O,R,K)` (micromatch, `{dot:!0}`). Session disable: `a$()` checks `process.env.CLAUDE_CODE_DISABLE_CLAUDE_MDS\|\|sT(process.env.CLAUDE_CODE_SIMPLE)`. HTML strip: `Kp6` referenced in `Rp6` module exports: `{stripHtmlComments:()=>Kp6,...}`. GFM-off confirmed: `new $X({gfm:!1})` in `iy4()`. |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, overview table, invalidation tests |
| behavior | [032_b32_claudemd_at_ref_path_filter.md](032_b32_claudemd_at_ref_path_filter.md) | B32: @-reference path format filter (precedes content pipeline) |
| behavior | [033_b33_claudemd_loading_limits.md](033_b33_claudemd_loading_limits.md) | B33: silent failure modes and size limits during file loading |
| behavior | [030_b30_subagent_context_inheritance.md](030_b30_subagent_context_inheritance.md) | B30: subagents inherit assembled CLAUDE.md content via system-reminder |
| fault | [../fault/readme.md](../fault/readme.md) | Q7: `tengu_paper_halyard` silently suppresses Project/Local CLAUDE.md files |
