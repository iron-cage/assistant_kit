# Test: Invariant — CLI Surface Consistency

Test case planning for [invariant/003_cli_surface_consistency.md](../../../docs/invariant/003_cli_surface_consistency.md). Tests validate that the four descriptions of the CLI surface — command pages, parameter pages, `param/readme.md`, and `known_params` in the binary — describe the same parameter-to-command mapping, and that the declared totals are real counts rather than remembered ones.

**Source:** [invariant/003_cli_surface_consistency.md](../../../docs/invariant/003_cli_surface_consistency.md)
**Related:** [cli/param/readme.md](../../../docs/cli/param/readme.md), [cli/command/readme.md](../../../docs/cli/command/readme.md)

## Test Case Index

| ID | Test Name | Category | Status |
|----|-----------|----------|--------|
| DC-1 | Every parameter a command page names has a parameter page | Reachability | ✅ |
| DC-2 | Every live parameter page is named by at least one command page | Reachability | ✅ |
| DC-3 | `param/readme.md` and each parameter page agree on commands | Doc-to-Doc | ✅ |
| DC-4 | Each parameter page agrees with the binary's accepted set | Doc-to-Code | ✅ |
| DC-5 | `param/readme.md`'s two declared totals equal the live page count | Counting | ✅ |

## Test Coverage Summary

- Reachability: 2 tests (DC-1, DC-2)
- Doc-to-Doc: 1 test (DC-3)
- Doc-to-Code: 1 test (DC-4)
- Counting: 1 test (DC-5)

**Total:** 5 invariant test cases (all executable)

## Architectural Constraint

**The code side is read from the binary, not from the source.** `known_params`
is private to `cli_main.rs` and has no library path, so an integration test
cannot call it. Parsing the source for the match arms would pin the docs to a
*transcription* of the code — which drifts the same way the docs do, and worse,
drifts silently in a file nobody reads as documentation. Instead DC-4 runs
`clj <command> zz_probe_unknown_param::1` and reads the `Accepted:` line off
stderr, which is `known_params` verbatim as the user sees it. This has a useful
second effect: if the unknown-parameter rejection path itself regresses, the
probe exits 0 instead of 1 and every DC-4 assertion fails loudly, rather than
the gate quietly comparing against an empty set.

**Rejection runs before the command does any work.** That is what makes the
probe safe to run against all nine commands: `.chart` writes no file, `.serve`
binds no port, `.prune` deletes nothing. The probe is a parse-stage question,
answered and exited before the command body is entered.

**Command pages are held to a weaker rule than the other three sources.** DC-1
and DC-2 require reachability in both directions but never equality: a command
page may enumerate fewer parameters than the command accepts. Every
event-reading command takes all nine filters, and repeating a nine-row table on
five pages would bury the parameters each command is actually about. DC-3 and
DC-4 are the equality gates, and they run over the three sources where equality
is the right rule.

**Tombstones are recognized by their own text, not by a list in the test.**
A page is exempt from DC-2 and from DC-5's count when its `- **Type:**` line
says `not accepted`. Keying off a hardcoded `["28_include_stdout.md"]` would
mean the next retraction needs a test edit to land — so the retraction and the
test would have to be got right together, which is exactly the coupling this
invariant exists to remove.

**The gates were mutation-checked before being trusted.** All five passed on
their first run, which is where a vacuous gate is indistinguishable from a
correct one. Five independent defects were injected — a bogus parameter added
to `.status`'s accepted set, a deleted `verbosity` row, a readme/page
disagreement on `sort`, a page/binary disagreement on `by` with the readme kept
consistent so only DC-4 could see it, and a wrong `(N total)` heading — and
each of the five gates failed for its own injected reason, naming the specific
parameter and the two sources that disagreed.

---

### DC-1: Every parameter a command page names has a parameter page

