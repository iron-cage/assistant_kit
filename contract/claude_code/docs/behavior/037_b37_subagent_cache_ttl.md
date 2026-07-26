# Behavior B37: Subagent Cache Isolation and 5-Minute TTL

### Scope

- **Purpose**: Document the prompt-cache behavior of Agent tool subagents — each subagent builds an isolated cache prefix from zero, and its cache entries live on the 5-minute TTL tier while the main conversation's entries live on the 1-hour tier (subscription).
- **Responsibility**: Authoritative record of the subagent-vs-main cache tier asymmetry, its JSONL observability, and its token-cost consequences.
- **In Scope**: Cache prefix isolation at spawn; `ephemeral_5m_input_tokens` vs `ephemeral_1h_input_tokens` tier attribution; TTL reset semantics and the >5-minute-gap re-write consequence; cache pricing multipliers; fork and usage-credit exceptions; corroborating external measurements.
- **Out of Scope**: Context/CLAUDE.md inheritance at spawn (→ [030_b30_subagent_context_inheritance.md](030_b30_subagent_context_inheritance.md)); subagent storage layout (→ [013_b13_subagent_directory.md](013_b13_subagent_directory.md)); runtime process model (→ [027_b27_agent_no_os_process.md](027_b27_agent_no_os_process.md)); usage-object field schema (→ [../jsonl/008_usage_object.md](../jsonl/008_usage_object.md)); the subscription rate-limit weighting formula (undisclosed by Anthropic).

### Behavior

**Statement**: Every Agent tool subagent starts its own conversation with its own system prompt and tool set, and therefore builds its own prompt-cache prefix — zero cache hits on its first API call, regardless of how much of that prefix (system prompt, CLAUDE.md, tool definitions) the parent conversation already holds cached. Additionally, subagent cache writes go to the **5-minute TTL tier** (`cache_creation.ephemeral_5m_input_tokens`) even on a subscription, while the main conversation writes to the **1-hour TTL tier** (`ephemeral_1h_input_tokens`). Both facts are directly observable in session JSONL.

**Status**: ✅ Confirmed | **Certainty**: 95% | **Tier**: UNVERIFIED | **Since**: ≤v2.1.197 | **Evidence**: E67, E68, E69

#### Cache prefix isolation

