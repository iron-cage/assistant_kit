# Format: Context Report Tables

### Scope

- **Purpose**: Specify the exact table shape a context report renders, so that any two runs against the same session produce byte-identical structure and a reader can rely on column meaning without consulting the producer.
- **Responsibility**: Authoritative instance for the three report tables — their columns, cell vocabularies, ordering rule, and the glyph scale used for relative size.
- **In Scope**: Block Table, Path Table, Layer Table; the `Src`/`Force`/`Kind`/`State` enumerations; the weight glyph scale; ordering and stability rules; placeholder tokens used in place of host-specific values.
- **Out of Scope**: What must never appear in a rendered report (→ [`../invariant/001_no_private_data.md`](../invariant/001_no_private_data.md)); how the underlying state is folded from a session log (→ `claude_storage_core/docs/data_structure/004_session_context_state.md`); the CLI surface that prints these tables (→ [`../feature/002_cli_contract.md`](../feature/002_cli_contract.md)).

### Abstract

A context report answers one question — *what is in this session's context, in what order, and how much of it is each part* — as three tables. The tables are the format: a consumer diffs two reports by diffing rows, so column order, row order, and cell vocabulary are all contract, not presentation.

Every value that identifies a machine, an account, or a filesystem owner is a **placeholder token** in this specification, and is subject to redaction policy at render time. This document contains no real path, host, account, or session identifier.

### Placeholder Tokens

This specification and every example in it use these tokens. A renderer substitutes real values only when redaction policy permits (→ [`../invariant/001_no_private_data.md`](../invariant/001_no_private_data.md)).

| Token | Stands for |
|-------|------------|
| `{home}` | The account's home directory |
| `{repo}` | Absolute path of the repository root under inspection |
| `{cwd}` | Absolute path of the session's working directory |
| `{project-id}` | Encoded project directory name under the session store |
| `{session-id}` | Session UUID |
| `{scratch}` | Absolute path of the session scratch directory |
| `{user-email}` | Account email address |
| `{branch}` | Git branch name |

### Table 1 — Block Table

One row per context block, in wire order. This is the report's spine; the other two tables reference it by row number.

| Column | Type | Required | Meaning |
|--------|------|----------|---------|
| `#` | integer | ✅ | 1-based position in wire order. Dense, no gaps, never reordered between runs |
| `Block` | string | ✅ | Block label. Message rows are prefixed `M{n} · {role}`; sub-blocks of a compound message are indented with `├`/`└` |
| `Src` | enum | ✅ | Origin channel — see `Src` vocabulary |
| `Carries` | string | ✅ | One-line summary of the block's content. Never the content itself |
| `Wt` | glyph | ✅ | Relative size on the weight scale |
| `Force` | enum | ✅ | How the block constrains behaviour — see `Force` vocabulary |

**`Src` vocabulary** — closed set:

| Value | Origin |
|-------|--------|
| `sys` | Harness-authored system prompt |
| `cfg` | Injected configuration (instruction files, rulebooks) |
| `usr` | User turn |
| `ast` | Assistant turn |
| `fn` | Tool-result or tool-catalog injection |
| `rem` | System reminder |

**`Force` vocabulary** — closed set:

| Value | Meaning |
|-------|---------|
| `rule` | Constrains behaviour; must be obeyed |
| `data` | Factual state; no behavioural demand |
| `info` | Advisory; may be disregarded |
| `catalog` | An inventory of available capabilities |
| `conditional` | A rule that activates only in a named situation |
| `evidence` | Retrieved material supporting a claim |
| `history` | A prior turn, retained for continuity |
| `dormant` | Loaded but explicitly inert this turn |
| `callable` | An invocable interface |
| `live` | The turn currently being answered |

**Ordering rule.** Rows follow wire order — the order the model receives the blocks — never alphabetical, never grouped by `Src`. A compound message emits one parent row plus one row per sub-block; sub-block rows immediately follow their parent and inherit its position.

### Table 2 — Path Table

Every filesystem path named anywhere in Table 1, one row per path.

| Column | Type | Required | Meaning |
|--------|------|----------|---------|
| `Row` | integer | ✅ | Table 1 row this path was named in. **Must resolve to an existing Table 1 row** |
| `Kind` | enum | ✅ | `file` or `dir` |
| `Path` | string | ✅ | Absolute path, fully written. Never elided, never prefix-abbreviated |
| `State` | enum | ✅ | Relationship between the path and the context — see `State` vocabulary |

**`State` vocabulary** — closed set, mutually exclusive:

| Glyph | Value | Meaning |
|-------|-------|---------|
| 🟢 | `loaded` | The file's content is present in context |
| 🟡 | `evicted` | Content was loaded earlier this session and has since been dropped |
| ⚪ | `named` | The path appears in context; content was never loaded |
| ❌ | `absent` | The path appears in context but does not exist on disk |

**Ordering rule.** Sort by `Row` ascending, then by `Path` ascending. This is the sole ordering; semantic grouping is forbidden because the table is a lookup keyed on `Row`.

