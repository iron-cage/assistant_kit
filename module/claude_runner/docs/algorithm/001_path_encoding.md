# Algorithm: Path Encoding (Df())

### Scope

- **Purpose**: Convert an absolute filesystem path to a filesystem-safe string used as a Claude Code session or memory storage directory segment.
- **Responsibility**: Specify the exact encoding steps, character mapping, special cases for hyphen-prefixed components, and lossiness properties of the Df() encoding.
- **In Scope**: Encoding algorithm steps, lossy character collisions, hyphen-prefixed directory handling, examples, edge cases, current implementation.
- **Out of Scope**: Decoding heuristics (see `claude_storage_core::path::decode_path`); 200-char truncation+hash extension (not yet implemented); git root detection (→ `002_git_root_detection.md`); session file selection (→ `003_session_file_selection.md`).

### Algorithm

Given an input absolute path `P`:

1. **Validate UTF-8** — error if `P` contains non-UTF-8 bytes.
2. **Strip enclosing separators** — remove leading and trailing `/`.
3. **Split on `/`** — produce a list of path components. Error if the resulting list is empty or contains only an empty string (e.g., root path `/`).
4. **Normalize each component** — replace every `_` (underscore) with `-` (hyphen). This step is lossy: the original underscore is irrecoverable from the encoded form.
5. **Build output string**:
   - Prepend a leading `-`.
   - For each component after the first, insert a separator before the component:
     - If the (post-normalization) component begins with `-`: insert `--` (double hyphen) and append the component body without its leading `-`.
     - Otherwise: insert `-` and append the component as-is.
   - The first component is handled like subsequent components except the separator rule uses a single `-` for the leading hyphen (not a path separator).
6. **Return** the resulting string.

**Lossiness note:** The encoding cannot distinguish between `/` (path separator), `_` (underscore), and `-` (hyphen) in the original path. All three produce `-` in the output. The decoder uses heuristics to reconstruct the most likely original path.

### Examples

| Input Path | Encoded Output |
|------------|----------------|
| `/home/user/project` | `-home-user-project` |
| `/lib/claude_storage` | `-lib-claude-storage` |
| `/commands/-default_topic` | `-commands--default-topic` |
| `/home/user/-commit_sessions/-plan` | `-home-user--commit-sessions--plan` |
| `/-commit` | `--commit` |

**Encoding of the working directory for session storage:**

```
Df("/home/alice/my_project") → "-home-alice-my-project"
CLAUDE_SESSION_DIR = ~/.claude/projects/-home-alice-my-project/
```

### Edge Cases

| Scenario | Behavior |
|----------|----------|
| Root path `/` | Error: empty after normalization |
| Path with non-UTF-8 bytes | Error: UTF-8 validation failure |
| Hyphen-prefixed component (`/-name`) | Double-hyphen prefix in output: `--name` |
| Underscore in component (`my_project`) | Lossy: becomes `my-project`; decoder uses heuristics |
| Consecutive hyphen-prefixed dirs (`/-a/-b`) | `--a--b` — each gets `--` prefix |
| Path with literal hyphen (`/foo-bar`) | `-foo-bar` — indistinguishable from `/foo/bar` encoded |

### Implementation

| Location | Symbol | Role |
|----------|--------|------|
| `claude_storage_core/src/path.rs` | `encode_path(path: &Path) -> Result<String>` | Canonical Df() encoder |
| `claude_storage_core/src/path.rs` | `decode_path(encoded: &str) -> Result<PathBuf>` | Lossy heuristic decoder |
| `claude_storage_core/src/continuation.rs` | `to_storage_path_for(dir: &Path) -> Option<PathBuf>` | Applies `encode_path()` to compute `~/.claude/projects/{encoded}/` |

**Gap:** `to_storage_path_for()` uses `$HOME` (not `$CLAUDE_HOME`). The new `scope_for()` function will fix this by checking the `CLAUDE_HOME` env var. See `variable/001_claude_home.md`.

**Not yet implemented:** The full Df() specification includes a 200-character truncation fallback (truncate to 200 chars + `-` + hash of original path). `encode_path()` does not implement this cap. Paths exceeding 200 characters in their encoded form may cause filesystem issues on some systems. The `scope_for()` implementation should add this cap.

### Related Docs

| File | Relationship |
|------|--------------|
| [`002_git_root_detection.md`](002_git_root_detection.md) | Git root detection — uses Df() on the resolved git root for memory path |
| [`003_session_file_selection.md`](003_session_file_selection.md) | Session selection — operates on the directory computed by Df() |
| [`../variable/001_claude_home.md`](../variable/001_claude_home.md) | CLAUDE_HOME — base used before Df() is applied |
| [`../variable/002_claude_projects_dir.md`](../variable/002_claude_projects_dir.md) | CLAUDE_PROJECTS_DIR — Df() output appended to this |
| [`../variable/003_claude_session_dir.md`](../variable/003_claude_session_dir.md) | CLAUDE_SESSION_DIR — full session path using Df(target_dir) |
| [`../variable/004_claude_memory_dir.md`](../variable/004_claude_memory_dir.md) | CLAUDE_MEMORY_DIR — full memory path using Df(git_root(target_dir)) |
| [`../feature/005_session_path_resolution.md`](../feature/005_session_path_resolution.md) | Feature hub: `scope_for()` and session cross-loading |
