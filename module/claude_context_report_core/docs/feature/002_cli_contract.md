# Feature: CLI Contract

### Scope

- **Purpose**: Fix the command surface a consuming binary exposes for printing context reports, so the surface is designed once here rather than re-invented per consumer.
- **Responsibility**: Define the `context` subcommand, its arguments, defaults, exit codes, and the delegation boundary between the binary and this crate.
- **In Scope**: Subcommand name and placement, argument list with defaults, output format selection, redaction level selection, exit codes, worked invocations.
- **Out of Scope**: Table structure (→ [`../format/001_context_report_tables.md`](../format/001_context_report_tables.md)); redaction semantics (→ [`../invariant/001_no_private_data.md`](../invariant/001_no_private_data.md)); the report model (→ [001_context_report.md](001_context_report.md)); the consuming binary's other subcommands (→ `claude_runner/docs/cli/`).

### Abstract

The command prints the three tables and nothing else. Every classification decision — order, weight, force, path attribution, redaction — happens in this crate; the binary parses arguments, calls one function, and styles the result. That split is what makes the tables *exact*: two consumers that render the same model can differ in colour and border glyphs, never in rows.

### Placement

`context` joins the existing `clr` subcommand set (`run`, `ask`, `isolated`, `refresh`, `help`, `ps`, `kill`, `tools`, `scope`, `query`, `topic`, `topics`, `daemon`, `chat`, `sessions`).

It belongs on `clr` rather than `clg` (storage CLI) because the subject is a *running session's* context, which is the runner's domain — `clg` explores the store as a database, and already has `ps`/`sessions` as its neighbours here.

**Pitfall:** adding a subcommand requires updating both `KNOWN_SUBCOMMANDS` and the dispatch match — the crate already carries a comment recording that a prior mismatch made a subcommand parse as a prompt.

### Command Surface

```
clr context [--session <id>] [--dir <path>]
            [--table blocks|paths|layers|corrections|all]
            [--format md|text|json]
            [--redact strict|paths|off]
            [--bands <spec>]
```

| Argument | Default | Meaning |
|----------|---------|---------|
| `--session <id>` | current session | Session to report on. Without it, resolves the session for `--dir` |
| `--dir <path>` | process cwd | Working directory whose session is reported |
| `--table <name>` | `all` | Which table(s) to print. Repeatable |
| `--format <fmt>` | `text` on a terminal, `md` otherwise | Output rendering |
| `--redact <level>` | `off` on a terminal, `strict` otherwise | Path disclosure level |
| `--bands <spec>` | renderer default | Weight band boundaries, echoed into the legend |

**Both defaults are terminal-sensitive, and for the same reason.** An interactive terminal is a local, private sink: absolute paths help and styled text is readable. A pipe or a file is a sharing sink: the output is being captured to paste somewhere, so it defaults to Markdown and to `strict`. This makes the safe case the automatic one — a user who pipes a report into a paste buffer gets a redacted report without having asked for it, and a user who wants raw paths on a terminal already has them.

### Output Formats

| Format | Shape | For |
|--------|-------|-----|
| `text` | Aligned columns, ANSI styling, glyph weights | Reading in a terminal |
| `md` | GitHub-flavoured Markdown tables | Pasting into an issue or review |
| `json` | The report model, serialised | Machine consumption and diffing |

`json` serialises the model itself, so a consumer diffing two reports diffs typed values rather than rendered text. The legend is a field of the document, not a comment, so band settings travel with the data.

### Exit Codes

| Code | Condition |
|------|-----------|
| 0 | Report produced |
| 1 | Session has no transcript yet — a session spawned moments ago legitimately has none |
| 2 | Transcript exists but could not be read or folded |
| 3 | Arguments invalid — unknown table name, unknown level, unresolvable `--dir` |

Code 1 is distinct from code 2 on purpose: "not yet written" is a normal state a poller retries, while "unreadable" is a fault. The daemon's context path already draws this exact line with a dedicated no-transcript error, and the CLI surfaces it rather than flattening both into a generic failure.

### Delegation Boundary

| Step | Owner |
|------|-------|
| Parse arguments, resolve `--dir` and `--session` | binary |
| Locate the transcript | binary |
| Fold, classify, weight, attribute, redact | **this crate** |
| Style the model into `text` / `md` / `json` | binary |

The binary must not classify. If a consumer finds itself deciding a `Force` value or a weight band, the decision belongs here instead — that is the boundary breaking, and the tables stop being exact.

### Worked Invocations

```bash
# Read the current session's context in a terminal — absolute paths, styled
clr context

# Capture for an issue — Markdown, paths tokenised, no flags needed
clr context > report.md

# Just the path table, for a specific session
clr context --session <id> --table paths

# Machine-diff two reports
clr context --format json > before.json
# ... work happens ...
clr context --format json > after.json
```

**Verification.** `clr context --format json | <json-tool> '.blocks | length'` prints the block-row count, and `clr context --table blocks | grep -c '^|'` counts rendered rows; the two must agree once the header and separator rows are discounted. A disagreement means the renderer dropped or invented a row, which is precisely the failure the model/render split exists to prevent.

### Counterpart Commands

The report is read-only by design; there is no write counterpart, and none is planned. The nearest neighbours already exist and are not duplicated here:

| Need | Command |
|------|---------|
| List live sessions | `clr ps`, `clr sessions` |
| Inspect the store as a database | `clg` |
| Ask the daemon for a wire-shaped summary | daemon `ContextSummary` request |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| feature | [readme.md](readme.md) | Feature collection master index |
| feature | [001_context_report.md](001_context_report.md) | The model this command prints |
| format | [`../format/001_context_report_tables.md`](../format/001_context_report_tables.md) | Table structure and vocabularies |
| invariant | [`../invariant/001_no_private_data.md`](../invariant/001_no_private_data.md) | What `--redact` may and may not relax |
