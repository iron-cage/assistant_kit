# User Story Procedure

### Trigger

Add a new user story when a distinct persona-goal combination emerges that is not covered by existing stories.

### Steps

1. Create `NNN_slug.md` where `NNN` is the next sequential three-digit number and `slug` is a short kebab-case identifier.
2. Fill in: Scope, Persona, Goal, Acceptance Criteria, Referenced Commands, optionally Referenced Parameters and Referenced Formats.
3. Add a row to `user_story/readme.md` User Story Index.
4. Add `### Referenced User Stories` entries in each referenced command section of `001_commands.md`.
5. Add `### Referenced User Stories` entries in each referenced parameter section of `005_params.md`.
6. Add `### Referenced User Stories` entries in each referenced parameter group section of `003_parameter_groups.md`.
7. Add `### Referenced User Stories` entries in any referenced format files under `format/`.
8. Add the new user story node and edges to `../doc_graph.yml`.
