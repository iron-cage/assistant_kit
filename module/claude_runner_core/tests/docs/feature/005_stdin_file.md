# Feature :: Stdin File Piping

### Scope

- **Purpose**: FT- test cases verifying the `stdin_file` and `stdin_content` fields on `ClaudeCommand` and their effect on describe output and execution.
- **Responsibility**: Acceptance criteria confirming stdin-file/stdin-content describe rendering, nonexistent-path error propagation, dry-run file-open/materialization skipping, override semantics, `stdin_file`-over-`stdin_content` precedence, and parity across all four spawn methods (`execute()`, `execute_interactive()`, `spawn_piped()`, `spawn_tty()`).
- **In Scope**: `with_stdin_file()`/`with_stdin_content()` describe output, absence of stdin reference when unset, `Err` on nonexistent path (both `execute()` and `execute_interactive()`), dry-run skip of file open / content materialization, last-write-wins override, `stdin_file`-over-`stdin_content` precedence, `describe_compact()` inline placement, `stdin_content` delivery across all four spawn methods.
- **Out of Scope**: `run_isolated()`/`IsolatedModel` behavior (-> `004_run_isolated.md`), CLAUDECODE env var unsetting (-> `006_unset_claudecode.md`), `claude_runner` CLI-layer sourcing of `stdin_content` from piped stdin (-> `../../../../claude_runner/tests/docs/cli/param/025_file.md`).

Behavioral requirement cases for the `stdin_file` and `stdin_content` fields on `ClaudeCommand`. See
[feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md) for the specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | `with_stdin_file(path)` → path appears in dry-run describe output | Behavioral Divergence |
| FT-2 | No `stdin_file` → describe output contains no stdin reference | Behavioral Divergence |
| FT-3 | Nonexistent file path → `execute()` returns `Err` with path in message | Error Path |
| FT-4 | `with_dry_run(true)` + nonexistent path → `execute()` returns `Ok` (file not opened) | Dry-Run Interaction |
| FT-5 | `with_stdin_file(a).with_stdin_file(b)` → describe shows `b`, not `a` (last-write wins) | Override Semantics |
| FT-6 | Nonexistent file path → `execute_interactive()` returns `Err` with path in message | Interactive Error Path |
| FT-7 | `with_dry_run(true)` + nonexistent path → `execute_interactive()` returns `Ok` (file not opened) | Dry-Run Interaction |
| FT-8 | `describe_compact()` with `stdin_file` set starts with `"env -u CLAUDECODE"` (not `"< path"`) | Inline Placement |
| FT-9 | `with_stdin_content(bytes)` → dry-run describe output shows `<piped stdin, N bytes>` | Behavioral Divergence |
| FT-10 | `with_stdin_file(path).with_stdin_content(bytes)` → describe shows the path, not the byte count (`stdin_file` wins) | Override Semantics |
| FT-11 | `with_stdin_content(bytes)` (no `stdin_file`) → `execute()` subprocess receives `bytes` on stdin | Behavioral Divergence |
| FT-12 | `with_stdin_content(bytes)` (no `stdin_file`) → `execute_interactive()` subprocess receives `bytes` on stdin | Interactive Behavioral Divergence |
| FT-13 | `with_stdin_content(bytes)` (no `stdin_file`) → `spawn_piped()` child receives `bytes` on stdin | Behavioral Divergence |
| FT-14 | `with_stdin_content(bytes)` (no `stdin_file`) → `spawn_tty()` child receives `bytes` on stdin | Behavioral Divergence |
| FT-15 | `with_dry_run(true)` + `with_stdin_content(bytes)` → `execute()` returns `Ok` (bytes not materialized to a tempfile) | Dry-Run Interaction |

## Test Coverage Summary

- Behavioral Divergence: 6 tests (FT-1, FT-2, FT-9, FT-11, FT-13, FT-14)
- Error Path: 1 test (FT-3)
- Dry-Run Interaction: 3 tests (FT-4, FT-7, FT-15)
- Override Semantics: 2 tests (FT-5, FT-10)
- Interactive Error Path: 1 test (FT-6)
- Interactive Behavioral Divergence: 1 test (FT-12)
- Inline Placement: 1 test (FT-8)

**Total:** 15 feature cases

---

### FT-1: stdin_file in describe output

- **Given:** a temp file at a known path; `ClaudeCommand::new().with_stdin_file(path).with_dry_run(true)`
- **When:** `execute()` is called
- **Then:** Returns `Ok`; the describe output string contains `"< "` followed by the file path
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md)

---

### FT-2: No stdin_file → no stdin reference in describe

- **Given:** `ClaudeCommand::new().with_dry_run(true)` (no `with_stdin_file` call)
- **When:** `execute()` is called
- **Then:** Returns `Ok`; the describe output does NOT contain `"< "`
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md)

---

### FT-3: Nonexistent file → Err with path

- **Given:** a path that does not exist; `ClaudeCommand::new().with_stdin_file(nonexistent_path)` (no dry-run)
- **When:** `execute()` is called
- **Then:** Returns `Err`; the error message contains the file path string
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md)

---

### FT-4: dry_run skips file open for nonexistent path

- **Given:** a path that does not exist; `ClaudeCommand::new().with_stdin_file(nonexistent_path).with_dry_run(true)`
- **When:** `execute()` is called
- **Then:** Returns `Ok` (dry-run returns describe output before any file open attempt)
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md)

