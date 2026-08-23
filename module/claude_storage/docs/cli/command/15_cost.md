# Command :: 15. `.cost`

### Scope

- **Purpose**: Specify the `.cost` CLI command.
- **Responsibility**: Syntax, parameters, exit codes, and examples for `.cost`.
- **In Scope**: Invocation syntax, accepted parameters, output structure, error conditions, pricing model.
- **Out of Scope**: Parameter definitions (→ `param/`), session family contract (→ [`../../invariant/002_session_family.md`](../../invariant/002_session_family.md)), scan/aggregation internals (→ `claude_storage_core/src/cost.rs`, `claude_storage_core/src/family.rs`).

Implemented in `src/cli/cost.rs`; all scanning and aggregation is delegated to `claude_storage_core::cost` — `cost_report()` (single-pass per-session scan: per-model token attribution, cache-TTL split, compaction count, max context) and `aggregate_reports()` (family fold-in) — plus `claude_storage_core::family::find_family()` (agent discovery across both storage layouts). `src/cli/cost.rs` itself only resolves which conversations to report on, applies pricing, and renders the table. Pricing deliberately lives in the CLI layer, NOT the core engine: token counts are facts of the transcript, while prices change over time and carry an as-of date (`PRICES_AS_OF` in `src/cli/cost.rs`, surfaced verbatim in the output's trailing note) — the same core/CLI division [`.usage`](13_usage.md)/[`.rollup`](14_rollup.md) establish for their own engines.

**Representation Absorption Test** (per [`../command_group/readme.md`](../command_group/readme.md), the mandatory gate before adding any new command name): closest candidates are [`.rollup`](14_rollup.md) (also an aggregated token table) and [`.usage`](13_usage.md) (also per-session usage). Fails both criteria against both: (1) *identical routine* — `cost_routine()` delegates to `claude_storage_core::cost::cost_report()`/`aggregate_reports()` and `family::find_family()`, none of which `rollup_routine()` or `usage_routine()` call; neither existing command has any code path for agent fold-in (agent sessions never contribute a row there — both explicitly exclude them), per-model pricing, cache-TTL split, compaction counting, or conversation-keyed rows, so no change of parameter defaults can reach `.cost`'s output from either routine. (2) *identical parameter set* — `.cost` registers `session_ids::` and `agents::`, which no other command registers, and registers none of `.rollup`'s `group::`/`sort::`/`order::`/`model::`/`columns::`/`scope::`/`depth::`/`limit::`; the only shared parameter is `path::`. Confirmed as a genuinely new command, not a reparameterization of either candidate.

Print a per-conversation cost accounting table: exact token counts (fresh input, output, cache read, cache write), deduplicated request count, context compactions, largest single-call context, and estimated USD cost — with each conversation's agent (subagent) sessions folded into its row by default. Use this to answer "what did this conversation cost?" including the agent work it spawned — the billing-audit counterpart to [`.usage`](13_usage.md)'s activity view and [`.rollup`](14_rollup.md)'s cross-sectional aggregates, both of which exclude agent sessions and price nothing.

**Parameters:** `session_ids::`, `path::`, `agents::`

**Exit:** `0` success | `1` argument error (`agents::` outside `0`/`1`; `session_ids::` splitting to zero IDs) or resolution/read error (an ID matching no session, an ambiguous prefix, an unreadable ROOT session file) | `2` default resolution failure (`session_ids::` omitted and the current directory or `path::` has no project, or the project has no session — the [`.usage`](13_usage.md)/[`.rollup`](14_rollup.md) "not found = usage error" convention)

**Syntax:**
```bash
claude_storage .cost
claude_storage .cost session_ids::feed0011
claude_storage .cost session_ids::aaaa1111,bbbb2222 agents::0
claude_storage .cost path::/data/repos/myproject
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `session_ids::` | String (comma list) | optional | most recent session of the current directory's project | Comma-separated session IDs or unique ID prefixes, searched across ALL projects |
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | cwd | Directory whose project anchors the default resolution when `session_ids::` is omitted |
| `agents::` | Boolean | optional | `1` | `1` folds each conversation's agent sessions into its row; `0` reports the root session alone |

**Conversation semantics:** A row is a *conversation* — a root session plus, at `agents::1`, every agent session in its family per the [Session Family invariant](../../invariant/002_session_family.md): hierarchical-layout agents (every `*.jsonl` under `{uuid}/subagents/` — membership by directory structure, never by `sessionId` field) and flat-layout agents (top-level `agent-*.jsonl` whose FIRST entry's `sessionId` matches the root). Both layouts fold in simultaneously. An unreadable agent session is skipped with a stderr warning and the rest of the family still reports (per-session graceful degradation, the `Fix(BUG-506)` convention); an unreadable ROOT is a hard error, since the user asked for exactly that conversation.

**`session_ids::` semantics:** Each requested ID is resolved against every non-agent session across ALL projects (agent sessions are not directly addressable — they are constituents of their root's row). An exact match wins; otherwise the request is treated as an ID prefix, which must be unique — a prefix matching several distinct sessions is an error listing every match, and a request matching nothing is an error naming the request. A session ID physically present in several project directories (git-worktree-style forked history) resolves to the copy with the greatest entry count — the same `Fix(BUG-528)` tie-break [`.rollup`](14_rollup.md) applies. Duplicate requests for one conversation collapse to the first occurrence; row order follows request order.

**`agents::` semantics:** `1` (default) — the row aggregates the root and every family agent session; the `Agents` column counts the agent files folded in. `0` — the row is the root session alone; `Agents` shows `0`. Deliberately named `agents::`, not [`agent::`](../param/01_agent.md): that existing parameter is a session-type *filter* (main-vs-agent) on listing commands, a different semantic this command must not overload — see [`../param/40_agents.md`](../param/40_agents.md).

**Pricing semantics:** Estimated USD at published API list prices as of the `PRICES_AS_OF` date shown in the output's trailing note (`src/cli/cost.rs`'s `MODEL_RATES`). Rates are matched by case-insensitive substring against each recorded model ID, first match wins (more specific needles precede the generic ones they contain, e.g. `opus-4-1` before `opus-4`). Per model, cost = `input×rate + output×rate + cache_read×rate + 5m_cache_writes×rate + 1h_cache_writes×rate`; cache writes carry TTL-specific multipliers of the input rate (5-minute ≈ 1.25×, 1-hour ≈ 2×), taken from the transcript's `usage.cache_creation` breakdown. A cache write whose TTL the transcript does not break down (older format) is priced at the 5m rate — 5 minutes is the API default TTL. A model with no rate entry contributes `$0.00` and is named in a `note:` footnote — silence would misread as "cost fully covered". Token counts themselves are always exact regardless of pricing coverage.

**Algorithm (7 steps):**
1. Validate arguments before any storage access — `agents::` must be `0`/`1`; `session_ids::`, when given, must split (on `,`, trimmed) to at least one non-empty ID
2. Resolve the conversation set: `session_ids::` given → resolve each request per the semantics above (one walk over all projects' non-agent sessions; per-project read failures skipped gracefully); omitted → the most recent non-agent session of the project owning `path::` (or cwd), exiting `2` when no project or no session exists
3. For each conversation, resolve its family (`find_family()`): root path plus agent paths from both layouts, agent list sorted for determinism
4. Scan the root (`cost_report()`) — hard error if unreadable — and, at `agents::1`, each agent session (unreadable agents warn on stderr and are skipped)
5. Fold the family's reports into one `ConversationUsage` (`aggregate_reports()`): model buckets merge by name in first-appearance order, compactions sum, `max_context_tokens` takes the largest single value, `agent_count` counts agent reports
6. Price each row's models against `MODEL_RATES`, collecting unpriced model names
7. Render: header, one row per conversation in request order, a TOTAL row when more than one row is shown, one `note:` footnote per unpriced model, and the price-date note

**Examples:**
```bash
# Current conversation (most recent session of the cwd's project), agents folded in
claude_storage .cost

# One conversation by unique ID prefix, searched across every project
claude_storage .cost session_ids::feed0011

# Compare two conversations, root sessions only — adds a TOTAL row
claude_storage .cost session_ids::aaaa1111,bbbb2222 agents::0

# Most recent conversation of another project
claude_storage .cost path::/data/repos/myproject
```

**Output** (two conversations selected — single-conversation output omits the TOTAL row):
```
Conversation  Agents      Req           Input          Output          CacheR          CacheW           Total       MaxCtx  Compact        Cost
dddd4444           0        3       1,500,000         300,000       3,000,000         700,000       5,500,000    4,500,000        1       $5.25
aaaa1111           0        1             100              50               0               0             150          100        0       $0.00
TOTAL              0        4       1,500,100         300,050       3,000,000         700,000       5,500,150            —        1       $5.25
note: no pricing for model 'claude-test' — its tokens are excluded from Cost
Cost: estimated at API list prices (2026-08-21); tokens are exact.
```
- `Conversation`: the root session ID's 8-character short form (same `short_id()` convention as [`.usage`](13_usage.md)/[`.rollup`](14_rollup.md), local copy per those files' own precedent)
- `Agents`: agent session files folded into the row (`0` under `agents::0` or when the family has none)
- `Req`: deduplicated API calls — distinct `message.id` values, so one multi-content-block response counts once (`Fix(issue-038)` convention)
- `Input`/`Output`/`CacheR`/`CacheW`/`Total`: exact integers with thousands separators — a deliberate divergence from [`.usage`](13_usage.md)/[`.rollup`](14_rollup.md)'s rounded `N.Nk`/`N.NM` rendering, because a billing-audit table's counts must be cross-checkable against an invoice; `Total = Input + Output + CacheR + CacheW`
- `MaxCtx`: the largest single API call's context size (`input + cache_read + cache_creation` for one deduplicated call — the input side only, never output) across every session in the row
- `Compact`: context compactions (`"type":"system","subtype":"compact_boundary"` entries) summed across the family
- `Cost`: estimated USD, `$N.NN`
- Footnotes: one `note:` line per unpriced model (sorted, deduplicated across rows), then the price-date note — always the last line

**Notes:**
- **`MaxCtx` is per-call, never additive** — the TOTAL row deliberately shows `—` there; summing context high-water marks across conversations would be meaningless.
- **The TOTAL row's `Cost` sums per-row costs before rounding**, so it can differ from the sum of the displayed (independently rounded) row costs by a cent.
- **`<synthetic>` assistant entries are skipped entirely** (no request, no tokens, no model bucket) — they are locally-generated placeholders, not API calls.
- **Compaction detection is parse-based** on the top-level `type`/`subtype` fields — a transcript line merely *mentioning* the marker inside message content never matches, because there the text is an escaped string value, not top-level fields.
- Unreadable/unparseable individual lines are skipped (`Fix(BUG-489)`/`Fix(BUG-508)` per-line graceful degradation), matching every other scanning command.

### Referenced Parameter Groups

*(none — `.cost` is not a Scope Configuration member: its `path::` selects the project for default conversation resolution, like [`.tail`](12_tail.md)'s, not a `scope::` anchor; `.cost` registers no `scope::`)*

### Referenced Parameters

| # | Parameter | Type | Required |
|---|-----------|------|----------|
| 9 | [`path::`](../param/09_path.md) | [`StoragePath`](../type/10_storage_path.md) | optional |
| 39 | [`session_ids::`](../param/39_session_ids.md) | String (comma list) | optional |
| 40 | [`agents::`](../param/40_agents.md) | Boolean | optional |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
