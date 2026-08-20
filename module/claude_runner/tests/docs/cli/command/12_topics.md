# Test: `topics`

Integration test planning for the `topics` command. See [command/12_topics.md](../../../../docs/cli/command/12_topics.md) for specification.

`topics` is the read-only counterpart to `topic`: a listing form and a `--path NAME`
resolver form, neither of which spawns a subprocess or creates a directory. Tests focus
on base resolution, what counts as a topic, the resolver's purity, and the cross-check
that the resolver and the runner never disagree.

Every case pins its base at a `tempfile::TempDir` — via `run_cli_in_dir` for the cwd
default, or `CLR_TOPIC_HOME` for `--global` — so no case reads or writes the host's real
`<temp-dir>/clr-topic`.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TP-1 | Two topic dirs under cwd listed, sorted, under a `NAME SESSIONS PATH` header | Listing |
| TP-2 | Base with no topics → `no topics in <base>` on stderr, stdout empty, exit 0 | Listing |
| TP-3 | Plain directories and `-`-prefixed *files* are not topics | Recognition |
| TP-4 | A bare `-` directory is not a topic | Recognition |
| TP-5 | `--dir <base>` lists that base regardless of cwd | Base Resolution |
| TP-6 | `--global` lists `$CLR_TOPIC_HOME` instead of cwd | Base Resolution |
| TP-7 | `--dir` outranks `--global` when both are given | Base Resolution |
| TP-8 | `--path NAME` prints exactly `<base>/-NAME`, one line, exit 0 | Resolver |
| TP-9 | `--path` resolves a non-existent topic and creates nothing | Resolver |
| TP-10 | `--path` honors `--global` | Resolver |
| TP-11 | `--path a/b` → exit 1 (single-name-component guard) | Error Handling |
| TP-12 | Unknown option → exit 1, option named in stderr | Error Handling |
| TP-13 | `--path` with no value → exit 1 | Error Handling |
| TP-14 | `topics help` / `--help` / `-h` → topics help, exit 0 | Help |
| TP-15 | SESSIONS counts real `*.jsonl`; never-entered topic reports 0 | Session Count |
| TP-16 | `topics --path X` == the effective dir `--dry-run --topic X` reports | Determinism |
| TP-17 | `topics` dispatches as a subcommand, never parsed as a run message | Dispatch |

## Test Coverage Summary

- Listing: 2 tests (TP-1, TP-2)
- Recognition: 2 tests (TP-3, TP-4)
- Base Resolution: 3 tests (TP-5, TP-6, TP-7)
- Resolver: 3 tests (TP-8, TP-9, TP-10)
- Error Handling: 3 tests (TP-11, TP-12, TP-13)
- Help: 1 test (TP-14)
- Session Count: 1 test (TP-15)
- Determinism: 1 test (TP-16)
- Dispatch: 1 test (TP-17)

**Total:** 17 tests

**Implemented by:** `tests/topics_command_test.rs`

---

### TP-1: Topics under cwd are listed and sorted

- **Command:** `clr topics`, run in a base holding `-zebra` and `-alpha`
- **Expected behavior:** stdout begins with a header naming `NAME`, `SESSIONS`, and `PATH`; exactly 2 data rows follow, `alpha` first then `zebra`
- **Exit:** 0
- **Source:** [command/12_topics.md](../../../../docs/cli/command/12_topics.md)

---

### TP-2: Empty base reports on stderr and still succeeds

- **Command:** `clr topics`, run in a base with no topic directories
- **Expected behavior:** stdout is empty; stderr contains `no topics in`
- **Exit:** 0 — an empty result is not an error, so the command is safe under `set -e`
- **Source:** [command/12_topics.md](../../../../docs/cli/command/12_topics.md)

---

### TP-3: Non-topic entries are excluded

- **Command:** `clr topics`, run in a base holding `src/` (plain dir), `-not-a-dir.txt` (file), and `-real/`
- **Expected behavior:** exactly 1 data row, `real`; neither `src` nor `not-a-dir` appears
- **Exit:** 0
- **Source:** [command/12_topics.md](../../../../docs/cli/command/12_topics.md)

---

### TP-4: A bare `-` directory is not a topic

- **Command:** `clr topics`, run in a base holding only a directory literally named `-`
- **Expected behavior:** stdout empty — stripping the prefix yields an empty name, which cannot round-trip back through `--topic ""` (identity)
- **Exit:** 0
- **Source:** [command/12_topics.md](../../../../docs/cli/command/12_topics.md)

---

### TP-5: `--dir` selects the base independently of cwd

- **Command:** `clr topics --dir <base>`, run from an unrelated directory
- **Expected behavior:** stdout lists `<base>`'s topics
- **Exit:** 0
- **Source:** [command/12_topics.md](../../../../docs/cli/command/12_topics.md)

