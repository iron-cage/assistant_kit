# docs/

Documentation for the `assistant` workspace covering behavioral requirements, structural patterns, integration contracts, and invariants.

## Responsibility Table

| File/Directory | Responsibility |
|----------------|----------------|
| `claude_code/` | Shared Claude Code knowledge (filesystem, settings format, storage layout) |
| `feature/` | Workspace design and crate inventory |
| `invariant/` | Privacy, versioning, testing, and performance constraints |
| `pattern/` | Four-layer crate dependency architecture pattern |
| `integration/` | Cross-workspace integration protocol |
| `error/` | Claude Code error message catalog |
| `runbox/` | Variability analysis: runbox infrastructure parameters and plugins |
| `doc_graph.yml` | Cross-reference graph for all doc instances |
