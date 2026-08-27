# Feature: Context Report

### Scope

- **Purpose**: Turn a session's accumulated context state into an ordered, weighted, redacted inventory that a consumer can render as tables without making any classification decisions of its own.
- **Responsibility**: Define this crate's single responsibility, the report model it produces, its layer position, and the boundary against the three neighbouring crates that also touch session context.
- **In Scope**: The report model (`ContextReport` and its row types), block classification, weight assignment, path attribution, layer partitioning, redaction application.
- **Out of Scope**: Folding a session log into state (→ `claude_storage_core`); rendering the model to a terminal or a wire format (→ consumer); the CLI surface (→ [`002_cli_contract.md`](002_cli_contract.md)); table column vocabulary (→ [`../format/001_context_report_tables.md`](../format/001_context_report_tables.md)).

### Abstract

`claude_storage_core::ContextFold` answers *what is in context* as sets and scalars: which tools are deferred, which skills are on offer, how many tokens remain. That is the right shape for a query and the wrong shape for a report — it has no notion of **order**, no notion of **relative size**, and no notion of **which block named which path**.

A report needs all three. This crate adds exactly that layer and nothing else: it consumes `SessionContextState` plus the ordered event stream, and produces a model whose rows are already in wire order, already weighted, already attributed, and already redacted. A consumer renders it; it does not decide anything.

### Single Responsibility

**Producing the context report model.** One sentence, and the crate holds nothing else:

- It does **not** parse session lines — `claude_storage_core` does.
- It does **not** detect credentials — `json_redact` does.
- It does **not** render — the consumer does, from a model that already fixes every value and every ordering.
- It does **not** locate sessions or talk to a daemon — the caller passes a transcript path.

### Report Model

| Type | Holds |
|------|-------|
| `ContextReport` | The whole report: legend, block rows, path rows, layer rows, corrections |
| `BlockRow` | One Table 1 row — position, label, source, summary, weight, force |
| `PathRow` | One Table 2 row — owning block position, kind, path, state |
| `LayerRow` | One Table 3 row — layer name, block range, share, mutability |
| `Correction` | One conditional-table row — claim, owning block, observed reality, impact |
| `Legend` | Weight bands in force for this report, so two reports are comparable |
| `RedactionLevel` | `Strict` / `Paths` / `Off` — see [`../invariant/001_no_private_data.md`](../invariant/001_no_private_data.md) |

The model is data, not text. Every cell is a typed value or a closed-set enum; no field holds a pre-formatted string. This is what makes "print exact tables" enforceable — two consumers rendering the same model cannot disagree about content, only about styling.

### Layer Position

**Layer 0.** The crate's only workspace dependencies are Layer `*` primitives:

| Dependency | Layer | For |
|------------|-------|-----|
| `claude_storage_core` | `*` | `SessionContextState`, `SessionEvent`, `ContextFold` |
| `json_redact` | `*` | Credential and token detection |

This is the same shape as `claude_session_core`, which is Layer 0 for the same reason: a crate whose only workspace deps are Layer `*` sits alongside `claude_core` rather than in Layer 1.

**Why Layer 0 and not Layer 1.** Both `claude_daemon_core` (Layer 1) and `claude_runner` (Layer 2) are prospective consumers. At Layer 1 the daemon edge would be a same-layer dependency, which the Layer Invariant forbids and `cl1_no_same_layer_deps` fails. At Layer 0 both edges flow downward and no exception is needed.

### Boundary Against Neighbours

Four crates touch session context. The split is by *question answered*, not by data:

| Crate | Question | Output |
|-------|----------|--------|
| `claude_storage_core` | What lines does this session log hold? | `SessionEvent` stream |
| `claude_storage_core` | What state do those lines accumulate to? | `SessionContextState` |
| `claude_daemon_core` | What is in this session's context, over the wire? | JSON projection |
| **this crate** | What is in this session's context, in order, weighted, safe to share? | `ContextReport` model |

**Known tension, and its resolution.** `claude_daemon_core::context::summary` hand-projects `SessionContextState` into JSON field by field. Once this crate exists, that projection and this crate's model are two renderings of one state — the beginning of a divergence. The resolution is directional and deliberate: the daemon's JSON is the **wire** shape and stays owned by the protocol, while this crate's model is the **report** shape. If they are ever unified, unification moves in one direction only — the daemon serialises this crate's model — because the reverse would put report concerns into a wire protocol. Until a second consumer needs the wire shape, unifying is premature.

### Requirements

| # | Requirement |
|---|-------------|
| R1 | Given a transcript path, produce a `ContextReport` whose block rows are in wire order |
| R2 | Assign each block row exactly one `Force` and one `Src` from the closed sets in the format spec |
| R3 | Attribute every path to the block row that named it; a path with no owning row is not emitted |
| R4 | Partition block rows into layers such that the layer ranges cover every row exactly once |
| R5 | Apply the requested `RedactionLevel` to every string field, not only path fields |
| R6 | Emit a `Legend` stating the weight bands used, so two reports are comparable |
| R7 | Count, and report, block kinds the current version does not model, rather than dropping them |
| R8 | Produce identical output for identical input — no ordering resolved by iteration order of a hash container |

R7 mirrors the degradation policy `claude_storage_core` already applies to unmodelled line kinds: a schema that has fallen behind must say so rather than silently under-report.

### Testing

Tests live in this crate's `tests/`, against fixture transcripts committed here — not in a consumer, and not against the live session store. A fixture-backed test is reproducible; a test that reads whatever sessions happen to exist on the developer's machine is not, and the workspace has already been bitten by exactly that (see the non-conversation type contract in `claude_storage/docs/invariant/003_entry_type_format.md`, whose four named values have no reproducible artifact).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| feature | [readme.md](readme.md) | Feature collection master index |
| feature | [002_cli_contract.md](002_cli_contract.md) | CLI surface a consuming binary must expose |
| format | [`../format/001_context_report_tables.md`](../format/001_context_report_tables.md) | Table structure this model renders to |
| invariant | [`../invariant/001_no_private_data.md`](../invariant/001_no_private_data.md) | Redaction guarantee |
| pattern | `docs/pattern/001_crate_layering.md` | Layer hierarchy this crate is placed in |
