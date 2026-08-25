# Parameter :: `out`

Edge case tests for the `out` parameter — `.chart`'s SVG destination. Tests
validate the default path, the override, and the fact that `out` and `output`
are different parameters on different commands rather than two spellings of one.

**Source:** [param/29_out.md](../../../../docs/cli/param/29_out.md)
**Related:** [param/23_output.md](../../../../docs/cli/param/23_output.md), [invariant/003_cli_surface_consistency.md](../../invariant/003_cli_surface_consistency.md)

## Test Case Index

| ID | Test Name | Category | Status | Implementation |
|----|-----------|----------|--------|----------------|
| EC-1 | Absent -> writes `usage.svg` into the current directory | Default | ✅ | `viewer_integration_test.rs::ec14_chart_default_writes_usage_svg_in_cwd` |
| EC-2 | `out::PATH` -> writes there instead | Parsing | ✅ | `viewer_integration_test.rs::ec15_chart_custom_out_path` |
| EC-3 | `out::` survives a failed `open::1` — the file is written either way | Interaction | ✅ | `viewer_integration_test.rs::ec16_chart_open_failure_is_non_fatal` |
| EC-4 | `output::` on `.chart` and `out::` on `.export` both exit 1 | Non-Interchangeable | ⏳ | — |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)
- Interaction: 1 test (EC-3)
- Non-Interchangeable: 1 test (EC-4)

**Total:** 4 edge cases

## Architectural Constraint

**`out` is relative to the process's working directory, not the journal
directory.** EC-1 therefore runs `clj` with an explicit `current_dir` set to a
second temp directory, separate from the journal it reads. Asserting on
`usage.svg` inside the journal dir would pass against a build that resolved the
path either way, and fail to notice the one that matters — the user's cwd.

**`out` and `output` are not the same parameter and never overlap.** `.chart`
takes `out` (an SVG destination, defaulted); `.export` takes `output` (a
required file path, no stdout fallback). Neither command accepts the other's
spelling. EC-4 pins that, because the two names are close enough to be mistyped
in either direction and the failure would otherwise be silent — a rejected
`output::` on `.chart` would have written `usage.svg` somewhere unexpected while
the caller believed it had named the file.

**`out` had no page at all until `tests/cli_doc_consistency.rs` demanded one.**
It was accepted by the binary and documented on `.chart`'s command page from the
day the command landed, which is exactly the gap DC-1 closes: a parameter named
by a command page must have a page of its own.

## Test Cases

---

### EC-1: Absent -> writes `usage.svg` into the current directory

- **Given:** a journal directory with events, and a *separate* empty directory used as the process's cwd
- **When:** `clj .chart journal_dir::<journal>` is run with its cwd set to the empty directory
- **Then:** exit 0; `usage.svg` exists in the cwd, starts with `<svg`, and stdout reports `Chart written`
- **And:** stdout contains no warning — no `open::` was requested, so nothing should be reported as having failed to open
- **Exit:** 0
- **Source:** [param/29_out.md](../../../../docs/cli/param/29_out.md)

---

### EC-2: `out::PATH` -> writes there instead

- **Given:** journal with events; a target path in a temp directory that does not yet exist
- **When:** `clj .chart out::/tmp/.../custom.svg`
- **Then:** exit 0; the named file exists
- **Exit:** 0
- **Source:** [param/29_out.md](../../../../docs/cli/param/29_out.md)

---

### EC-3: `out::` survives a failed `open::1`

- **Given:** journal with events; a container with no browser and no `xdg-open`
- **When:** `clj .chart out::<path> open::1`
- **Then:** exit **0** and the SVG exists — a browser that cannot be opened is a warning, not a failure, because the artifact the command exists to produce was produced
- **Exit:** 0
- **Note:** `open::1`, not `open::true`. `true` is no longer an accepted Boolean and now exits 1, which would make this case pass for the wrong reason — the file would be absent *and* the assertion would never run
- **Source:** [param/29_out.md](../../../../docs/cli/param/29_out.md), [param/17_open.md](../../../../docs/cli/param/17_open.md)

---

### EC-4: `output::` on `.chart` and `out::` on `.export` both exit 1

- **Given:** any journal directory, and a path that must not be created
- **When:** `clj .chart output::/tmp/should_not_exist.svg`, then `clj .export out::/tmp/should_not_exist.jsonl`
- **Then:** both exit 1 with `unknown parameter`, and **neither path exists afterwards** — rejection happens before the command body, so a mistyped destination cannot half-run
- **Exit:** 1 both times
- **Note:** the non-existence assertion is the load-bearing half. Exit 1 alone would also be produced by a command that wrote the file and then failed, which is the outcome this case exists to rule out
- **Source:** [param/29_out.md](../../../../docs/cli/param/29_out.md), [param/23_output.md](../../../../docs/cli/param/23_output.md)
