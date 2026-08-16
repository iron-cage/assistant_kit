# User Story :: Session Cross-Loading (Transplant)

Test case spec for [028_session_transplant.md](../../../../docs/cli/user_story/028_session_transplant.md).

## Test Case Index

| ID | Test Name | AC | Status |
|----|-----------|-----|-----|
| US-1 | Clone outward: transplant planned from source into target storage | AC-1 | ✅ |
| US-2 | Inject inward: runs in CWD, transplant planned into CWD storage | AC-2 | ✅ |
| US-3 | No source history → no `-c`; fresh session starts | AC-3 | ✅ |
| US-4 | `--from` alias accepted; behavior identical to `--session-from` | AC-4 | ✅ |
| US-5 | `--to` alias accepted; behavior identical to `--dir` | AC-5 | ✅ |
| US-6 | `--session-dir` takes precedence over `--session-from` | AC-6 | ✅ |
| US-7 | Source session files not modified after cross-loaded run | AC-7 | ✅ |

---

### US-1: Clone outward — transplant planned from source into target storage

- **Given:** source dir `/tmp/project-a` has a non-empty `.jsonl` session file (UUID stem `abc-123`); target dir `/tmp/project-b` is a fresh directory; fake claude binary in PATH
- **When:** `clr --to /tmp/project-b --session-from /tmp/project-a --dry-run "Continue"`
- **Then:** dry-run output includes the full plan line `# session-transplant: <source storage>/abc-123.jsonl -> <claude_home>/projects/<Df(canonical target)>` and `cd /tmp/project-b` (subprocess working directory is target)
- **Exit:** 0
- **Verifies:** AC-1

---

### US-2: Inject inward — runs in CWD, transplant planned into CWD storage

- **Given:** source dir `/tmp/project-b` has a non-empty `.jsonl` session file (UUID stem `def-456`); fake claude binary in PATH; CWD is `/tmp/project-a`
- **When:** `clr --session-from /tmp/project-b --dry-run "What did you do in B?"`
- **Then:** dry-run output includes the full plan line `# session-transplant: <source storage>/def-456.jsonl -> <claude_home>/projects/<Df(canonical CWD)>` (destination = CWD's own storage); no `cd <source>` appears (subprocess stays in CWD)
- **Exit:** 0
- **Verifies:** AC-2

---

### US-3: No source history — fresh session

- **Given:** source dir `/tmp/empty-source` exists but contains no qualifying `.jsonl` files; fake claude binary in PATH
- **When:** `clr --session-from /tmp/empty-source --dry-run "Start fresh"`
- **Then:** dry-run output does NOT include `-c` and does NOT include a `# session-transplant:` plan line; subprocess starts a fresh session
- **Exit:** 0
- **Verifies:** AC-3

---

### US-4: `--from` alias

- **Given:** same setup as US-1
- **When:** `clr --to /tmp/project-b --from /tmp/project-a --dry-run "Continue"`
- **Then:** identical to US-1; plan line `# session-transplant: <source storage>/abc-123.jsonl -> ` present; working dir `/tmp/project-b`
- **Exit:** 0
- **Verifies:** AC-4

---

### US-5: `--to` alias

- **Given:** source dir `/tmp/project-a` has session `abc-123.jsonl`; fake claude binary in PATH
- **When:** `clr --to /tmp/project-b --session-from /tmp/project-a --dry-run "test"`
- **Then:** dry-run subprocess working directory is `/tmp/project-b` (not CWD)
- **Exit:** 0
- **Verifies:** AC-5

---

### US-6: `--session-dir` takes precedence over `--session-from`

- **Given:** source dir `/tmp/project-a` has session `abc-123.jsonl`; raw session storage `/tmp/override-sessions` has session `xyz-789.jsonl`; fake claude in PATH
- **When:** `clr --session-from /tmp/project-a --session-dir /tmp/override-sessions --dry-run "test"`
- **Then:** dry-run output includes `CLAUDE_CODE_SESSION_DIR=/tmp/override-sessions` (raw path — the raw export is BUG-493's own domain and still emitted); the source's computed storage path does NOT appear; no `# session-transplant:` plan line (raw override suppresses the transplant); `--session-dir` wins
- **Exit:** 0
- **Verifies:** AC-6

---

### US-7: Source session files not modified

- **Given:** source dir `/tmp/project-a` has session `abc-123.jsonl` with known mtime T1; fake claude binary in PATH; target dir `/tmp/project-b`
- **When:** `clr --to /tmp/project-b --session-from /tmp/project-a --dry-run "Continue"`; run completes
- **Then:** `abc-123.jsonl` mtime is still T1; file size unchanged; source dir contents identical to before the run
- **Exit:** 0
- **Verifies:** AC-7
