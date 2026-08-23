# Parameter :: `live::`

Edge case tests for the `live::` parameter, and for the liveness-derived affordances it shares with `.projects`' renderers — the conditional `STATUS` column and the `detail::sessions` state tag.

**Source:** [param/44_live.md](../../../../docs/cli/param/44_live.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `live::0` lists every project when none is live | Happy Path |
| EC-2 | `live::0` and an unset `live::` agree when nothing is live | Three-State Default |
| EC-3 | `live::1` never lists a project with no attached process | Narrowing |
| EC-4 | Non-boolean value rejected | Invalid Value |
| EC-5 | Out-of-range boolean value rejected | Invalid Value |
| EC-6 | `STATUS` column absent when no row is live | Conditional Rendering |
| EC-7 | `detail::sessions` carries no state tag when no row is live | Conditional Rendering |
| EC-8 | `live::` composes with `filter::` | Composition |
| EC-9 | `ids::1 live::0` passes a non-live project through | Scripting Mode |
| EC-10 | `ids::1 live::1` withholds a non-live project's ids | Scripting Mode |
| EC-11 | Tree and flat layouts agree on the state tag | Conditional Rendering |

## Test Coverage Summary

- Happy Path: 1 test (EC-1)
- Three-State Default: 1 test (EC-2)
- Narrowing: 1 test (EC-3)
- Invalid Value: 2 tests (EC-4, EC-5)
- Conditional Rendering: 3 tests (EC-6, EC-7, EC-11)
- Composition: 1 test (EC-8)
- Scripting Mode: 2 tests (EC-9, EC-10)

**Total:** 11 edge cases — `tests/cli_param_live_test.rs`

**Behavioral Divergence Pairs:**
- EC-1 (`live::0`, every non-live project retained) ↔ EC-3 (`live::1`, every non-live project dropped)
- EC-9 (`ids::1 live::0`, ids kept) ↔ EC-10 (`ids::1 live::1`, ids withheld) — the same divergence on the scripting branch, which reaches its answer before the listing path's filter runs and so has to apply `live::` itself

## Why These Assert on the Negative

Liveness is inferred from the real process table (→ [`algorithm/002_session_liveness.md`](../../../../docs/algorithm/002_session_liveness.md)). There is no injection point through the CLI boundary, and no fixture can conjure an attached Claude Code process whose cwd is a freshly-created temp directory. What a black-box test *can* pin is the half of the contract that holds regardless of what runs on the host: a fixture project is never live, so every liveness-derived affordance must be absent for it.

The positive half — the `/proc` walk, the history join, the working/waiting split — is covered by the unit tests in `src/cli/liveness.rs`, which build a real `/proc`-shaped directory tree with real `comm` files and real `cwd` symlinks, and can therefore assert on presence rather than absence.

EC-3 is the one case whose output legitimately differs by host, and it asserts the disjunction rather than picking a branch: with no Claude Code process running anywhere, the command emits the unavailable-detection note; with processes running against unrelated projects, the fixture rows are simply filtered away. Both are correct, and no third shape may appear.

## Test Cases

---

### EC-1: `live::0` lists every project when none is live

- **Commands:** `.projects`
- **Given:** two path-based projects under one temp root, neither of which can have an attached process
- **When:** `clg .projects scope::global live::0` with `HOME` redirected to the temp root so no real history file is consulted
- **Then:** both projects appear — `live::0` is the inverse filter, not a no-op alias for hiding everything
- **Exit:** 0
- **Source:** [param/44_live.md](../../../../docs/cli/param/44_live.md)

---

### EC-2: `live::0` and an unset `live::` agree when nothing is live

- **Commands:** `.projects`
- **Given:** one fixture project; two invocations differing only in the presence of `live::0`
- **When:** `clg .projects scope::global live::0` and `clg .projects scope::global`
- **Then:** stdout is identical once the relative-age column is pinned. Unset is a third state, distinct from `0`: it applies no filter at all. The two coincide only when no project is live — which is exactly the fixture case, making this the strongest available check that unset does not silently default to `0` or `1`
- **Exit:** 0 for both
- **Note:** the comparison runs through a normalizer that replaces the `N<unit> ago` token and collapses whitespace. Ages render relative to now and size the column that holds them, so two spawns straddling a second boundary differ in that column — routine under a loaded parallel run — without differing in anything this case asserts. A raw byte comparison across two spawns is a latent flake, not a stronger assertion
- **Source:** [param/44_live.md](../../../../docs/cli/param/44_live.md)

---

### EC-3: `live::1` never lists a project with no attached process

- **Commands:** `.projects`
- **Given:** one fixture project with no attached process
- **When:** `clg .projects scope::global live::1`
- **Then:** the fixture project is absent, and the output is one of exactly two documented forms — the `No attached Claude Code processes found.` note (nothing running anywhere) or a zero-project listing (processes running elsewhere). Detection reports only positives, so an empty result must never be presented as a bare answer
- **Exit:** 0
- **Source:** [param/44_live.md](../../../../docs/cli/param/44_live.md) § When detection is unavailable

---

### EC-4: Non-boolean value rejected

- **Commands:** `.projects`
- **Given:** an empty temp storage root, neutral cwd
- **When:** `clg .projects live::bogus`
- **Then:** non-empty error on stderr; no listing on stdout — a typo must fail as an argument error before any storage access, never degrade silently into one of the two states
- **Exit:** 1
- **Source:** [param/44_live.md](../../../../docs/cli/param/44_live.md)

---

### EC-5: Out-of-range boolean value rejected

- **Commands:** `.projects`
- **Given:** an empty temp storage root, neutral cwd
- **When:** `clg .projects live::2`
- **Then:** non-empty error on stderr. Separate from EC-4: `2` parses as an integer and fails the range check, where `bogus` fails the parse itself
- **Exit:** 1
- **Source:** [param/44_live.md](../../../../docs/cli/param/44_live.md)

---

### EC-6: `STATUS` column absent when no row is live

- **Commands:** `.projects`
- **Given:** one fixture project, default flat layout
- **When:** `clg .projects scope::global`
- **Then:** `LAST`, `CONV`, `AGENTS`, and `PROJECT` all render; `STATUS` does not. The column is conditional, like `⚠ gone` before it — reserving width for a column empty on every row wastes the terminal line the terse overview exists to conserve, and a blank `STATUS` cell would read as "not running" when it means "not detected"
- **Exit:** 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md) § Session Liveness