**Two hard rules, both learned from defects:**

1. **`Row` is a row number, never a category.** A path that cannot be attributed to a Table 1 row does not belong in this table. Introducing a pseudo-value such as `pending` to hold unattributed paths breaks the join and silently converts the column into free text.
2. **Paths are absolute and unabbreviated.** Collapsing a shared prefix to save column width defeats the table's only purpose. If width is a problem, the renderer wraps; it does not elide.

### Table 3 — Layer Table

Aggregate rollup. One row per layer.

| Column | Type | Required | Meaning |
|--------|------|----------|---------|
| `Layer` | string | ✅ | Layer name |
| `Rows` | range list | ✅ | Table 1 rows belonging to this layer |
| `Share` | glyph + percent | ✅ | Layer's share of total weight |
| `Mutable` | enum | ✅ | `yes` / `no`, with a short reason |

Layers partition Table 1 exactly: every row belongs to exactly one layer, and the union of all `Rows` ranges equals the full row set.

### Weight Scale

Relative size, five positions, filled-then-empty:

| Glyph | Band |
|-------|------|
| `●○○○○` | smallest |
| `●●○○○` | small |
| `●●●○○` | medium |
| `●●●●○` | large |
| `●●●●●` | largest |

**The scale is relative and must be labelled as an estimate.** Band boundaries are a renderer parameter, not part of this format; a report states its bands in a legend so two reports with different bands are not silently compared. A renderer that can measure exactly emits exact figures alongside the glyphs rather than replacing them — the glyph column exists for scanning, the figures for arithmetic.

### Legend Requirement

Every rendered report emits a legend before the first table, declaring: the weight bands in use, the `Src` vocabulary, and the `State` glyph mapping. A report without a legend is not a conforming report — the glyph columns are unreadable without it, and a reader cannot tell an estimate from a measurement.

### Corrections Table

When rendering discovers that a context claim is contradicted by the filesystem — a path asserted present that does not exist, a stated fact the disk refutes — those discrepancies render as a fourth, **conditional** table. It is emitted only when non-empty.

| Column | Type | Meaning |
|--------|------|---------|
| `#` | integer | Sequential within this report |
| `Claim` | string | What the context asserts |
| `Row` | integer | Table 1 row making the claim |
| `Reality` | string | What was observed |
| `Impact` | string | What breaks if the claim is trusted |

A correction is only ever recorded from a direct observation made during this run. A discrepancy inferred from another part of the context, rather than checked, is not a correction and must not appear here.

### Stability Contract

Two runs against the same session state produce identical tables. Specifically:

- Row numbering is a pure function of wire order.
- Cell vocabularies are closed sets; a renderer encountering an unmodelled value emits it verbatim and counts it, rather than mapping it onto a neighbouring value.
- Path ordering is total (`Row`, then `Path`), so no tie is resolved by chance.

### Example

Structure only — every host-specific value is a placeholder token.

```
Legend: Wt ●○ relative, bands {…} · Src sys|cfg|usr|ast|fn|rem · State 🟢loaded 🟡evicted ⚪named ❌absent

Table 1 — Blocks
| # | Block            | Src | Carries                             | Wt    | Force |
|---|------------------|-----|-------------------------------------|-------|-------|
| 1 | Tool schemas     | sys | 12 callable tools                   | ●●●●● | callable |
| 2 | Environment      | sys | cwd, platform, model                | ●●○○○ | data  |
| 3 | M1 · user        | usr | resume-after-compact                | ●●●●● | mixed |
| 4 | ├ instruction file | cfg | project rules                     | ●●●●● | rule  |
| 5 | └ summary        | usr | digest of prior thread              | ●●●●○ | history |

Table 2 — Paths
| Row | Kind | Path                                  | State |
|-----|------|---------------------------------------|-------|
| 2   | dir  | {cwd}                                 | 🟢    |
| 4   | file | {home}/.claude/CLAUDE.md              | 🟢    |
| 5   | file | {repo}/module/example_core/src/lib.rs | ⚪    |

Table 3 — Layers
| Layer   | Rows | Share      | Mutable |
|---------|------|------------|---------|
| Harness | 1–2  | ●●●●○ ~35% | no      |
| History | 3–5  | ●●●●● ~65% | yes     |
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| format | [readme.md](readme.md) | Format collection master index |
| invariant | [`../invariant/001_no_private_data.md`](../invariant/001_no_private_data.md) | What must never appear in a rendered report |
| feature | [`../feature/001_context_report.md`](../feature/001_context_report.md) | The report model this format renders |
| feature | [`../feature/002_cli_contract.md`](../feature/002_cli_contract.md) | CLI surface a consuming binary must expose |
| data_structure | `claude_storage_core/docs/data_structure/004_session_context_state.md` | Folded session state feeding the report |
| contract | `contract/claude_code/docs/readme.md` | Index of the session-log event taxonomy the fold consumes — linked at the collection index rather than at an instance, because the envelope/attachment/system collections are being restructured and instance paths are not yet stable |