---

### TP-6: `--global` lists the global topic home

- **Command:** `CLR_TOPIC_HOME=<home> clr topics --global`, run from a cwd that has its own topic
- **Expected behavior:** stdout lists `<home>`'s topics and none of the cwd's
- **Exit:** 0
- **Source:** [param/087_global.md](../../../../docs/cli/param/087_global.md)

---

### TP-7: `--dir` outranks `--global`

- **Command:** `CLR_TOPIC_HOME=<home> clr topics --global --dir <base>`
- **Expected behavior:** stdout lists `<base>`'s topics and none of `<home>`'s — an explicit path beats a named default
- **Exit:** 0
- **Source:** [param/087_global.md](../../../../docs/cli/param/087_global.md)

---

### TP-8: `--path NAME` prints the resolved path

- **Command:** `clr topics --path auth-refactor`, run in `<base>`
- **Expected behavior:** stdout is exactly one line, `<base>/-auth-refactor`
- **Exit:** 0
- **Source:** [command/12_topics.md](../../../../docs/cli/command/12_topics.md)

---

### TP-9: `--path` is a pure computation

- **Command:** `clr topics --path never-created`, run in an empty `<base>`
- **Expected behavior:** the path is printed; the named directory does not exist afterwards; `<base>` still holds zero entries
- **Exit:** 0
- **Source:** [command/12_topics.md](../../../../docs/cli/command/12_topics.md)

---

### TP-10: `--path` honors `--global`

- **Command:** `CLR_TOPIC_HOME=<home> clr topics --global --path notes`
- **Expected behavior:** stdout is `<home>/-notes` — the property that makes a global topic addressable from any directory in a later shell
- **Exit:** 0
- **Source:** [param/087_global.md](../../../../docs/cli/param/087_global.md)

---

### TP-11: `--path` rejects a value containing `/`

- **Command:** `clr topics --path a/b`
- **Expected behavior:** stderr explains the single-topic-name constraint; nothing on stdout. Mirrors `--topic`'s own BUG-230 guard — a topic name is a directory name, never a path
- **Exit:** 1
- **Source:** [command/12_topics.md](../../../../docs/cli/command/12_topics.md)

---

### TP-12: Unknown option is rejected

- **Command:** `clr topics --not-a-real-flag`
- **Expected behavior:** stderr names the unknown option
- **Exit:** 1
- **Source:** [command/12_topics.md](../../../../docs/cli/command/12_topics.md)

---

### TP-13: `--path` with no value is rejected

- **Command:** `clr topics --path`
- **Expected behavior:** stderr says a value is required — the next token is never silently swallowed and no default is assumed
- **Exit:** 1
- **Source:** [command/12_topics.md](../../../../docs/cli/command/12_topics.md)

---

### TP-14: All three help forms print topics help

- **Command:** `clr topics help`, `clr topics --help`, `clr topics -h`
- **Expected behavior:** each prints topics-specific help mentioning `--path`. The bare positional `help` needs its own intercept (BUG-249 pattern) or it is parsed as an unknown option
- **Exit:** 0 for all three
- **Source:** [command/12_topics.md](../../../../docs/cli/command/12_topics.md)

---

### TP-15: SESSIONS reflects real session storage

- **Command:** `CLAUDE_HOME=<home> clr topics`, run in a base holding `-entered` (with one real `<uuid>.jsonl` seeded in its own encoded storage) and `-virgin` (never entered)
- **Expected behavior:** `entered`'s SESSIONS column reads `1`; `virgin`'s reads `0`
- **Exit:** 0
- **Source:** [command/12_topics.md](../../../../docs/cli/command/12_topics.md)

---

### TP-16: Resolver and runner agree

- **Command:** `CLR_TOPIC_HOME=<home> clr topics --global --path cross-check`, then `CLR_TOPIC_HOME=<home> clr --dry-run --global --topic cross-check "x"`
- **Expected behavior:** the dry-run output contains the exact path the resolver printed. This is the guarantee the command rests on — both sides compute it through `topic_path::topic_dir()`, and this case fails the moment either caller stops
- **Exit:** 0 for both
- **Source:** [user_story/031_topic_discovery.md](../../../../docs/cli/user_story/031_topic_discovery.md) AC-8

---

### TP-17: `topics` is a dispatched subcommand

- **Command:** `clr topics --path x`
- **Expected behavior:** the resolver runs and prints a path ending in `-x` — `topics` is never parsed as a `run` message. Guards against a missing `KNOWN_SUBCOMMANDS` entry
- **Exit:** 0
- **Source:** [invariant/003_command_naming.md](../../../docs/invariant/003_command_naming.md)