- **Given:** the nine `docs/cli/command/NN_*.md` pages and the parameter pages in `docs/cli/param/`
- **When:** each command page's `### Parameters` table is read and its first column collected
- **Then:** every name collected has a `docs/cli/param/NN_<name>.md` page
- **And:** the failure message names the command page and the parameter, so the fix location is not a search
- **Note:** this is the direction that caught `out`. `.chart`'s page documented `out::` from the day the command landed; the parameter had no page of its own until this gate demanded one
- **Source:** [invariant/003_cli_surface_consistency.md](../../../docs/invariant/003_cli_surface_consistency.md) Threshold: 0 parameters named by a command page without a page

---

### DC-2: Every live parameter page is named by at least one command page

- **Given:** all `docs/cli/param/NN_*.md` pages except `readme.md`
- **When:** pages whose `- **Type:**` line contains `not accepted` are set aside as tombstones; the rest are matched against the union of every command page's Parameters table
- **Then:** every live page is named by at least one command page
- **And:** a tombstone is not required to be reachable — that is the whole point of the tombstone; it exists for the links pointing at it, not for a command
- **Note:** the reverse of DC-1, and the one that would catch a parameter page written for a command that was never wired up. `include_stdout` is precisely that case, which is why it is a tombstone rather than a deletion
- **Source:** [invariant/003_cli_surface_consistency.md](../../../docs/invariant/003_cli_surface_consistency.md) Threshold: 0 live pages named by no command page

---

### DC-3: `param/readme.md` and each parameter page agree on commands

- **Given:** the `Commands` column of `param/readme.md`'s `### All Parameters` table, and each parameter page's `### Referenced Commands` table
- **When:** both are parsed into sets of command names per parameter
- **Then:** the two sets are equal for every parameter, checked in both directions — a parameter in the readme with no page, and a page with no readme row, both fail
- **And:** the failure message prints both sets side by side (`` `sort`: readme says {".list", ".tail"}, 11_sort.md says {".list"} ``) rather than reporting only that they differ
- **Note:** this is a doc-to-doc gate and would pass happily if both sources were wrong in the same way. DC-4 is what stops that, by holding one of them to the binary
- **Source:** [invariant/003_cli_surface_consistency.md](../../../docs/invariant/003_cli_surface_consistency.md) Threshold: 0 disagreements among (2), (3) and (4)

---

### DC-4: Each parameter page agrees with the binary's accepted set

- **Given:** the built `clj` binary and each live parameter page's `### Referenced Commands` table
- **When:** `clj <command> zz_probe_unknown_param::1` is run once per command, its exit code asserted to be 1, and the `Accepted: ` line parsed off stderr into the real accepted set
- **Then:** for every live parameter, the set of commands accepting it equals the set its page claims — checked in both directions, so a parameter the binary accepts with no page and a page claiming a command that rejects it both fail
- **And:** the probe name is deliberately implausible as a real parameter, so a future parameter cannot collide with it and turn the probe into a successful parse
- **Note:** requires the container — the binary must exist and be runnable, so the case calls `assert_container()` first rather than failing obscurely on a missing `CARGO_BIN_EXE_clj`
- **Note:** this is the gate that found `exit::` documented as `.list`-only when it reaches five commands, and `.tail` accepting `since::`/`limit::` while applying neither
- **Source:** [invariant/003_cli_surface_consistency.md](../../../docs/invariant/003_cli_surface_consistency.md) Code side: read from the binary, not the source

---

### DC-5: `param/readme.md`'s two declared totals equal the live page count

- **Given:** `param/readme.md`'s `### All Parameters (N total)` heading and its `**Total:** N parameters` line
- **When:** both numbers are extracted and compared against the count of live (non-tombstone) parameter pages
- **Then:** all three agree
- **And:** the failure message quotes the heading verbatim and states the real count (`` `### All Parameters (24 total)` but 25 live parameter pages exist ``)
- **Note:** two separately-written numbers describing one set is a standing invitation to update one of them. The count is cheap to derive and was wrong before this gate existed
- **Source:** [invariant/003_cli_surface_consistency.md](../../../docs/invariant/003_cli_surface_consistency.md) Threshold: declared totals equal the live page count
