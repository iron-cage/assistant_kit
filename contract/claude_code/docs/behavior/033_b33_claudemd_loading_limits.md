# Behavior B33: CLAUDE.md Loading Silent-Failure and Truncation Modes

### Scope

- **Purpose**: Document how the claude binary handles I/O failures, enforces size limits, and deduplicates paths during CLAUDE.md loading — all of which operate silently without user-visible errors.
- **Responsibility**: Authoritative instance for behavior B33 — defines ENOENT/EISDIR/EACCES silent handling, the extension whitelist, the 40,000-character content cap, the MEMORY.md 200-line truncation, the 5-level @-include depth limit, and circular/symlink deduplication.
- **In Scope**: Silent null returns for ENOENT/EISDIR; EACCES telemetry-only handling; extension whitelist (~50 types); empty file exclusion; 40,000-char content limit (`Xm`); MEMORY.md 200-line truncation with warning (`$P`); 5-level recursive depth (`ny4`); ultra-memory 3,000-char limit (`QKT`); circular and symlink deduplication via Set in `WN()`.
- **Out of Scope**: @-reference path format filter (→ [B32](032_b32_claudemd_at_ref_path_filter.md)); content pipeline transformations applied after loading (→ [B34](034_b34_claudemd_content_pipeline.md)); access control mechanisms that suppress files before loading (→ [B34](034_b34_claudemd_content_pipeline.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 99% | **Tier**: UNVERIFIED | **Since**: v2.1.74 | **Evidence**: E59

The `Kf_()` file reader and `WN()` recursive loader use silent failure throughout — no error is surfaced to the model or displayed in the UI when a file cannot be loaded or a limit is reached.

**Silent failure modes (Kf_ function):**
- `ENOENT` → `return null` — file not found; reference silently skipped
- `EISDIR` → `return null` — path resolves to a directory; reference silently skipped
- `EACCES` → fires `tengu_claude_md_permission_error` Statsig telemetry event, then `return null` — no user-visible output, no in-session warning
**`Xm` (40,000 chars) is a warning threshold, not a hard limit.** Files exceeding it are fully loaded and injected into the model's context. Three UI mechanisms fire: (1) interactive-mode status bar component (`id:"large-memory-files"`) renders ⚠️ `Large [file] will impact performance (N chars > 40,000) • /memory to edit`; (2) `dV9()` produces a warning string for the settings display; (3) `eIO()` returns a doctor diagnostic with `severity:"warning"`. None of these callers truncate or exclude the file. Estimated impact: 40,000 chars ≈ ~10,000 tokens (at 4 chars/token) ≈ ~465–700 lines.

- Non-whitelisted file extension → `return null` — silently skipped; whitelist covers ~50 types: `.md`, `.txt`, `.json`, `.yaml`, `.toml`, `.rs`, `.py`, `.go`, `.sh`, `.env`, `.sql`, `.graphql`, `.proto`, and many more; binary/compiled/image extensions excluded
- Empty file (content trims to `""`) → `WN()` returns `[]` — silently excluded from injection

**Size limits (confirmed constants from binary v2.1.74):**

| Constant | Value | Governs |
|----------|-------|---------|
| `Xm` | 40,000 chars | **Warning threshold only** — file fully loaded and injected; UI warning fires in interactive mode |
| `$P` | 200 lines | MEMORY.md line cap; lines 201+ dropped; warning appended to truncated content |
| `ny4` | 5 levels | Maximum recursive @-include depth; chains deeper than 5 silently truncate |
| `QKT` | 3,000 chars | **Inoperative in v2.1.74** — `so()` always returns `null`; the "ultra-claude-md" UI component and the `dV9()` warning check that use this constant are permanently inactive |

The MEMORY.md truncation warning is appended inline (visible to model):
> `WARNING: MEMORY.md is N lines (limit: 200). Only the first 200 lines were loaded. Move detailed content into separate topic files and keep MEMORY.md as a concise index.`

**`QKT` status — inoperative (confirmed):** The `so()` function, which is supposed to return the "ultra-memory" content checked against `QKT`, has exactly one definition in the binary: `function so(){return null}`. All three call sites (`dV9()` performance warning, `idO` "ultra-claude-md" UI component `isActive` check, `idO` render) short-circuit on null. The "CLAUDE.md entries marked as IMPORTANT" feature and its `QKT=3,000` char limit are declared but completely inoperative in v2.1.74.

**Deduplication (WN function):**
- Circular @-include references are prevented by a `Set` tracking visited paths — the same file path cannot be included twice regardless of how many distinct @-references point to it
- Symlinks: both the symlink path and the resolved real path (via `realpathSync`, function `A1()`) are added to the visited set — a symlink pointing to an already-included file is deduplicated correctly

**Extension whitelist (Qy4 Set, confirmed):**

`.md` `.txt` `.text` `.json` `.yaml` `.yml` `.toml` `.xml` `.csv` `.html` `.htm` `.css` `.scss` `.sass` `.less` `.js` `.ts` `.tsx` `.jsx` `.mjs` `.cjs` `.mts` `.cts` `.py` `.pyi` `.rb` `.go` `.rs` `.java` `.kt` `.kts` `.scala` `.c` `.cpp` `.h` `.hpp` `.cs` `.swift` `.sh` `.bash` `.zsh` `.fish` `.ps1` `.bat` `.cmd` `.env` `.ini` `.cfg` `.conf` `.sql` `.graphql` `.proto` `.vue` `.svelte` `.astro` `.php` `.lua` `.r` `.dart` `.ex` `.erl` `.clj` `.lock` `.log` `.diff` `.patch` — and others. Filenames without extensions are also accepted.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E59 | B33 | Code | Binary analysis — `strings /home/user1/.local/share/claude/versions/2.1.74` — v2.1.74 session (2026-06-29) | `Kf_()` at line 492301 context; `WN()` at same line; constants at line 492298 | Error handling extracted: `if(K==="ENOENT"\|\|K==="EISDIR")return null; if(K==="EACCES")Q("tengu_claude_md_permission_error",{is_access_error:1,has_home_dir:T.includes(tq())?1:0})`. Constants confirmed: `im6,L1="MEMORY.md",$P=200` at line 492298; `Xm=40000` confirmed via `oo()` function reference; `ny4=5` at WN() depth check `if(q.has(A)\|\|O>=ny4)return[]`; `QKT=3000`. Extension whitelist `Qy4=new Set([...])` at line 492307, listing ~50+ types. MEMORY.md warning text confirmed: `> WARNING: MEMORY.md is ${z.length} lines (limit: ${$P}). Only the first ${$P} lines were loaded.` |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, overview table, invalidation tests |
| behavior | [032_b32_claudemd_at_ref_path_filter.md](032_b32_claudemd_at_ref_path_filter.md) | B32: @-reference path format filter (precedes loading) |
| behavior | [034_b34_claudemd_content_pipeline.md](034_b34_claudemd_content_pipeline.md) | B34: content transformations and access control (follows loading) |
| behavior | [030_b30_subagent_context_inheritance.md](030_b30_subagent_context_inheritance.md) | B30: subagents inherit full CLAUDE.md context including loaded files |