---

### FT-5: Last-write wins on repeated with_stdin_file

- **Given:** two distinct paths `path_a` and `path_b`; `ClaudeCommand::new().with_stdin_file(path_a).with_stdin_file(path_b).with_dry_run(true)`
- **When:** `execute()` is called
- **Then:** Returns `Ok`; describe output contains `path_b` and does NOT contain `path_a`
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md)

---

### FT-6: execute_interactive() opens stdin file — nonexistent path → Err with path

- **Given:** a path that does not exist; `ClaudeCommand::new().with_stdin_file(nonexistent_path)` (no dry-run)
- **When:** `execute_interactive()` is called
- **Then:** Returns `Err`; the error message contains the file path string — proving the file-open attempt is made in the interactive execution path, identical to the `execute()` behavior
- **Note:** Mirrors FT-3 but for `execute_interactive()`. The feature spec states the two paths behave identically for stdin connection; this case verifies that guarantee holds.
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md) — Interactive mode section

---

### FT-7: execute_interactive() dry_run skips file open for nonexistent path

- **Given:** a path that does not exist; `ClaudeCommand::new().with_stdin_file(nonexistent_path).with_dry_run(true)`
- **When:** `execute_interactive()` is called
- **Then:** Returns `Ok` (dry-run returns early before any file open attempt)
- **Note:** Mirrors FT-4 but for `execute_interactive()`. Both execution paths must have independent dry_run guards before the file open — testing one does not guarantee the other.
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md)

---

### FT-8: describe_compact() with stdin_file set — `< path` inline on claude invocation line

- **Given:** `ClaudeCommand::new().with_stdin_file(path)` (no dry_run)
- **When:** `describe_compact()` is called
- **Then:** The returned string starts with `"env -u CLAUDECODE"` AND contains `"< "` followed by the path — proving `< path` is inline on the invocation line, not emitted as a separate last line
- **Note:** `contains("< path")` alone is insufficient — it passes even if `< path` is the only line. `starts_with("env -u CLAUDECODE")` is required to guard inline placement.
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md)

---

### FT-9: stdin_content in describe output

- **Given:** `ClaudeCommand::new().with_stdin_content(b"hello".to_vec()).with_dry_run(true)`
- **When:** `execute()` is called
- **Then:** Returns `Ok`; the describe output string contains `"<piped stdin, 5 bytes>"`
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md) — Raw content variant section

---

### FT-10: stdin_file takes priority over stdin_content

- **Given:** a temp file at a known path; `ClaudeCommand::new().with_stdin_file(path).with_stdin_content(b"ignored".to_vec()).with_dry_run(true)`
- **When:** `execute()` is called
- **Then:** Returns `Ok`; the describe output contains the file path and does NOT contain `"<piped stdin,"` — `stdin_file` wins regardless of call order
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md) — Raw content variant section

---

### FT-11: execute() delivers stdin_content bytes to the subprocess

- **Given:** `ClaudeCommand::new().with_stdin_content(b"piped_content".to_vec())` (no `stdin_file`, no dry-run) against a fake `claude` binary that echoes stdin to stdout
- **When:** `execute()` is called
- **Then:** Returns `Ok`; `ExecutionOutput.stdout` contains `"piped_content"` — the bytes were materialized into a tempfile and attached as the subprocess's stdin
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md) — Raw content variant section

---

### FT-12: execute_interactive() delivers stdin_content bytes to the subprocess

- **Given:** `ClaudeCommand::new().with_stdin_content(b"piped_content".to_vec())` (no `stdin_file`, no dry-run)
- **When:** `execute_interactive()` is called
- **Then:** Returns `Ok` with a success exit status — the same materialize-and-attach path as `execute()` runs for the interactive spawn method
- **Note:** Mirrors FT-11 but for `execute_interactive()`. Both execution paths must independently check `stdin_content` — testing one does not guarantee the other.
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md) — Raw content variant section

---

### FT-13: spawn_piped() delivers stdin_content bytes to the child

- **Given:** `ClaudeCommand::new().with_stdin_content(b"piped_content".to_vec())` (no `stdin_file`)
- **When:** `spawn_piped()` is called and the returned `Child`'s output is collected via `wait_with_output()`
- **Then:** The child's captured stdout contains `"piped_content"` — `spawn_piped()`'s existing `Stdio::null()` fallback (when neither field is set) is bypassed in favor of the materialized tempfile
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md) — All four spawn methods section

---

### FT-14: spawn_tty() delivers stdin_content bytes to the child

- **Given:** `ClaudeCommand::new().with_stdin_content(b"piped_content".to_vec())` (no `stdin_file`)
- **When:** `spawn_tty()` is called
- **Then:** The child process receives the materialized tempfile as its stdin, identical in mechanism to `spawn_piped()`
- **Note:** Mirrors FT-13 but for `spawn_tty()` — the method whose stdout/stderr inherit the parent TTY rather than being piped.
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md) — All four spawn methods section

---

### FT-15: dry_run skips stdin_content materialization

- **Given:** `ClaudeCommand::new().with_stdin_content(b"hello".to_vec()).with_dry_run(true)`
- **When:** `execute()` is called
- **Then:** Returns `Ok` (dry-run returns describe output before any tempfile is created) — no `tempfile::tempfile()` call occurs
- **Source:** [feature/005_stdin_file.md](../../../docs/feature/005_stdin_file.md) — Dry-run interaction section