Official documentation ([code.claude.com/docs/en/prompt-caching](https://code.claude.com/docs/en/prompt-caching), "Subagents and the cache"):

> "A subagent starts its own conversation with its own system prompt and tool set. It builds its own cache, starting with no cache hits on its first call."

Consequences:

- Every spawn re-writes the full assembled prefix (system prompt + CLAUDE.md injection + tool definitions + task prompt) at cache-**write** rates, even though the parent already paid for and holds a near-identical cached prefix.
- Parallel siblings race the cache rather than sharing it: independent measurement (Systima) recorded a sibling spawned 6 seconds after its twin writing its entire 52,022-token prefix with zero cache read.
- The documented exception is the **fork** agent type, which inherits the parent conversation — and with it the parent's cache prefix.

#### TTL tier asymmetry

Same documentation section:

> "Subagents use the five-minute TTL even on a subscription, since the automatic one-hour TTL applies to the main conversation."

Observed in live session JSONL (session `3cf8fab1`, 2026-07-25, v2.1.197 — see E68):

| Context | `ephemeral_5m_input_tokens` | `ephemeral_1h_input_tokens` |
|---------|------------------------------|------------------------------|
| Main conversation (assistant entries) | 0 | 4,908 |
| Subagent `a015a043…` (`isSidechain: true`) | 48,690 | 0 |
| Subagent `a1523dc9…` | 60,811 | 0 |
| Subagent `aa77dc6d…` | 72,407 | 0 |
| …all 13 subagents in the session | 42,884–72,407 each | 0 |

The 13 subagents together wrote ~770k cache-write tokens of prefix material largely duplicating what the main thread already held on the 1-hour tier — while the main thread's own incremental writes for the same period were under 5k tokens.

#### TTL reset semantics and the stall consequence

The cache timer resets only on requests that hit the cache ("Each request that hits the cache resets the timer" — same source). A subagent whose turns are separated by long tool executions therefore silently loses its entire cache whenever the gap exceeds 5 minutes: a test-suite run, a long build, a bounded poll, a slow resume, or a rate-limit backoff each expire the prefix, and the next API call re-writes the whole accumulated conversation at cache-write rates. The main conversation absorbs the same gaps for up to 1 hour.

Cost magnitude, using Anthropic's published cache pricing (reads 0.1x base input; 5-minute writes 1.25x; 1-hour writes 2x — see E69): a 100k-token cached context costs ~10k effective tokens on a warm turn but ~125k effective on a cold write — **12.5x per context token**. A stall-heavy subagent pays the ~125k again after every expired gap, so for workloads with several >5-minute tool executions the TTL term dominates the one-time cold-spawn term by an order of magnitude.

#### Exceptions

- **Fork agents** inherit the parent conversation's cache — no cold prefix, and no separate 5-minute tier observed for the inherited prefix.
- **The main conversation itself drops to the 5-minute TTL** when usage draws on extra usage credits instead of the subscription windows ([code.claude.com/docs/en/costs](https://code.claude.com/docs/en/costs)) — the 1-hour tier is a subscription benefit, not a structural property of the main thread.

#### Corroborating external measurements

- Systima, "The Subagent Tax" (hash-chained token audit, [systima.ai/blog/subagent-tax](https://systima.ai/blog/subagent-tax)): 2.6x–5.9x metered-token amplification for subagent fan-outs vs sequential single-thread execution, depending on model and fan-out width; fan-outs were never faster end-to-end in their runs.
- Anthropic engineering ([anthropic.com/engineering/built-multi-agent-research-system](https://www.anthropic.com/engineering/built-multi-agent-research-system)): agents ≈ 4x chat token consumption; multi-agent systems ≈ 15x chat.
- Anthropic costs documentation: agent teams ≈ 7x the tokens of a comparable single session.
- Upstream anecdote ([anthropics/claude-code#4911](https://github.com/anthropics/claude-code/issues/4911)): 2–3k-token main-thread task consuming 153.8k tokens when routed through a subagent.
- `/usage` on subscription plans attributes a distinct "subagents" category and flags "cache misses" when they exceed ~10% of recent usage — the plan metering is cache-aware, though the exact weighting formula is undisclosed.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E67 | B37 | Doc | Official Claude Code documentation (code.claude.com/docs/en/prompt-caching) | "Subagents and the cache" section | "A subagent starts its own conversation with its own system prompt and tool set. It builds its own cache, starting with no cache hits on its first call." And: "Subagents use the five-minute TTL even on a subscription, since the automatic one-hour TTL applies to the main conversation." Forks are the documented exception — a fork inherits the parent conversation's cache. Timer semantics: "Each request that hits the cache resets the timer." |
| E68 | B37 | Observation | Live session JSONL — session `3cf8fab1` (2026-07-25, v2.1.197) | Main session file vs `subagents/agent-*.jsonl` siblings | Main-conversation assistant entries: `"cache_creation":{"ephemeral_5m_input_tokens":0,"ephemeral_1h_input_tokens":4908}` — 1-hour tier only. All 13 subagent transcripts from the same session (`isSidechain: true`): 5-minute tier only (`ephemeral_1h` = 0), per-agent first-call prefix writes of 42,884–72,407 tokens (769,900 cache-write tokens total for prefixes the parent already held cached). |
| E69 | B37 | Doc | Anthropic platform documentation (docs.anthropic.com — prompt caching pricing) + code.claude.com/docs/en/costs | Pricing multipliers; TTL policy | Cache writes bill 1.25x base input (5-minute TTL) / 2x (1-hour TTL); cache reads bill 0.1x. Costs doc: the 1-hour TTL applies to the main conversation on subscription and drops to 5 minutes when drawing on extra usage credits; `/usage` attributes a distinct "subagents" category and flags "cache misses" when ≥10% of recent usage. |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| behavior | [030_b30_subagent_context_inheritance.md](030_b30_subagent_context_inheritance.md) | What context a subagent receives at spawn (CLAUDE.md yes, parent conversation no) |
| behavior | [013_b13_subagent_directory.md](013_b13_subagent_directory.md) | Where subagent JSONL transcripts live — the storage this behavior's evidence reads |
| behavior | [027_b27_agent_no_os_process.md](027_b27_agent_no_os_process.md) | Subagents as API inference threads, not OS processes |
| jsonl | [../jsonl/008_usage_object.md](../jsonl/008_usage_object.md) | `usage.cache_creation` field schema (`ephemeral_5m_input_tokens` / `ephemeral_1h_input_tokens`) |
| tool | [../tool/007_agent.md](../tool/007_agent.md) | Agent tool — the spawn surface whose cache behavior this documents |
| fault | [../fault/readme.md](../fault/readme.md) | Fault index — Q8 (subagent cache cost trap), F5 (API-key billing misroute) |