---

### EC-7: `detail::sessions` carries no state tag when no row is live

- **Commands:** `.projects`
- **Given:** one fixture project rendered through the session listing
- **When:** `clg .projects scope::global detail::sessions`
- **Then:** the session id renders (proving the listing path ran) and neither `● working` nor `○ waiting` appears. Same contract as EC-6 on a different render path — `format_session_line` rather than the terse table
- **Exit:** 0
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md) § Session Liveness

---

### EC-8: `live::` composes with `filter::`

- **Commands:** `.projects`
- **Given:** two fixture projects with distinguishable names
- **When:** `clg .projects scope::global live::0 filter::ec8-alpha`
- **Then:** only the substring-matching project survives. `live::` is one narrowing among several and must intersect with the others — composition is where a filter added late tends to break, by being applied before the others or by short-circuiting them
- **Exit:** 0
- **Source:** [param/44_live.md](../../../../docs/cli/param/44_live.md)

---

### EC-9: `ids::1 live::0` passes a non-live project through

- **Commands:** `.projects`
- **Given:** one fixture project holding two root conversations
- **When:** `clg .projects project::<path> ids::1 live::0`, and the same with `count::1`
- **Then:** both conversation ids are listed and `count::1` reports `2` — identical to plain `ids::1`, since nothing is live. `ids::` answers before the listing path reaches its filter, so a `live::` not re-applied on this branch is discarded in the one mode whose caller has no rendered output to notice it in
- **Exit:** 0
- **Source:** [param/44_live.md](../../../../docs/cli/param/44_live.md) § With `ids::1`

---

### EC-10: `ids::1 live::1` withholds a non-live project's ids

- **Commands:** `.projects`
- **Given:** one fixture project with no attached process
- **When:** `clg .projects project::<path> ids::1 live::1`
- **Then:** the fixture's ids never appear, and the run takes one of exactly two documented shapes — exit 0 with empty stdout (`count::1` = `0`) when detection is available and saw nothing attached, or exit 1 with the reason on stderr when it could see nothing at all. Which one occurs depends on the host, not the fixture. The distinction matters more here than in a listing: a listing can say "detection unavailable" in prose, but stdout a script parses cannot carry that caveat, so the unanswerable case has to arrive as a failure rather than as an empty answer
- **Exit:** 0 or 1, per the above
- **Source:** [param/44_live.md](../../../../docs/cli/param/44_live.md) § With `ids::1`

---

### EC-11: Tree and flat layouts agree on the state tag

- **Commands:** `.projects`
- **Given:** one fixture project rendered through `detail::sessions` twice, once per layout
- **When:** `clg .projects scope::global detail::sessions` and the same with `show_tree::1`
- **Then:** the session id renders under both (proving both listing paths ran) and neither carries a state tag. `detail::sessions` has two renderers and only one of them originally received the liveness map, so picking a layout silently decided whether "is this one running" was answered at all — a presentation choice must not gate a fact. Pins the two paths to each other; the positive direction is unreachable from a fixture and belongs to `src/cli/liveness.rs`'s unit tests
- **Exit:** 0 for both
- **Source:** [command/07_projects.md](../../../../docs/cli/command/07_projects.md) § Session Liveness
