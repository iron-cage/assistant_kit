# Test: `.chart`

### Scope

- **Purpose**: Verify `.chart` renders a usage SVG to the right path and fails in the right places.
- **Responsibility**: Test case coverage for all 4 `.chart` parameters — `out`, `open`, and the two global parameters.
- **In Scope**: Default and overridden output path, the empty-journal placeholder, the non-fatal browser open, the missing-journal error, and command discoverability in `.help`.
- **Out of Scope**: SVG rendering internals (→ `claude_journal_charts`), the `out` parameter's own edge cases (→ `../param/29_out.md`), `open`'s (→ `../param/17_open.md`).

Test case planning for [command/09_chart.md](../../../../docs/cli/command/09_chart.md).

## Test Case Index

| ID | Test Name | Category | Status | Implementation |
|----|-----------|----------|--------|----------------|
| IT-1 | No args -> writes `usage.svg` into the cwd and confirms on stdout | Default | ✅ | `viewer_integration_test.rs::ec14_chart_default_writes_usage_svg_in_cwd` |
| IT-2 | `out::PATH` -> writes there instead | Output Path | ✅ | `viewer_integration_test.rs::ec15_chart_custom_out_path` |
| IT-3 | `open::1` -> SVG still written when no browser can be launched | Non-Fatal Open | ✅ | `viewer_integration_test.rs::ec16_chart_open_failure_is_non_fatal` |
| IT-4 | Empty journal -> a valid placeholder SVG, exit 0 | Empty Journal | ✅ | `viewer_integration_test.rs::ec18_chart_empty_journal_produces_placeholder` |
| IT-5 | Missing `journal_dir::` -> exit non-zero and **no file written** | Error Path | ✅ | `viewer_integration_test.rs::ec19_chart_journal_dir_param_resolution_nonexistent_dir_errors` |
| IT-6 | `.help` lists `.chart` | Discoverability | ✅ | `viewer_integration_test.rs::ec17_help_lists_chart` |

## Test Coverage Summary

- Default: 1 test (IT-1)
- Output Path: 1 test (IT-2)
- Non-Fatal Open: 1 test (IT-3)
- Empty Journal: 1 test (IT-4)
- Error Path: 1 test (IT-5)
- Discoverability: 1 test (IT-6)

**Total:** 6 tests (all executable)

## Architectural Constraint

**Two failure modes are distinguished, and the distinction is the whole design.**
A browser that will not open is a *warning* — the SVG was produced, so the
command did its job (IT-3). A journal directory that does not exist is an
*error* — there is nothing to chart, so exit is non-zero and, critically, no
file is written (IT-5). Asserting only the exit code on IT-5 would miss the
case that actually hurts: a stale or empty SVG left behind at the path the
caller named, which a later reader has no way to recognize as a failed run.

**An empty journal is not an error.** IT-4 pins that a journal with no events
still yields a valid `<svg`-rooted document rather than exiting 1 or writing a
zero-byte file. "Nothing happened this week" is a real answer a chart should be
able to show.

**IT-1 asserts on the cwd, not the journal directory.** `out` defaults to
`usage.svg` relative to the process's working directory, which is a different
directory from the journal being read. The case therefore runs `clj` with an
explicit `current_dir` set to a third temp directory — see
[../param/29_out.md](../param/29_out.md) for why asserting inside the journal
dir would pass against the wrong implementation.

---

### IT-1: No args -> writes `usage.svg` into the cwd and confirms on stdout

- **Given:** a journal directory with events, and a separate empty directory used as the process's cwd
- **When:** `clj .chart journal_dir::<journal>` with cwd set to the empty directory
- **Then:** exit 0; `usage.svg` exists in the cwd and starts with `<svg`; stdout reports `Chart written`
- **And:** stdout contains no warning — no `open::` was requested, so nothing should be reported as failing to open
- **Exit:** 0
- **Source:** [command/09_chart.md](../../../../docs/cli/command/09_chart.md), [param/29_out.md](../../../../docs/cli/param/29_out.md)

---

### IT-2: `out::PATH` -> writes there instead

- **Given:** journal with events; a target path in a temp directory
- **When:** `clj .chart out::<path>`
- **Then:** exit 0; the named file exists
- **Exit:** 0
- **Note:** IT-1 covers the complementary half — that the *default* lands in the cwd — so this case does not also assert the absence of `usage.svg`; between the two, a build that ignored `out::` fails IT-2 and a build that ignored the default fails IT-1
- **Source:** [command/09_chart.md](../../../../docs/cli/command/09_chart.md), [param/29_out.md](../../../../docs/cli/param/29_out.md)

---

### IT-3: `open::1` -> SVG still written when no browser can be launched

- **Given:** journal with events; a container with no browser and no `xdg-open`
- **When:** `clj .chart out::<path> open::1`
- **Then:** exit **0** and the SVG exists — the browser launch is a convenience, and its failure is reported without discarding the artifact
- **Exit:** 0
- **Note:** `open::1`, not `open::true` — `true` is no longer an accepted Boolean and exits 1, which would leave the file absent and the real assertion unreached
- **Source:** [command/09_chart.md](../../../../docs/cli/command/09_chart.md), [param/17_open.md](../../../../docs/cli/param/17_open.md)

---

### IT-4: Empty journal -> a valid placeholder SVG, exit 0

- **Given:** a journal directory with no files written to it at all
- **When:** `clj .chart out::<path>`
- **Then:** exit 0; the file exists and starts with `<svg` — a well-formed document, not an empty file and not an error
- **Exit:** 0
- **Source:** [command/09_chart.md](../../../../docs/cli/command/09_chart.md)

---

### IT-5: Missing `journal_dir::` -> exit non-zero and no file written

- **Given:** a path under a temp directory that does not exist
- **When:** `clj .chart journal_dir::<missing> out::<path>`
- **Then:** exit non-zero, **and** the output path does not exist afterwards
- **Exit:** non-zero
- **Note:** the non-existence half is the load-bearing one. A run that wrote an empty or stale chart and *then* failed would satisfy the exit code alone, and leave a file that reads as a successful render
- **Source:** [command/09_chart.md](../../../../docs/cli/command/09_chart.md), [param/21_journal_dir.md](../../../../docs/cli/param/21_journal_dir.md)

---

### IT-6: `.help` lists `.chart`

- **Given:** any journal directory
- **When:** `clj .help`
- **Then:** exit 0; stdout names `.chart` among the commands
- **Exit:** 0
- **Note:** a command absent from `.help` is a command nobody finds. This is cheap to assert and the only thing standing between a working `.chart` and an undiscoverable one
- **Source:** [command/09_chart.md](../../../../docs/cli/command/09_chart.md)
