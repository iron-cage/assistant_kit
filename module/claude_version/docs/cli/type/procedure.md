# type/ — Procedure

### Scope

- **Purpose**: Operational steps for managing type instance files in this directory.
- **Responsibility**: Procedures for adding, updating, or removing cm type documentation.

---

### Procedure :: 1. Add a New Type

**Trigger:** A new semantic type is introduced for a cm parameter.

1. Assign the next sequential number (NN) to the new type.
2. Create `NN_type_name.md` with:
   - Summary Block (`-- **Label:** value` lines)
   - Description paragraph
   - Base type, constraints, validation errors
   - Usage table (if applicable: levels, variants, etc.)
   - Code examples
   - Cross-reference sections: Referenced Parameters → Referenced User Stories
3. Update the `### All Types` index table in `readme.md`.
4. Link the type from its parameter file (`### Referenced Types` table).
5. Update `../readme.md` Completion Matrix if level changes.

---

### Procedure :: 2. Update an Existing Type

**Trigger:** A type's constraints, valid values, or validation behavior changes.

1. Open the relevant `NN_type_name.md` file.
2. Update constraints, valid values, validation error messages, and examples.
3. If the type is used in test specs, update `../../tests/docs/cli/type/NN_type_name.md`.

---

### Procedure :: 3. Remove a Type

**Trigger:** A semantic type is retired (its parameter is removed or merged).

1. Delete the `NN_type_name.md` file.
2. Remove the type row from the `### All Types` table in `readme.md`.
3. Remove `### Referenced Types` link from the associated parameter file.
