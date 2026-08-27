# Invariant: No Private Data In Rendered Reports

### Scope

- **Purpose**: Guarantee that a context report can be pasted into an issue, a review, or a public channel without disclosing credentials, account identity, or host details.
- **Responsibility**: State which classes of value must never leave the renderer unredacted, define the redaction levels, and fix the failure direction when classification is uncertain.
- **In Scope**: Value classes subject to redaction, the three redaction levels and their defaults, the fail-closed rule, delegation to the workspace redaction primitive.
- **Out of Scope**: Table structure and column vocabulary (→ [`../format/001_context_report_tables.md`](../format/001_context_report_tables.md)); the workspace-level dependency privacy constraint, which is a different subject sharing the word "privacy" (→ `docs/invariant/001_privacy_invariant.md`).

### Boundary With The Workspace Privacy Invariant

`docs/invariant/001_privacy_invariant.md` constrains **what this workspace may depend on** — no path dependencies into a private consumer workspace. This invariant constrains **what a report may emit at runtime**. They share a word and nothing else; neither implies the other.

### Invariant Statement

A rendered context report discloses no credential, no account identity, and no host identity, at any redaction level — including the most permissive one.

**Never emitted, at any level:**

| Class | Examples |
|-------|----------|
| Credentials | API keys, OAuth access and refresh tokens, bearer tokens, JWTs, session keys |
| Account identity | Email addresses, account UUIDs, subscription identifiers, usernames |
| Host identity | Hostname, kernel version, OS build, CPU model, MAC and IP addresses, container and VM identifiers |
| Message content | Prompt text, assistant output, tool-result bodies, file contents |

The `Carries` column states *what a block is*, never *what it says*. A report describes the shape of a context; it never reproduces it.

### Redaction Levels

Three levels, differing **only** in how filesystem paths are treated. No level relaxes the table above.

| Level | Paths | Intended use |
|-------|-------|--------------|
| `strict` | Every path replaced by its placeholder token (`{home}`, `{repo}`, `{cwd}`, `{project-id}`, `{session-id}`, `{scratch}`) | Sharing outside the machine — the default when output is not a terminal |
| `paths` | Paths below `{repo}` shown relative to it; everything above `{repo}` tokenised | Sharing within a team that already knows the repository |
| `off` | Absolute paths as-is | Local inspection only — the default when output **is** an interactive terminal |

`off` is a statement about paths and nothing more. Credentials, account identity, host identity, and message content stay redacted at `off`, because the point of the invariant is that no flag combination can produce a leaking report.

### Fail-Closed Rule

When a value's class is uncertain, redact it. A report that over-redacts a benign value is a nuisance; one that emits a token is a disclosure. This matches the doctrine of the workspace redaction primitive, which deliberately errs toward over-redaction and replaces subtrees it cannot fully inspect.

Two consequences worth stating explicitly:

- **Unmodelled values are redacted, not passed through.** A newly-added block kind whose payload the renderer does not recognise is emitted as its kind and count, never its content.
- **Path tokenisation is applied to every string column, not just the path table.** A path mentioned inside a `Carries` summary is subject to the same substitution as one in Table 2 — otherwise the strictest level leaks through the loosest column.

### Delegation

Value-class detection is not reimplemented here. Credential and token detection delegates to the workspace redaction primitive (`json_redact`), which already implements key-atom matching and secret-shaped value patterns and is the single place that logic is maintained. This crate adds only the path-tokenisation layer and the level selection above it.

### Measurement

| Check | Method | Target |
|-------|--------|--------|
| No credential in output | Render a report from a session fixture seeded with token-shaped values; scan output for each seeded value | Occurrences: 0 |
| No account identity | Scan rendered output for the account email and account UUID | Occurrences: 0 |
| No host identity | Scan rendered output for hostname, kernel string, OS build | Occurrences: 0 |
| `strict` tokenises all paths | Render at `strict`; assert no output line matches an absolute-path pattern | Matches: 0 |
| `off` still redacts non-paths | Render at `off` against the seeded fixture; repeat the three scans above | Occurrences: 0 |
| No message content | Assert no `Carries` cell contains a substring of any fixture message body | Occurrences: 0 |

Each check is a test in this crate's own `tests/`, not in a consumer — the behaviour is owned here.

### Violation Consequences

A report is the artifact most likely to be pasted into a bug report, because its entire purpose is to describe a state someone else needs to understand. A leak here is therefore a leak into a public channel by default rather than by accident, and is not recoverable by editing the source afterwards.

### Sources

| File | Relationship |
|------|--------------|
| `module/json_redact/src/lib.rs` | Credential and token detection this invariant delegates to |
| `docs/invariant/001_privacy_invariant.md` | Different subject sharing the word "privacy" — dependency direction, not runtime output |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| invariant | [readme.md](readme.md) | Invariant collection master index |
| format | [`../format/001_context_report_tables.md`](../format/001_context_report_tables.md) | Table structure and placeholder tokens |
| feature | [`../feature/002_cli_contract.md`](../feature/002_cli_contract.md) | Where the redaction level is selected |
