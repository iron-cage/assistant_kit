# User Story :: Hosted Session Chat

Test case spec for [032_hosted_session_chat.md](../../../../docs/cli/user_story/032_hosted_session_chat.md).

## How this story is covered

Uniquely among the stories here, part of it cannot be automated — and the reason is
specific rather than general. The promise is about the behaviour of a *real* Claude Code:
its first-run prompts, its input handling, the transcript it writes. A fake `claude` shim
has none of that, and one that echoed its input would pass every end-to-end case below
while proving nothing.

So each case names where its evidence actually comes from, and there are three kinds:

| Kind | Where | What it proves |
|------|-------|----------------|
| `*_test.rs` in this crate | `cargo nextest` | The argument surface, and what `clr` decides on its own |
| `*_test.rs` in `claude_daemon_core` | `cargo nextest` | The socket, the lock, and the session table — real PTY children, no mocks |
| `MD-N` | `tests/manual/readme.md` | The end-to-end round trip through a real model call |

A ✅ against a manual case means the case is written, executable in one command, and has
been run — not that it runs in CI. Which cases are manual and why is argued in
`tests/manual/readme.md`; the short version is that everything *under* the round trip is
automated, and the round trip itself is the part a shim cannot stand in for.

## Test Case Index

| ID | Test Name | AC | Status |
|----|-----------|-----|-----|
| US-1 | The answer is printed, not the terminal it arrived on | AC-1 | ✅ |
| US-2 | A second chat in the same directory continues the first | AC-2 | ✅ |
| US-3 | The first chat starts the daemon and the session itself | AC-3 | ✅ |
| US-4 | The session is still hosted when the command returns | AC-4 | ✅ |
| US-5 | A session is addressable by conversation id and by directory | AC-5 | ✅ |
| US-6 | A prompt of any length is delivered, not left in the input box | AC-6 | ✅ |
| US-7 | Asking what is hosted does not start anything | AC-7 | ✅ |
| US-8 | `status` / `start` / `stop` describe the state on return | AC-8 | ✅ |
| US-9 | A second daemon loses the lock race and exits quietly | AC-9 | ✅ |
| US-10 | The daemon is in a process group of its own | AC-10 | ✅ |
| US-11 | `--raw` still reaches the terminal bytes | AC-11 | ✅ |
| US-12 | An answer ended by `--timeout` still exits 0 | AC-12 | ✅ |
| US-13 | An argument error starts nothing and leaves nothing | AC-13 | ✅ |
| US-14 | Stopping the daemon takes its sessions with it | AC-14 | ✅ |

---

### US-1: The answer is printed, not the terminal it arrived on

- **Given:** a directory with no session in it yet
- **When:** `clr chat "Reply with exactly one word and nothing else: pineapple" | cat -A`
- **Then:** stdout is exactly `pineapple$` — no box rules, no `❯` input line, no status bar, no spinner frames. The `Starting a session in <dir> …` note is on stderr, so the pipe carries the answer alone
- **Exit:** 0
- **Verifies:** AC-1
- **Implemented by:** `tests/manual/readme.md::MD-1` end to end; `chat_answer_test.rs::ca1_an_assistant_text_block_is_the_answer` and `ca2_thinking_and_tool_blocks_are_not_the_answer` for the reading rule underneath it

---

### US-2: A second chat in the same directory continues the first

- **Given:** a session that has already answered one prompt in this directory
- **When:** `clr chat "What single word did I ask you to reply with a moment ago? Answer with just that word."`
- **Then:** the answer is that word — only possible if the prompt reached the same session rather than a fresh one
- **Exit:** 0
- **Verifies:** AC-2
- **Implemented by:** `tests/manual/readme.md::MD-2`
- **Rationale:** this is the whole difference between `clr chat` and `clr ask`, and the only reason the daemon exists. It cannot be shown without two real turns against one real session

---

### US-3: The first chat starts the daemon and the session itself

- **Given:** no daemon running, and no session in this directory
- **When:** `clr chat "…"`
- **Then:** the answer arrives without any preceding `clr daemon start`
- **Exit:** 0
- **Verifies:** AC-3
- **Implemented by:** `tests/manual/readme.md::MD-1`
- **Rationale:** the asymmetry with `clr sessions` (US-7) is deliberate, and each half needs its own case: a client asking to talk to a session wants a session, so an extra setup step would be ceremony rather than safety

