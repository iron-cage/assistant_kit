# docs

### Scope

- **Purpose**: All behavioral requirements and knowledge for `claude_profile`.
- **Responsibility**: Feature and invariant doc instances, CLI design reference, and cross-reference graph.
- **In Scope**: Functional requirements (feature/), non-functional constraints (invariant/), CLI reference (cli/), entity index, doc graph.
- **Out of Scope**: Test implementations (→ `tests/`), test planning docs (→ `tests/docs/`), source code (→ `src/`).

| File | Responsibility |
|------|----------------|
| cli/ | CLI design documentation (commands, params, types) |
| feature/ | Functional requirement doc instances (feature/001–069; IDs 041–060 unassigned) |
| invariant/ | Non-functional constraint doc instances (invariant/001 through invariant/009) |
| algorithm/ | Decision algorithm doc instances |
| state_machine/ | Domain lifecycle state machine doc instances |
| schema/ | On-disk file format schema doc instances |
| subprocess/ | Subprocess layer API contract doc instances |
| pitfall/ | Recurring design pitfall and trap doc instances |
| entity/ | Master index of all doc entities and instances |
| doc_graph.yml | Cross-reference graph for feature/ and invariant/ doc instances |
| research_interactive/ | Investigation findings on Claude binary behavior and interaction modes |
