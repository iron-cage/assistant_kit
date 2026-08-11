# Commands: Provider

Global inference provider selection command.

---

### Command: 21. `.provider.select`

Get, set, or reset the global inference provider in `~/.clr/config.toml`. The selected provider is a single static scalar — never derived or filtered — that constrains account rotation (see [algorithm/004](../../algorithm/004_eligibility_gates.md) Gate 10): rotation only considers accounts whose `inference_provider` matches this value. Without parameters, prints the current provider (`anthropic` when never explicitly set). With `id::`, writes the selection. With `reset::1`, removes the override and reverts to `anthropic`.

-- **Parameters:** [`id::`](../param/064_id.md), [`reset::`](../param/066_reset.md), [`format::`](../param/002_format.md)
-- **Exit:** 0 (success) | 1 (usage: empty `id::`, or `id::` and `reset::1` together) | 2 (runtime: HOME not set)

**Syntax:**

```bash
clp .provider.select                # get
clp .provider.select id::kimi       # set
clp .provider.select reset::1       # reset to anthropic
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `id::` | `string` | *(omit)* | Provider name to select; activates set mode; non-empty required |
| `reset::` | `bool` | `0` | Remove `provider` key from `~/.clr/config.toml`'s user tier; idempotent |
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format (get mode only) |

**Mode dispatch:**

| `id::` | `reset::` | Mode |
|--------|-----------|------|
| absent | `0` (default) | get — read `provider` from `~/.clr/config.toml`'s user tier; default `anthropic` when absent |
| present | `0` (default) | set — validate non-empty, write to `~/.clr/config.toml`'s user tier |
| absent | `1` | reset — remove `provider` key; create or preserve file |
| present | `1` | error — exit 1; stderr: `id:: and reset::1 are mutually exclusive` |

**Algorithm (get, 2 steps):**
1. Read `~/.clr/config.toml`'s user tier via `claude_core::toml_io::get_tiered`; extract `provider` key; treat absence (or missing file) as `"anthropic"` — never `(unset)`, since a global provider always has an effective value
2. Render `"provider.select: VALUE"` in requested `format::` — JSON output always uses the `provider` key

**Algorithm (set, 3 steps):**
1. Validate `id::VALUE` is non-empty — exit 1 on empty with `id:: must be a non-empty provider name` in stderr
2. Create `~/.clr/config.toml`'s parent `.clr` directory if absent; set `provider = VALUE` in the file's user tier via `claude_core::toml_io::set_user_tier` (preserves other keys)
3. Print `"provider.select: VALUE (selected)"` to stdout; exit 0

**Algorithm (reset, 3 steps):**
1. If `~/.clr/config.toml` absent — print `"provider.select: anthropic (reset to default)"` and exit 0 (idempotent)
2. Remove `provider` key via `claude_core::toml_io::remove_user_tier`; preserve all other keys; write back
3. Print `"provider.select: anthropic (reset to default)"` to stdout; exit 0

**Examples:**

```bash
clp .provider.select
# provider.select: anthropic

clp .provider.select id::kimi
# provider.select: kimi (selected)

clp .provider.select reset::1
# provider.select: anthropic (reset to default)

clp .provider.select format::json
# {"provider":"anthropic"}

clp .provider.select id::
# exit 1: id:: must be a non-empty provider name

clp .provider.select id::kimi reset::1
# exit 1: id:: and reset::1 are mutually exclusive
```

**Notes:**
- The selected provider is a global config scalar, not a filter — exactly one provider is active at a time, and only this command changes it. No other command derives or falls back across providers.
- Rotation (`.usage rotate::1` and auto-rotation) is constrained by the selected provider: only accounts whose `inference_provider` field matches the current selection are eligible (Gate 10, [algorithm/004](../../algorithm/004_eligibility_gates.md)).
- Default is `anthropic` — matches the default value new accounts receive when `inference_provider::` is omitted at `.account.save` time (see [param 073](../param/073_inference_provider.md)).
- Backing store (`~/.clr/config.toml`'s `provider` key) is independent of `.model`'s `model`/`effort` keys (`scope::subprocess`, Feature 035) — all are short-form keys in the same tiered flat-TOML file, written and read via the same `claude_core::toml_io` primitive, but never interact.
- `.provider.select` appears in the "Status & info" group of `clp .help`.

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Inference Provider Selection](../../feature/072_inference_provider_selection.md) | Full specification for this command |

### Referenced Algorithms

| # | Algorithm | Role |
|---|-----------|------|
| 1 | [Eligibility Gates](../../algorithm/004_eligibility_gates.md) | Gate 10 — rotation constrained to accounts matching the selected provider |

### Referenced Schema

| # | Schema | Role |
|---|--------|------|
| 1 | [claude_core toml_io (`~/.clr/config.toml`)](../../../../claude_core/docs/api/002_toml_io.md) | Flat-TOML format storing the `provider` selection |

### Referenced Formats

| # | Format | Trigger |
|---|--------|---------|
| 1 | [text](../format/001_text.md) | `format::text` (default) |
| 2 | [json](../format/002_json.md) | `format::json` |
