# CLI Surface Consistency

**Status**: Implemented | **Since**: 1.3.0

### Scope

- **Purpose**: Guarantee that every description of which command takes which parameter says the same thing.
- **Responsibility**: Names the four places the CLI surface is described and requires them to agree.
- **In Scope**: Parameter-to-command membership, the existence of a page per parameter, and the declared parameter totals.
- **Out of Scope**: Parameter *semantics* — defaults, value grammars, and per-command behavior notes are prose this invariant does not read (→ `docs/cli/type/`, and the per-parameter pages themselves). Whether a command page enumerates every parameter it accepts (→ Description, below: it deliberately need not).

## Description

The CLI surface is described in four places:

1. each command page's Parameters table (`docs/cli/command/NN_*.md`)
2. each parameter page's Referenced Commands table (`docs/cli/param/NN_*.md`)
3. the `Commands` column of `docs/cli/param/readme.md`
4. `known_params` in `src/cli_main.rs` — the only one the CLI actually obeys

Three of the four are prose, and prose drifts silently. It had: `out` was
accepted by `.chart` and documented on its command page with no parameter page
of its own; `include_stdout` had a parameter page, a type-table entry, a group
membership, two user-story recipes and six planned test cases while not being
accepted at all, so every recipe printing it exited 1; and `param/readme.md`
claimed `exit::` reached only `.list` when it reaches five commands.

(2), (3) and (4) must describe **exactly** the same parameter-to-command
mapping. (1) is deliberately allowed to enumerate *fewer* — every event-reading
command accepts the whole filter vocabulary, and a nine-row table on each page
would bury the parameters that command is actually about — but every parameter
it *does* name must have a page, and every live page must be named by at least
one command page. A page is live unless its own `- **Type:**` line says
`not accepted`, which is how a retracted parameter keeps a tombstone for the
documents still linking to it without being held to the rest of this invariant.

## Measurement

- **Threshold**: 0 disagreements among (2), (3) and (4); 0 parameters named by a command page without a page; 0 live pages named by no command page; declared totals equal the live page count
- **Method**: `cargo nextest run -E 'test(/^dc[0-9]/)'` — five gates, all passing. Each reports the specific parameter and the two sources that disagree, not just a count
- **Code side**: read from the binary, not the source. `clj <command> zz_probe_unknown_param::1` prints `Accepted: …`, which is `known_params` verbatim — so the gate stays honest even though `known_params` is private to the binary, and it fails loudly if the unknown-parameter rejection path itself regresses

Verify by hand without the test harness:

```bash
clj .tail zz::1        # Accepted: … — the real set, straight from known_params
clj .chart zz::1       # journal_dir, no_color, open, out
```

Compare either against that parameter's Referenced Commands table and against
`docs/cli/param/readme.md`'s row for it. All three must name the same commands.

## Sources

- `tests/cli_doc_consistency.rs` — DC-1 through DC-5, the executable form of this invariant
- `src/cli_main.rs` `known_params()` — the authoritative set; `reject_unknown_params()` prints it
- `docs/cli/param/readme.md` — the human-readable authoritative mapping
- `docs/cli/command/readme.md` — why (1) is allowed to be narrower than (4)
