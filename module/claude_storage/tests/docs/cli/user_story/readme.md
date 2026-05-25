# User Story Test Index

Acceptance criteria test plans for `docs/cli/user_story/`.
Mirror of [user_story/](../../../../docs/cli/user_story/readme.md).

### Scope

- **Purpose**: Verify acceptance criteria for all user story personas.
- **Responsibility**: RWS-N acceptance test plans per user story.
- **In Scope**: All 5 user stories, acceptance criteria verification.
- **Out of Scope**: Parameter edge cases (→ `param/`), command integration (→ `command/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `procedure.md` | Test execution procedure for user story acceptance tests |
| `01_audit_session_history.md` | Acceptance tests for Audit Session History (developer) |
| `02_find_past_conversation.md` | Acceptance tests for Find Past Conversation (developer) |
| `03_export_session_for_review.md` | Acceptance tests for Export Session for Review (developer) |
| `04_query_storage_programmatically.md` | Acceptance tests for Query Storage Programmatically (developer) |
| `05_resume_claude_session.md` | Acceptance tests for Resume Claude Session (developer) |

### Test ID Convention

| Prefix | Category | Used In |
|--------|----------|---------|
| `RWS-N` | Real-world scenario | User story acceptance tests (`user_story/`) |

### Aggregate Counts

| File | Tests |
|------|-------|
| `01_audit_session_history.md` | 5 |
| `02_find_past_conversation.md` | 5 |
| `03_export_session_for_review.md` | 5 |
| `04_query_storage_programmatically.md` | 5 |
| `05_resume_claude_session.md` | 5 |
| **Total** | **25** |

### Related Documentation

- [user_story/](../../../../docs/cli/user_story/readme.md) — Source user stories with acceptance criteria
- [command/](../command/) — Integration tests per command
- [param/](../param/) — Edge case tests per parameter
