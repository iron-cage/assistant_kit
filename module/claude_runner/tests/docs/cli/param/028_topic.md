# Parameter :: `--topic`

Edge case coverage for the `--topic` parameter. See [028_topic.md](../../../../docs/cli/param/028_topic.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Default (no `--topic`) → effective dir equals `--dir` base | Behavioral Divergence |
| EC-2 | `--topic NAME` → fresh name plans a fork-mode topic session named NAME | Behavioral Divergence |
| EC-3 | `--topic .` (explicit identity) → effective dir equals `--dir` base | Edge Case |
| EC-4 | `--help` output contains `--topic` | Documentation |
| EC-5 | `--topic NAME` + `--dir PATH` → PATH is the topic's base | Interaction |
| EC-6 | `CLR_TOPIC=NAME` env var → topic session named NAME (CLI absent) | Env Var |
| EC-7 | `--topic NAME` CLI wins over `CLR_TOPIC=OTHER` env var | CLI-wins |
| EC-8 | `--topic ""` (empty string) → identity; no `/-` suffix in dry-run output | Edge Case |
| EC-9 | `--topic "a/b"` (slash in name) → exit 1; error mentions no separators | Validation |
| EC-10 | `--dry-run --topic NAME` → exit 0; no directory created on filesystem | Side-effect |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Edge Case: 2 tests (EC-3, EC-8)
- Documentation: 1 test (EC-4)
- Interaction: 1 test (EC-5)
- Env Var: 1 test (EC-6)
- CLI-wins: 1 test (EC-7)
- Validation: 1 test (EC-9)
- Side-effect: 1 test (EC-10)

**Total:** 10 edge cases

---

### EC-1: Default (no --topic) → no /-NAME suffix

- **Given:** No `--topic` flag; no `CLR_TOPIC` env var
- **When:** `clr --dry-run "task"`
- **Then:** Dry-run output contains no `/-` path component; no named topic is appended
- **Exit:** 0
- **Source:** [--topic](../../../../docs/cli/param/028_topic.md)
- **Commands:** run, ask

---

### EC-2: --topic NAME → fresh name plans a fork-mode topic session

- **Given:** No prior `--dir`; cwd is base; no pre-existing `-build` dir (fresh name → fork mode)
- **When:** `clr --topic build --dry-run "task"`
- **Then:** Dry-run output contains `topic=build ` in the `# topic-fork:`/`# topic-resume:` preview line; no `/-build` directory path appears
- **Exit:** 0
- **Source:** [--topic](../../../../docs/cli/param/028_topic.md)
- **Commands:** run, ask

---

### EC-3: --topic . (explicit identity) → no /-NAME suffix

- **Given:** No prior `--dir`; cwd is base
- **When:** `clr --topic . --dry-run "task"`
- **Then:** Dry-run output contains no `/-` path component; identity (`.`) is a no-op — same output as bare `clr --dry-run "task"`
- **Exit:** 0
- **Source:** [--topic](../../../../docs/cli/param/028_topic.md)
- **Commands:** run, ask

---

### EC-4: --help lists --topic

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--topic`
- **Exit:** 0
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md)
- **Commands:** run, ask

---

### EC-5: --topic NAME + --dir PATH → PATH is the topic's base

- **Given:** `--dir /tmp/project` and `--topic debug`; no pre-existing `/tmp/project/-debug` dir
- **When:** `clr --dir /tmp/project --topic debug --dry-run "task"`
- **Then:** Dry-run preview shows `topic=debug ` and `base=/tmp/project` — the base composes from `--dir`, not cwd (dir mode would show `/tmp/project/-debug` instead)
- **Exit:** 0
- **Source:** [--topic](../../../../docs/cli/param/028_topic.md)
- **Commands:** run, ask

---

### EC-6: CLR_TOPIC=NAME env var → topic session named NAME

- **Given:** `CLR_TOPIC=feature` set; no `--topic` CLI flag
- **When:** `CLR_TOPIC=feature clr --dry-run "task"`
- **Then:** Dry-run output contains `topic=feature ` in the preview line
- **Exit:** 0
- **Source:** [--topic](../../../../docs/cli/param/028_topic.md)
- **Commands:** run, ask

---

### EC-7: --topic CLI wins over CLR_TOPIC env var

- **Given:** `CLR_TOPIC=envname` set; `--topic cliname` on CLI
- **When:** `CLR_TOPIC=envname clr --topic cliname --dry-run "task"`
- **Then:** Dry-run output contains `topic=cliname `, NOT `topic=envname`
- **Exit:** 0
- **Source:** [--topic](../../../../docs/cli/param/028_topic.md)
- **Commands:** run, ask

---

### EC-8: --topic "" (empty string) → identity

- **Given:** No prior `--dir`; cwd is base
- **When:** `clr --topic "" --dry-run "task"`
- **Then:** Dry-run output contains no `/-` path component; empty string is treated as identity — same output as bare `clr --dry-run "task"` (Fix: BUG-229)
- **Exit:** 0
- **Source:** [--topic](../../../../docs/cli/param/028_topic.md)
- **Commands:** run, ask

---

### EC-9: --topic "a/b" (slash in name) → rejected with error

- **Given:** clean environment
- **When:** `clr --topic "a/b" "task"`
- **Then:** Exit 1; stderr contains error message about no `/` separators; no directory is created (Fix: BUG-230)
- **Exit:** 1
- **Source:** [--topic](../../../../docs/cli/param/028_topic.md)
- **Commands:** run, ask

---

### EC-10: --dry-run --topic NAME → no filesystem side effects

- **Given:** clean directory (no pre-existing `/-NAME` topic); cwd is base
- **When:** `clr --dry-run --topic sidecheck "task"`
- **Then:** Exit 0; dry-run output shows the `topic=sidecheck` fork preview; no `cwd/-sidecheck` directory and no topics-registry entry exist after the invocation (Fix: BUG-231; fork-mode side-effect coverage: F12 in [088_topic_mode.md](088_topic_mode.md))
- **Exit:** 0
- **Source:** [--topic](../../../../docs/cli/param/028_topic.md)
- **Commands:** run, ask
