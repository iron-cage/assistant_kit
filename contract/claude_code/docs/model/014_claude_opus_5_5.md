# Claude Opus 5.5

### Scope

- **Purpose**: Profile for `claude-opus-5-5` — the default Opus model since v2.1.280, what the `"opus"` CLI alias resolves to on first-party auth, and the workspace's built-in `DEFAULT_MODEL`.
- **Responsibility**: Documents this model's API ID, alias resolution, context window, max output, thinking and effort support, fast-mode eligibility, availability, and workspace role.
- **In Scope**: Model ID, alias, `[1m]` suffix form, max output, thinking/effort capabilities, fast-mode status, workspace constant assignment, availability status.
- **Out of Scope**: Cloud platform IDs for Bedrock/Vertex (→ Anthropic docs); fast-mode and batch pricing (→ Anthropic docs); model training details.

### Profile

| Field | Value |
|-------|-------|
| **API ID** | `claude-opus-5-5` |
| **Alias** | `opus` resolves here since v2.1.280 on first-party auth, Bedrock, Vertex and Mantle. The v2.1.283 catalog maps the same alias to `claude-opus-4-6` on Foundry and `claude-opus-4-7` on a gateway |
| **Suffixed form** | `claude-opus-5-5[1m]` — the catalog marks `supports_1m_suffix`, and the binary carries the literal string |
| **Tier** | Opus (high-capability general) |
| **Context Window** | 1M tokens (native) |
| **Max Output** | 128k — catalog `max_output_tokens` default and upper bound are both `128000` |
| **Thinking** | Adaptive — catalog capabilities list `adaptive_thinking` and `rejects_disabled_thinking`; no extended-thinking budget capability |
| **Effort** | `effort`, `xhigh_effort`, `max_effort`, `per_turn_effort`; catalog `default_effort` is `medium` |
| **Fast Mode** | Yes — catalog capability `fast_mode` |
| **Pricing** | $4 / $20 per Mtok, $0.20 per Mtok cache reads, per the v2.1.280 release note |
| **Knowledge Cutoff** | June 2026 (catalog `knowledge_cutoff`) |
| **3P fallback** | `claude-opus-5` (catalog `fallback_3p`) |
| **Status** | Active — **current default Opus** |

Every catalog value above comes from the baked model catalog inside the installed v2.1.283 binary, which is a first-party source. They aren't values carried over from a predecessor profile; see § Verification to re-read them. Latency isn't stated anywhere in that source, so it's left out.

### Since

v2.1.280 — the binary's embedded changelog: *"Added Claude Opus 5.5 (`claude-opus-5-5`), now the default Opus model — 1M context, $4/$20 per Mtok with $0.20/Mtok cache reads"*. This collection's `../version/` docs stop at v2.1.220, so the binary is the citation; the extraction command is in § Verification.

The same release changed the default model on Pro and Team Standard plans from Sonnet to Opus. It also stopped an effort level saved before `/effort` became per-model from applying to newly released models such as Opus 5.5 — they start at their catalog default (`medium`) until a level is picked.

### Workspace Usage

**`DEFAULT_MODEL`** (`module/claude_runner_core/src/isolated.rs`) is the explicit ID `"claude-opus-5-5"`, not the `opus` alias. It is the target of `IsolatedModel::Default` (`clr isolated`) and the level-5 built-in default of `clr run`/`ask`/`topic`. The alias would name a different model per release, per provider (Foundry and gateway above), and per `ANTHROPIC_DEFAULT_OPUS_MODEL` override, so the constant pins the ID and changes only by source edit. See [`012_workspace_defaults.md`](012_workspace_defaults.md) for the role table and update policy.

### Verification

```bash
V=~/.local/share/claude/versions/$(claude --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')

# Release note (provenance for § Since):
grep -ao 'Added Claude Opus 5\.5[^\\]*\\u2014[^\\]*' "$V" | head -1

# Catalog entry (provenance for the Profile table):
grep -ao '{id:"claude-opus-5-5",family:[^]]*][^}]*' "$V" | head -1 \
  | grep -oE 'max_output_tokens:\{[^}]*\}|default_effort:"[a-z]+"|knowledge_cutoff:"[^"]*"|window:[0-9e]+|"fast_mode"|"adaptive_thinking"'
grep -ao '{id:"claude-totally-fake-9",family:' "$V" | wc -l   # → 0 (negative control)

# Alias resolution per provider:
grep -ao 'aliases:{opus:{default:"[^"]*",per_provider:{[^}]*}' "$V" | head -1

# What the alias actually resolves to on your machine:
claude --model opus -p 'reply with only your model id' </dev/null
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master model entity index |
| doc | [012_workspace_defaults.md](012_workspace_defaults.md) | Role-to-model assignment and update policy |
| doc | [013_claude_opus_5.md](013_claude_opus_5.md) | Previous default Opus (v2.1.219–v2.1.279); this model's 3P fallback |
| doc | [../param/157_disable_1m_context.md](../param/157_disable_1m_context.md) | Opting out of the 1M window this model ships with |
| doc | [../param/042_model.md](../param/042_model.md) | `--model` flag and the `[1m]` suffix mechanism |
| source | `module/claude_runner_core/src/isolated.rs` | `DEFAULT_MODEL` constant |
