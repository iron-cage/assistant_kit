# command/ — Procedure

### Scope

- **Purpose**: Operational steps for managing command instance files in this directory.
- **Responsibility**: Procedures for adding, updating, or removing cm command documentation.

---

### Procedure :: 1. Add a New Command

**Trigger:** A new cm command is implemented and requires documentation.

1. Identify the namespace (root, version, processes, settings, or new namespace).
2. Open the appropriate namespace file (`root.md`, `version.md`, `processes.md`, `settings.md`).
3. Add a `### Command :: N. .name` section with:
   - Summary Block (`-- **Label:** value` lines)
   - Description paragraph
   - Syntax code block
   - Parameters table (link each param to `../param/NN_name.md`)
   - Exit Codes
   - Examples (if the command has non-trivial usage)
   - Cross-reference sections in alphabetical order: Referenced Formats → Referenced Parameter Groups → Referenced User Stories
4. Update the `### All Commands` index table in this `readme.md`.
5. Add the command to the param files for each parameter it accepts (`### Referenced Commands` table).
6. Add the command to the user story files it participates in (`### Referenced Commands` table).
7. Add the command to the param_group files for each group it belongs to (`### Referenced Commands` table).
8. Update `../readme.md` Completion Matrix if level changes.

---

### Procedure :: 2. Update an Existing Command

**Trigger:** A command's parameters, behavior, or exit codes change.

1. Open the relevant namespace file.
2. Locate the `### Command :: N. .name` section.
3. Update the Summary Block, Parameters table, and Exit Codes as needed.
4. Update cross-reference sections if group membership changes.
5. Update bidirectional links in affected param/param_group/user_story files.

---

### Procedure :: 3. Remove a Command

**Trigger:** A cm command is deprecated and removed.

1. Remove the `### Command :: N. .name` section from the namespace file.
2. Remove the command row from the `### All Commands` table in `readme.md`.
3. Remove bidirectional links from all param, param_group, and user_story files.
4. Update the `../readme.md` Completion Matrix.
