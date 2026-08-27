# Invariant Doc Entity

### Scope

- **Purpose**: State the measurable constraints a context report must satisfy, chiefly that it never discloses credentials, account identity, or host identity.
- **Responsibility**: Index of invariant doc instances for this crate.
- **In Scope**: Runtime output constraints, redaction levels, fail-closed classification, measurement methods.
- **Out of Scope**: Table structure (→ [`../format/`](../format/readme.md)); the report model (→ [`../feature/`](../feature/readme.md)); workspace dependency constraints (→ `docs/invariant/001_privacy_invariant.md`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [No Private Data In Rendered Reports](001_no_private_data.md) | Value classes never emitted, the three redaction levels, fail-closed rule | 🔄 |

### Type-Specific Requirements

All `invariant` doc instances must include:

1. **Title**: `# Invariant: {Constraint Name}`
2. **Scope** (H3): 4 required bullets — Purpose, Responsibility, In Scope, Out of Scope
3. **Invariant Statement** (H3): the constraint, stated as an absolute
4. **Measurement** (H3): a table of Check / Method / Target, each check mechanically runnable
5. **Violation Consequences** (H3): what breaks when the invariant does not hold
6. **Sources** (H3): files the invariant is derived from or enforced against

### Naming Boundary

This crate's invariant 001 and the workspace's `docs/invariant/001_privacy_invariant.md` share the word "privacy" and govern unrelated subjects — runtime output versus dependency direction. Neither implies the other, and each states the boundary explicitly. Any future instance here that touches the same word must do the same.

### Cross-Collection Dependencies

**This collection depends on**:
- `../format/` — the placeholder tokens redaction substitutes

**This collection consumed by**:
- `../feature/002_cli_contract.md` — the `--redact` argument selects among the levels defined here