---

### US-4: The session is still hosted when the command returns

- **Given:** a chat that has just returned
- **When:** `clr sessions`
- **Then:** one row — the conversation id, the pid, `idle`, and this directory
- **Exit:** 0
- **Verifies:** AC-4
- **Implemented by:** `tests/manual/readme.md::MD-2` for a populated listing; `sessions_command_test.rs::sc6_running_daemon_with_no_sessions` and `sc7_running_daemon_json_is_an_empty_array` for the empty one; `claude_daemon_core`'s `serve_test.rs::srv02_list_sessions_starts_empty` and `srv03_spawn_registers_a_session` for the listing itself, against real PTY-attached children

---

### US-5: A session is addressable by conversation id and by directory

- **Given:** a hosted session whose id `clr sessions` has printed
- **When:** `clr chat "carry on" --session <ID>`, and `clr chat "…" --dir <PATH>`
- **Then:** each reaches the named session — `--session` whatever directory it lives in and whether or not it is busy, `--dir` by canonicalised working directory
- **Exit:** 0
- **Verifies:** AC-5
- **Implemented by:** `chat_command_test.rs::ch10_session_flag_reaches_the_daemon` for `--session` reaching the daemon at all, and `ch5_flag_without_value_is_rejected` for the flag's own validation; `tests/manual/readme.md::MD-2` reaches a live session by directory
- **Rationale:** the id is never abbreviated in `clr sessions` output for exactly this reason — a handle you have to retype from memory is not a handle. CH-10 exists because an ignored `--session` is invisible from outside: `chat` would quietly resolve by directory instead, and on a machine with a working `claude` that is not even a failure

---

### US-6: A prompt of any length is delivered, not left in the input box

- **Given:** prompts of 26, 54, 68, 79, 88, and 137 bytes
- **When:** each is sent with `clr chat`
- **Then:** every one is answered; none returns empty and none returns a box rule
- **Exit:** 0
- **Verifies:** AC-6
- **Implemented by:** `tests/manual/readme.md::MD-3`; `claude_daemon_core`'s `serve_test.rs::srv13_the_submitting_return_is_not_sent_with_the_text` for the pause that makes it work
- **Rationale:** the regression this pins failed silently and *by length*. With the text and its submitting return written back to back, everything past about 55 bytes landed in the input box and stayed there, with the next prompt appearing underneath it — no error on either side. srv13 guards the pause mechanically, but the paste heuristic it exists for lives only in a real `claude`

---

### US-7: Asking what is hosted does not start anything

- **Given:** no daemon running
- **When:** `clr sessions`, and `clr sessions --json`
- **Then:** stderr explains that nothing is running, stdout stays empty (or `[]` under `--json`), and no socket exists afterwards
- **Exit:** 0
- **Verifies:** AC-7
- **Implemented by:** `sessions_command_test.rs::sc3_no_daemon_is_not_an_error`, `sc4_no_daemon_json_is_an_empty_array`, `sc5_no_daemon_leaves_stdout_empty`, `sc8_listing_does_not_start_a_daemon`; `tests/manual/readme.md::MD-5`

---

### US-8: `status` / `start` / `stop` describe the state on return

- **Given:** any starting state
- **When:** `start` → `status` → `start` again → `stop` → `status` → `stop` again
- **Then:** each exits on what is true when it returns rather than on what it did, so both transitions are safe to repeat and `clr daemon status || clr daemon start` works
- **Exit:** per subcommand — see `command/13_daemon.md`
- **Verifies:** AC-8
- **Implemented by:** `daemon_command_test.rs::it_09_lifecycle_start_status_stop` (six sequenced scenarios), `it_04_status_without_a_daemon_exits_one`, `it_05_bare_daemon_is_status`

---

### US-9: A second daemon loses the lock race and exits quietly

