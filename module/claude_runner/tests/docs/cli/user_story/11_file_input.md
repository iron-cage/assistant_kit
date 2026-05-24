# User Story: File Input

- **Source:** [docs/cli/user_story/011_file_input.md](../../../../docs/cli/user_story/011_file_input.md)
- **Primary flags:** `--file`
- **Command:** `run`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `--file` pipes file content as subprocess stdin |
| US-2 | Parameter interaction | `--file` combined with `--print` and a message |
| US-3 | Failure path | `--file` with non-readable path errors with OS message |
| US-4 | Boundary | `--file` path resolved relative to caller cwd after `--dir` |

---

### US-1: file content piped as stdin

- **Given:** A readable file exists at a known path
- **When:** `clr --file /path/to/input.txt --dry-run "Summarize this"`
- **Then:** Assembled command shows `--file /path/to/input.txt`; subprocess would receive file bytes on stdin alongside the prompt
- **Exit:** 0

### US-2: file with print mode

- **Given:** A readable file exists
- **When:** `clr --file /path/to/input.txt "Extract the key points" --dry-run`
- **Then:** Assembled command includes both `--file` and `--print` (message triggers print mode); file content feeds stdin while message provides the prompt
- **Exit:** 0

### US-3: non-readable file path

- **Given:** No file at `/tmp/nonexistent_input.txt`
- **When:** `clr --file /tmp/nonexistent_input.txt "test"`
- **Then:** Error message includes the path and OS error (e.g., "No such file or directory")
- **Exit:** non-zero

### US-4: path resolution with --dir

- **Given:** File `data.txt` exists in `/tmp/project/` but not in caller's cwd
- **When:** `clr --dir /tmp/project --file data.txt --dry-run "Analyze"`
- **Then:** `--file` path is resolved relative to the effective working directory (after `--dir` applies); assembled command references the resolved path
- **Exit:** 0
