# Parameter Group :: Runner Control

Interaction tests for Group 2 (Runner Control): `--no-skip-permissions`, `--interactive`,
`--new-session`, `--dir`, `--subdir`, `--max-tokens`, `--session-dir`, `--dry-run`, `--verbosity`,
`--trace`, `--no-ultrathink`, `--no-effort-max`, `--no-chrome`, `--no-persist`,
`--file`, `--strip-fences`, `--keep-claudecode`, `--output-file`, `--expect`,
`--expect-strategy`, `--max-sessions`, `--retry-on-transient`, `--transient-delay`,
`--timeout` (run/ask), `--retry-on-account`, `--account-delay`, `--retry-on-auth`,
`--auth-delay`, `--retry-on-service`, `--service-delay`, `--retry-on-process`, `--process-delay`,
`--retry-on-validation`, `--validation-delay`, `--retry-on-runner`, `--runner-delay`,
`--retry-on-unknown`, `--unknown-delay`, `--retry-override`, `--retry-override-delay`,
`--retry-default`, `--retry-default-delay`, `--output-style`, `--summary-fields`. Tests validate these forty-four flags
coexist without conflict and are consumed by the runner, not forwarded to claude.

**Source:** [param_group/02_runner_control.md](../../../../docs/cli/param_group/02_runner_control.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `--dry-run` + `--no-ultrathink` → both applied in preview | Interaction |
| CC-2 | `--new-session` + `--session-dir` → both accepted | Interaction |
| CC-3 | `--no-skip-permissions` + `--no-effort-max` → both suppressed | Interaction |
| CC-4 | All runner control flags together → no conflict | Combined |
| CC-5 | `--file` + `--strip-fences` + `--keep-claudecode` together → all accepted | Interaction |
| CC-6 | `--dir PATH` + `--subdir NAME` → effective dir is `PATH/-NAME` | Interaction |

## Test Coverage Summary

- Interaction: 5 tests (CC-1, CC-2, CC-3, CC-5, CC-6)
- Combined: 1 test (CC-4)

**Total:** 6 corner cases

## Test Cases
---

### CC-1: `--dry-run` + `--no-ultrathink` → preview shows suppression

- **Given:** clean environment
- **When:** `clr --dry-run --no-ultrathink "Fix bug"`
- **Then:** Preview shows no ultrathink suffix; `--dry-run` prevents execution
- **Exit:** 0
- **Source:** [param_group/02_runner_control.md](../../../../docs/cli/param_group/02_runner_control.md)
- **Commands:** run, ask
---

### CC-2: `--new-session` + `--session-dir` → both accepted

- **Given:** clean environment
- **When:** `clr --dry-run --new-session --session-dir /tmp/sessions "Fix bug"`
- **Then:** Env block contains `CLAUDE_CODE_SESSION_DIR=/tmp/sessions`; no `-c` flag in assembled command
- **Exit:** 0
- **Source:** [param_group/02_runner_control.md](../../../../docs/cli/param_group/02_runner_control.md)
- **Commands:** run, ask
---

### CC-3: `--no-skip-permissions` + `--no-effort-max` → both suppressed

- **Given:** clean environment
- **When:** `clr --dry-run --no-skip-permissions --no-effort-max "Fix bug"`
- **Then:** Assembled command has no `--dangerously-skip-permissions` and no `--effort`
- **Exit:** 0
- **Source:** [param_group/02_runner_control.md](../../../../docs/cli/param_group/02_runner_control.md)
- **Commands:** run, ask
---

### CC-4: All runner control flags together → no conflict

- **Given:** `/tmp/rc_test.txt` exists and is readable; clean environment
- **When:** `clr --dry-run --no-skip-permissions --interactive --new-session --dir /tmp/test --subdir work --max-tokens 100000 --session-dir /tmp/sessions --verbosity 2 --trace --no-ultrathink --no-effort-max --no-chrome --no-persist --file /tmp/rc_test.txt --strip-fences --keep-claudecode --output-file /tmp/rc_out.txt --expect "yes|no" --expect-strategy fail --max-sessions 5 --retry-on-transient 3 --transient-delay 30 --timeout 60 --retry-on-account 1 --account-delay 0 --retry-on-auth 1 --auth-delay 0 --retry-on-service 1 --service-delay 0 --retry-on-process 1 --process-delay 0 --retry-on-validation 2 --validation-delay 0 --retry-on-runner 0 --runner-delay 0 --retry-on-unknown 1 --unknown-delay 0 --retry-override 3 --retry-override-delay 0 --retry-default 2 --retry-default-delay 30 --output-style summary --summary-fields standard "Fix bug"`
- **Then:** Exit 0; all forty-four flags accepted without conflict; command assembled correctly; effective dir contains `/tmp/test/-work`; `--chrome` and `--dangerously-skip-permissions` are absent from assembled command; no unknown-flag error for any runner-control flag
- **Exit:** 0
- **Source:** [param_group/02_runner_control.md](../../../../docs/cli/param_group/02_runner_control.md)
- **Commands:** run, ask

---

### CC-5: `--file` + `--strip-fences` + `--keep-claudecode` → all accepted

- **Given:** `/tmp/cc5_input.txt` exists and is readable; parent env has `CLAUDECODE=1`
- **When:** `clr --dry-run --file /tmp/cc5_input.txt --strip-fences --keep-claudecode "task"`
- **Then:** Exit 0; describe output includes the file path; `CLAUDECODE` is present in the subprocess environment; no conflict between the three flags
- **Exit:** 0
- **Source:** [param_group/02_runner_control.md](../../../../docs/cli/param_group/02_runner_control.md)
- **Commands:** run, ask

---

### CC-6: `--dir PATH` + `--subdir NAME` → effective dir is `PATH/-NAME`

- **Given:** clean environment
- **When:** `clr --dry-run --dir /tmp --subdir build "task"`
- **Then:** Exit 0; dry-run output contains `cd /tmp/-build`; effective dir is `PATH/-NAME` (not `PATH` alone, not `/tmp/build`)
- **Exit:** 0
- **Source:** [param_group/02_runner_control.md](../../../../docs/cli/param_group/02_runner_control.md)
- **Commands:** run, ask