- **Given:** a daemon already holding `<claude-home>/-daemon/instance.lock`
- **When:** a second `__daemon_serve` tries to take it
- **Then:** the acquire is refused and the second exits 0 without binding anything — whoever started it wanted a daemon running, and there is one
- **Verifies:** AC-9
- **Implemented by:** `claude_daemon_core`'s `lock_test.rs::lock02_second_acquire_is_refused`, `lock03_lock_is_released_on_drop`, `lock07_distinct_paths_lock_independently`; `listener_test.rs::lis04_a_lock_from_elsewhere_is_refused` for the check that a lock held elsewhere proves nothing about this socket
- **Not covered:** the exit-0 arm in `run_daemon_serve` that acts on the refusal. Reaching it needs a genuine start race — `it_09`'s second `start` probes first, finds a daemon answering, and never spawns a child to lose one. What is tested is the refusal it depends on



---

### US-10: The daemon is in a process group of its own

- **Given:** a daemon started by `clr daemon start`
- **When:** its process group is read back from `/proc`
- **Then:** it is its own, not the shell's — so the terminal's `SIGINT`/`SIGQUIT`/`SIGTSTP`, sent to the foreground group, never reach it
- **Verifies:** AC-10
- **Implemented by:** `daemon_command_test.rs::it_10_the_daemon_has_its_own_process_group` (Linux only)
- **Rationale:** detachment is the claim the whole story rests on, and it is made of three things — own process group, immediate parent exit, stdio that never points at a terminal. This is the one of the three that can be read back from outside

---

### US-11: `--raw` still reaches the terminal bytes

- **Given:** a hosted session
- **When:** `clr chat "Say OK." --raw | head -20`
- **Then:** escape sequences, box rules, and the input box — the session's actual terminal bytes
- **Exit:** 0
- **Verifies:** AC-11
- **Implemented by:** `tests/manual/readme.md::MD-4`; `chat_command_test.rs::ch1_help_lists_every_option` for the flag's documented existence
- **Rationale:** this is the contrast that makes US-1 meaningful. The chrome is still there and still reachable; the default simply stops printing it

---

### US-12: An answer ended by `--timeout` still exits 0

- **Given:** a turn that will not finish inside the deadline
- **When:** `clr chat "…" --timeout <SECS>`
- **Then:** what arrived is printed, `Warning: gave up waiting` goes to stderr, and the session keeps working — chatting again prints the rest
- **Exit:** 0
- **Verifies:** AC-12
- **Implemented by:** `chat_command_test.rs::ch4_non_numeric_timeout_is_rejected` for the flag's parsing
- **Rationale:** exiting non-zero here would suggest the work was lost, which it was not. The deadline path itself is not automated — provoking it needs a real turn slow enough to outlast a deadline, which is a model-latency race rather than a test

---

### US-13: An argument error starts nothing and leaves nothing

- **Given:** an empty `HOME`
- **When:** `clr chat hello --loudly`
- **Then:** stderr names `--loudly`, and `$HOME/.claude/-daemon/daemon.sock` does not exist afterwards
- **Exit:** 1
- **Verifies:** AC-13
- **Implemented by:** `chat_command_test.rs::ch7_argument_errors_start_no_daemon`, plus `ch2`–`ch6` for the individual errors; `tests/manual/readme.md::MD-6`
- **Rationale:** `chat` auto-starts a daemon, which is right when there is a chat to have and wrong on the way to reporting a typo. Parsing happens first, which is what makes every other argument case cheap

---

### US-14: Stopping the daemon takes its sessions with it

- **Given:** a running daemon
- **When:** `clr daemon stop`
- **Then:** it acknowledges on the same connection, tears its sessions down *after* replying, and the command returns only once nothing answers the socket
- **Exit:** 0
- **Verifies:** AC-14
- **Implemented by:** `daemon_command_test.rs::it_09_lifecycle_start_status_stop`; `claude_daemon_core`'s `serve_test.rs::srv11_stop_daemon_answers_then_stops` for the reply-then-tear-down ordering and `table_test.rs::tab11_shutdown_ends_a_child_blocked_on_stdin` for a child that will not go quietly
- **Rationale:** `SIGTERM` would tell the sender nothing — not whether it arrived, not whether the sessions came down cleanly, not whether there was a daemon at all
