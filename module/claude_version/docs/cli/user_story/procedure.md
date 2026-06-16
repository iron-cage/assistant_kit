# User Story Procedure

### Trigger

Add a new user story when a distinct persona-goal combination emerges that is not covered by existing stories.

### Steps

1. Create `NNN_slug.md` where `NNN` is the next sequential three-digit number and `slug` is a short kebab-case identifier.
2. Fill in: Scope, Persona, Goal, Acceptance Criteria, Referenced Commands, optionally Referenced Parameters and Referenced Formats.
3. Add a row to `user_story/readme.md` User Story Index.
4. Add a row in `### Referenced User Stories` in each referenced command's section within the appropriate file under `../command/`.
5. Add a row in `### Referenced User Stories` in each referenced parameter's file under `../param/`.
6. Add a row in `### Referenced User Stories` in each referenced parameter group's file under `../param_group/`.
7. Add a row in `### Referenced User Stories` in each referenced format's file under `../format/`.
8. Add the new user story node and edges to `../doc_graph.yml`.
