# Command Groups

### Scope

- **Purpose**: Formalize sets of commands that share one implementing function and one parameter set, differing only in default values.
- **Responsibility**: Define command_group membership under a strict identity test — same handler function, same parameter set — distinct from the looser cross-command comparisons in `param_group/` and `docs/feature/037_accounts_usage_param_unification.md`.
- **In Scope**: Group membership, the Representation Absorption Test applied per candidate pair, shared-handler citations, default divergence (when any exists), and cross-references to commands/parameters/tests/user stories.
- **Out of Scope**: Individual parameter semantics (-> `../param/`), looser multi-command comparisons that don't share an identical parameter set (-> `../param_group/readme.md`, `../../feature/037_accounts_usage_param_unification.md`).

Every command in `command/` is evaluated against every other command using the Representation Absorption Test (see Evaluated, Not Qualifying below for the worked tests) before a new command name is ever added — this is a mandatory design gate, not documentation-after-the-fact.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| readme.md | This file — group index, membership rule, and zero-groups rationale |

### All Groups (0 total)

| # | Group | Members | Shared Handler | Divergence |
|---|-------|---------|-----------------|------------|

**Total:** 0 groups. All 14 live `clp` commands registered in `src/registry.rs` (`.credentials.status`, `.accounts`, `.account.limits`, `.account.save`, `.account.use`, `.account.delete`, `.account.relogin`, `.account.renewal`, `.account.inspect`, `.model`, `.models`, `.model.select`, `.paths`, `.usage`) — plus the binary-specific `.` (`dot_routine`, registered inline in `src/cli.rs`) — were evaluated pairwise under the Representation Absorption Test. **Zero pairs qualify.**

This is a structural fact, not a coverage gap: every `reg_cmd()` / `register_with_routine()` call in `src/registry.rs` (and the one inline call in `src/cli.rs`) passes a **distinct** routine function — no two commands are ever wired to the same `Box::new(...)` handler. Criterion (b) of the command_group membership rule (literal same dispatch/handler function) therefore fails for every possible pair before parameter-set comparison (criterion (a)) is even relevant. `.account.rotate`, `.account.assign`, and `.account.unclaim` are documented as DEPRECATED/REMOVED commands (see `../command/001_account.md`) but are **not registered in `src/registry.rs` at all** — they have no live implementing function and so cannot be command_group candidates either.

An honest empty result is a valid, complete outcome for this entity — see Evaluated, Not Qualifying below for the specific near-miss candidates considered and why each was rejected.

### Evaluated, Not Qualifying

| Candidate Pair | Shared Implementation | Why Not a Command Group |
|-----------------|------------------------|---------------------------|
| `.accounts` / `.usage` | `owner_dispatch::owner_batch_clear()`, `owner_dispatch::owner_named_dispatch()`, `owner_dispatch::bool_field_batch_set()`, `owner_dispatch::bool_field_named_dispatch()` (`src/owner_dispatch.rs`) — called from `accounts_routine` (`src/commands/accounts.rs:282,285,302,304`) and from `usage_routine` via `handle_mutation_dispatch()` (`src/usage/api_dispatch.rs:267,270,307,310`) | Different top-level routine functions (`accounts_routine` in `src/commands/accounts.rs:70` vs. `usage_routine` in `src/usage/api.rs`, re-exported from `src/usage/mod.rs:35`) — fails criterion (b) regardless of parameter overlap. Parameter sets also diverge beyond defaults: `.usage` registers 3 params `.accounts` does not (`rotate::`, `who::`, `solo::` — `src/registry.rs:258,260,262`), and `.accounts` registers 14 legacy field-presence params (`current::`, `sub::`, `tier::`, `expires::`, `email::`, `display_name::`, `host::`, `role::`, `billing::`, `model::`, `uuid::`, `capabilities::`, `org_uuid::`, `org_name::`) that `.usage` never registers at all — these are dead `REMOVED_TOGGLE` stubs kept only so `.accounts` can emit a `cols::`-migration error, not live parameters (`src/registry.rs:117-130`). Fails criterion (a) as well. `docs/feature/037_accounts_usage_param_unification.md` line 19 describes these two commands as sharing "an identical parameter interface" — that phrasing is imprecise; the two commands share a large common core (confirmed at the parameter-name level in `../param_group/readme.md`'s Fetch/Sort/Display Control rows) but are not identical, and even a truly identical parameter set would not be sufficient on its own without a shared handler. See `../../feature/037_accounts_usage_param_unification.md` for the documented (looser, and now more precisely qualified) relationship. |
| `.model` / `.models` / `.model.select` | None — no shared lower-level helper found beyond the generic `format::` output-formatting convention | Three distinct routine functions (`model_routine`, `models_routine`, `model_select_routine`, `src/registry.rs:192,199,206`) with three non-overlapping parameter sets beyond `format::` (`.model`: `set::`; `.models`: `offline::`, `name::`; `.model.select`: `id::`, `reset::`). Different config surfaces entirely — `.model` writes `~/.claude/settings.json`, `.model.select` writes `~/.clr/config.toml`, `.models` reads a catalog/API and writes nothing. Fails both criteria; not a near miss, listed for completeness given the shared `model` namespace. |

### Navigation

*(none — no qualifying groups)*
