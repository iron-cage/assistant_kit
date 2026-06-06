# User Story Tests: Output File Capture

**User Story:** [023 — Output File Capture](../../../../docs/cli/user_story/023_output_file_capture.md)

### Test Case Index

| ID | Scenario | Expected | Status |
|----|----------|----------|--------|
| US23-1 | `clr -p --output-file <tmp> "Repeat: hello"` — file created, stdout printed | File contains captured output; stdout identical | ✅ |
| US23-2 | `clr --dry-run --output-file /tmp/nofile.txt "task"` — dry-run skips write | Exit 0; file NOT created at given path | ✅ |
| US23-3 | `clr -p --output-file /nonexistent_dir/out.txt "task"` — write error | Exit 1; stderr contains OS error and path | ✅ |
| US23-4 | `CLR_OUTPUT_FILE=<tmp> clr -p "task"` — env var applies | File created with captured output | ✅ |
