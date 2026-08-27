# claude_context_report_core

Ordered, weighted, redacted inventory of what a Claude Code session holds in context.

**Status:** docs-only planned crate — specification complete, no manifest yet. Not a workspace member.

## What it is for

`claude_storage_core` answers *what is in context* as sets and scalars — which tools are deferred, which skills are on offer, how many tokens remain. That shape suits a query and not a report: it carries no order, no relative size, and no record of which block named which path.

This crate adds exactly that layer. It consumes folded session state plus the ordered event stream and produces a report model whose rows are already in wire order, already weighted, already attributed to a source, and already redacted. A consumer renders it; it decides nothing.

## Single responsibility

Producing the context report model. It does not parse session lines, does not detect credentials, does not render, and does not locate sessions — each of those belongs to a crate that already owns it.

## Layer position

**Layer 0.** Its only workspace dependencies are Layer `*` primitives — `claude_storage_core` for session state and `json_redact` for credential detection — which is the same shape that places `claude_session_core` at Layer 0.

```
Layer 2  claude_runner (clr context)
Layer 1  claude_daemon_core
              ↓
Layer 0  claude_context_report_core
              ↓
Layer *  claude_storage_core · json_redact
```

Layer 0 rather than Layer 1 so that `claude_daemon_core` can consume it without creating a same-layer edge, which the Layer Invariant forbids.

## Output

Three tables, plus a conditional fourth:

| Table | Holds |
|-------|-------|
| Blocks | One row per context block, in wire order — source, summary, weight, force |
| Paths | Every path named in the block table, attributed to its owning row |
| Layers | Aggregate rollup partitioning the block table |
| Corrections | Emitted only when a context claim is contradicted by direct observation |

## Privacy

A rendered report discloses no credential, no account identity, no host identity, and no message content — at any redaction level, including the most permissive. The `--redact` levels differ only in how filesystem paths are treated. Classification fails closed: an unrecognised value is redacted, never passed through.

Output defaults are sink-sensitive: an interactive terminal gets absolute paths and styled text, a pipe or file gets Markdown and full path tokenisation, so a captured report is safe to paste without anyone having asked for it.

## Documentation

See [`docs/readme.md`](docs/readme.md) for the reading order. The format specification is [`docs/format/001_context_report_tables.md`](docs/format/001_context_report_tables.md).
