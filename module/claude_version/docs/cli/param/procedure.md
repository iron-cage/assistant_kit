# param/ — Procedure

### Scope

- **Purpose**: Operational steps for managing parameter instance files in this directory.
- **Responsibility**: Procedures for adding, updating, or removing cm parameter documentation.

---

### Procedure :: 1. Add a New Parameter

**Trigger:** A new cm parameter is implemented and requires documentation.

1. Assign the next sequential number (NN) to the new parameter.
2. Create `NN_name.md` with:
   - Summary Block (`-- **Label:** value` lines)
   - Description paragraph
   - Type, Default, Commands, Validation bullet list
   - Code examples
   - Cross-reference sections in alphabetical order: Referenced Commands → Referenced Parameter Groups (if member) → Referenced Types → Referenced User Stories
3. Update the `### All Parameters` index table in `readme.md`.
4. Add the parameter to its group file if it belongs to one (`../param_group/NN_name.md`).
5. Add the parameter to each command file it applies to.
6. Update `../readme.md` Completion Matrix if level changes.

---

### Procedure :: 2. Update an Existing Parameter

**Trigger:** A parameter's type, default, validation, or command applicability changes.

1. Open the relevant `NN_name.md` file.
2. Update the Summary Block, type/default/validation info, and examples.
3. Update cross-reference sections if command applicability changes.
4. Update bidirectional links in affected command/param_group/user_story files.

---

### Procedure :: 3. Remove a Parameter

**Trigger:** A cm parameter is deprecated and removed.

1. Delete the `NN_name.md` file.
2. Remove the parameter row from the `### All Parameters` table in `readme.md`.
3. Remove bidirectional links from all command, param_group, and user_story files.
