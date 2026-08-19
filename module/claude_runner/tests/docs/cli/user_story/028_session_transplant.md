# User Story :: Session Cross-Loading (Transplant)

Test case spec for [028_session_transplant.md](../../../../docs/cli/user_story/028_session_transplant.md).

## Test Case Index

| ID | Test Name | AC | Status |
|----|-----------|-----|-----|
| US-1 | Clone outward: transplant planned from source into target storage | AC-1 | ✅ |
| US-2 | Inject inward: runs in CWD, transplant planned into CWD storage | AC-2 | ✅ |
| US-3 | No source history → no `-c`; fresh session starts | AC-3 | ✅ |
| US-4 | `--to` alone defaults `--from` to CWD; clones outward | AC-4 | ✅ |
| US-5 | `--to` alias accepted; behavior identical to `--dir` | AC-5 | ✅ |
| US-6 | deprecated `--session-dir` is inert — `--from` governs (BUG-493) | AC-6 | ✅ |
| US-7 | Source session files not modified after cross-loaded run | AC-7 | ✅ |
| US-8 | Bare invocation (neither flag) is a no-op; ordinary `-c` still applies | AC-8 | ✅ |

---

### US-1: Clone outward — transplant planned from source into target storage

- **Given:** source dir `/tmp/us28-project-a` has a non-empty `.jsonl` session file (UUID stem `abc-123`); target dir is a fresh temp directory; `CLAUDE_HOME` set to an isolated temp dir
- **When:** `clr --to <tgt> --from /tmp/us28-project-a --dry-run "Continue"`
- **Then:** dry-run output includes the full plan line `# session-transplant: <source storage>/abc-123.jsonl -> <claude_home>/projects/<Df(canonical target)>` and `cd <tgt>` (subprocess working directory is target)
- **Exit:** 0
- **Verifies:** AC-1
- **Implemented by:** `session_from_test.rs::us1_clone_outward_continue_injected`

---

### US-2: Inject inward — runs in CWD, transplant planned into CWD storage

- **Given:** source dir `/tmp/us28-project-b-inward` has a non-empty `.jsonl` session file (UUID stem `def-456`); `CLAUDE_HOME` set to an isolated temp dir; no `--to` given
- **When:** `clr --from /tmp/us28-project-b-inward --dry-run "What did you do in B?"`
- **Then:** dry-run output includes the full plan line `# session-transplant: <source storage>/def-456.jsonl -> <claude_home>/projects/<Df(canonical CWD)>` (destination = CWD's own storage); no `cd /tmp/us28-project-b-inward` appears (subprocess stays in CWD)
- **Exit:** 0
- **Verifies:** AC-2
- **Implemented by:** `session_from_test.rs::us2_inject_inward_cwd_unchanged`

---

### US-3: No source history — fresh session

- **Given:** source dir `/tmp/us28-empty-source` exists but contains no qualifying `.jsonl` files
- **When:** `clr --from /tmp/us28-empty-source --dry-run "Start fresh"`
- **Then:** dry-run output does NOT include `-c "` and does NOT include a `# session-transplant:` plan line; subprocess starts a fresh session
- **Exit:** 0
- **Verifies:** AC-3
- **Implemented by:** `session_from_test.rs::us3_no_source_history_fresh_session`

---

### US-4: `--to` alone defaults `--from` to CWD

- **Given:** CWD (canonicalized) has a non-empty `.jsonl` session file (UUID stem `us4-cwd-src`); target dir is a fresh temp directory; `CLAUDE_HOME` set to an isolated temp dir; no `--from` given
- **When:** `clr --to <tgt> --dry-run "Continue"`
- **Then:** dry-run output includes the full plan line `# session-transplant: <cwd storage>/us4-cwd-src.jsonl -> <claude_home>/projects/<Df(canonical target)>` — `--from` implicitly defaulted to CWD and cloned outward exactly as an explicit `--from <cwd>` would; `cd <tgt>` present (subprocess working directory is target)
- **Exit:** 0
- **Verifies:** AC-4
- **Implemented by:** `session_from_test.rs::us4_to_alone_defaults_from_to_cwd`

---

### US-5: `--to` alias

- **Given:** source dir `/tmp/us28-proj-a-to` has session `abc-123.jsonl`; target dir is a fresh temp directory
- **When:** `clr --to <tgt> --from /tmp/us28-proj-a-to --dry-run "test"`
- **Then:** dry-run subprocess working directory is `<tgt>` (not CWD)
- **Exit:** 0
- **Verifies:** AC-5
- **Implemented by:** `session_from_test.rs::us5_to_alias_sets_working_dir`

---

### US-6: deprecated `--session-dir` is inert — `--from` governs (BUG-493)

- **Given:** source dir `/tmp/us28-proj-a-prec` storage holds session `abc-123.jsonl` (under a temp `CLAUDE_HOME`); a raw override dir holds session `xyz-789.jsonl`
- **When:** `clr --from /tmp/us28-proj-a-prec --session-dir <override dir> --dry-run "test"`
- **Then:** dry-run output does NOT include `CLAUDE_CODE_SESSION_DIR=` (Fix(BUG-493) removed the last export); the transplant plan line `# session-transplant: <src storage>/abc-123.jsonl -> ` appears — `--from`'s computed source storage governs exactly as if `--session-dir` were absent
- **Exit:** 0
- **Verifies:** AC-6
- **Implemented by:** `session_from_test.rs::us6_session_dir_inert_from_governs`

---

### US-7: Source session files not modified

- **Given:** source dir `/tmp/us28-proj-a-immutable` has session `abc-123.jsonl` with recorded mtime and size; target dir is a fresh temp directory
- **When:** `clr --to <tgt> --from /tmp/us28-proj-a-immutable --dry-run "Continue"`; run completes
- **Then:** `abc-123.jsonl` mtime is unchanged; file size is unchanged
- **Exit:** 0
- **Verifies:** AC-7
- **Implemented by:** `session_from_test.rs::us7_source_session_files_not_modified`

---

### US-8: Bare invocation is a no-op

- **Given:** CWD (canonicalized) has a non-empty `.jsonl` session file (UUID stem `us8-bare-cwd`); `CLAUDE_HOME` set to an isolated temp dir; neither `--from` nor `--to` given
- **When:** `clr --dry-run "Continue"`
- **Then:** dry-run output does NOT include a `# session-transplant:` plan line (both source and target default to CWD, so the self-copy guard suppresses the transplant); output DOES include `-c "` — ordinary continuation still detects the existing CWD session independently of cross-loading
- **Exit:** 0
- **Verifies:** AC-8
- **Implemented by:** `session_from_test.rs::us8_bare_invocation_neither_flag_is_noop`
