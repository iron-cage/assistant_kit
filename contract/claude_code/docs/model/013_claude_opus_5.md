# Claude Opus 5

### Scope

- **Purpose**: Profile for `claude-opus-5` — the default Opus model since v2.1.219, and what the `"opus"` CLI alias now resolves to.
- **Responsibility**: Documents this model's API ID, context window, fast-mode eligibility, availability, and workspace role.
- **In Scope**: Model ID, alias, `[1m]` suffix form, fast-mode status, workspace constant assignment, availability status.
- **Out of Scope**: Cloud platform IDs for Bedrock/Vertex (→ Anthropic docs); model training details.

### Profile

| Field | Value |
|-------|-------|
| **API ID** | `claude-opus-5` |
| **Alias** | `opus` (CLI alias resolves here since v2.1.219) |
| **Suffixed form** | `claude-opus-5[1m]` — observed as the reported model ID of a live 1M-context session |
| **Tier** | Opus (high-capability general) |
| **Context Window** | 1M tokens |
| **Fast Mode** | Yes — `/fast` applies to Opus 5 and Opus 4.8 |
| **Pricing (fast mode)** | $10 / $50 per Mtok, per the v2.1.219 release note |
| **Status** | Active — **current default Opus** |

Fields deliberately omitted rather than guessed: max output, extended/adaptive thinking, effort-parameter default, latency, and knowledge/training cutoff. Sibling profiles such as [`003_claude_opus_4_8.md`](003_claude_opus_4_8.md) carry those, but no first-party source cited in this collection states them for Opus 5, and copying a predecessor's values forward is the failure mode this collection exists to avoid.

### Since

v2.1.219 (2026-07-24) — [`../version/115_v2_1_219.md`](../version/115_v2_1_219.md): *"Added Claude Opus 5 (`claude-opus-5`), now the default Opus model — 1M context, fast mode at $10/$50 per Mtok"*

### Workspace Usage

**`ISOLATED_DEFAULT_MODEL`** is the string `"opus"`, a CLI alias — so it resolves to *this* model as of v2.1.219, without any source change. That is the alias working as designed; see [`012_workspace_defaults.md`](012_workspace_defaults.md) for the role table and update policy.

### Related v2.1.219 changes

- **Opus 4.7 removed from fast mode.** `/fast` now covers Opus 5 and Opus 4.8 only.
- **`/model` picker**: the merged Opus row was fixed to display "Opus (1M context)" rather than plain "Opus".
- **`claude-api` skill** now defaults to Opus 5, with a documented migration path from Opus 4.8.

**One unresolved discrepancy, recorded rather than silently resolved.** A live v2.1.220 session's own system prompt states that fast mode "is available on Opus 5/4.8/4.7" — including 4.7, which the v2.1.219 changelog says was removed from fast mode. Both are first-party; they disagree. This collection does not adjudicate: the changelog is dated and specific, the system prompt is live but may carry stale text. Anyone depending on 4.7 fast-mode availability should test it directly rather than trust either source.

### Verification

```bash
V=~/.local/share/claude/versions/2.1.220
for k in claude-opus-5 claude-opus-4-8 claude-totally-fake-9; do
  printf '%-24s %s\n' "$k" "$(grep -ac "$k" "$V")"
done   # → 39, 52, 0 (last is the negative control)

# Provenance for every claim above:
grep -n 'Opus 5' ../version/115_v2_1_219.md

# What the alias actually resolves to on your machine:
claude --model opus -p 'reply with only your model id' </dev/null
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index |
| doc | [012_workspace_defaults.md](012_workspace_defaults.md) | Role-to-model assignment and update policy |
| doc | [003_claude_opus_4_8.md](003_claude_opus_4_8.md) | Previous default Opus; still fast-mode eligible |
| doc | [../param/157_disable_1m_context.md](../param/157_disable_1m_context.md) | Opting out of the 1M window this model ships with |
| doc | [../param/042_model.md](../param/042_model.md) | `--model` flag and the `[1m]` suffix mechanism |
| doc | [../version/115_v2_1_219.md](../version/115_v2_1_219.md) | Release introducing the model |
| source | `module/claude_runner_core/src/isolated.rs` | `ISOLATED_DEFAULT_MODEL` constant |
