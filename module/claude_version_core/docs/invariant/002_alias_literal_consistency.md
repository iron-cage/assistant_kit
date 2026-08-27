# Invariant: Alias Literal Consistency

### Scope

- **Purpose**: Keep the pinned `stable` alias literal identical everywhere it is *presented as the alias table*, while allowing unrelated fixture uses of the same string to drift freely.
- **Responsibility**: Name the single source of truth, give the enumeration command, and define the triage rule that separates a must-update site from a must-not-update one.
- **In Scope**: The `stable` entry of `VERSION_ALIASES` (INV-1), mirror sites (INV-2), fixture exemption (INV-3).
- **Out of Scope**: Layering and error-type constraints (→ `001_layer_one_boundary.md`).

### Invariant Statement

| ID | Invariant |
|----|-----------|
| INV-1 | `VERSION_ALIASES` in `src/version.rs` is the single source of truth for the pinned `stable` value |
| INV-2 | Every documentation or test-planning file that *presents the alias table or a resolution example* states the same value as INV-1 |
| INV-3 | A file using the same literal as arbitrary-but-consistent fixture data is **exempt** and must not be mechanically rewritten during a bump |

### Why This Needs an Invariant

The value is a compile-time constant mirrored into prose. Nothing in the type system, and no
test, relates the constant to the markdown that quotes it — so a bump that updates the constant
and half the prose produces documentation that contradicts the binary, silently and
indefinitely. `src/version.rs` carries an in-source maintenance comment listing the known
mirror sites; this document is its specification, and the two must be updated together.

The triage rule is the difficult half. At the time of writing the literal appears in roughly
thirty files, while only nine are genuine mirrors. Blanket search-and-replace is therefore
**wrong** — it rewrites fixture data in unrelated crates (`claude_runner_core`'s type and
parameter tests, lock-state drift fixtures, verbosity-rendering fixtures) that merely needed
*some* stable-looking semver and are decoupled from this table by design.

### Enforcement Mechanism

**Step 1 — read the canonical value:**

```bash
grep -n 'name : "stable"' module/claude_version_core/src/version.rs
```

**Step 2 — enumerate every file carrying that literal** (substitute the value from step 1):

```bash
grep -rl '2\.1\.220' module/ --include='*.md' --include='*.rs'
```

**Step 3 — triage each hit** against the rule below. This step is judgment, not mechanism;
the commands above only produce the candidate list.

| Hit is… | Action |
|---------|--------|
| A doc rendering the alias table, a resolution example, or a walkthrough transcript | **Must update** — it is asserting the alias's value |
| A test-planning spec whose expected value is the alias's value | **Must update** |
| A Rust fixture using the literal as arbitrary sample data | **Leave alone** |
| A Rust test deriving its expectation from `VERSION_ALIASES` programmatically | **Leave alone** — it tracks the constant automatically |

**Step 4 — reconcile the in-source list.** After triage, update the maintenance comment above
`VERSION_ALIASES` in `src/version.rs` so its enumerated locations match what step 3 concluded.
A stale list there is what makes the next bump miss a site.

### Violation Consequences

- **INV-2 violated:** Documentation asserts a version the binary will never resolve to. Because
  no test compares the two, the contradiction survives until a user reports it — and the doc,
  being more discoverable than the constant, is likely to be believed over the code.
- **INV-3 violated:** A blanket replace rewrites fixtures in `claude_runner_core` and
  `claude_version`'s test suite that have no relationship to this table. Where a fixture
  encodes a boundary condition, changing its value can silently weaken or invalidate the test
  without failing it.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [api/002_version_surface.md](../api/002_version_surface.md) | The `VERSION_ALIASES` contract |
| doc | `../../../claude_version/docs/feature/001_version_management.md` | The principal mirror site — the CLI alias table |
| source | `../../src/version.rs` | `VERSION_ALIASES` and the in-source maintenance comment this rule specifies |
| test | `../../tests/version_test.rs` | Alias-resolution tests, which derive expectations from the constant |
