# Type Tests — Procedure

### Scope

- **Purpose**: Workflow for creating and maintaining type test spec files.
- **Responsibility**: Steps for adding, updating, or removing type test specs.

---

### Procedure :: 1. Add a New Type Spec

**Trigger:** A new semantic type is added to `docs/cli/type/` and requires test coverage.

1. Assign the next sequential 2-digit number (NN) matching the source type file.
2. Create `NN_type_name.md` following the type spec format:
   - Scope section (source link to `docs/cli/type/NN_type_name.md`)
   - Test Case Index table (TC-N prefix, minimum 4 cases)
   - Test Coverage Summary
   - Individual `### TC-N:` sections (Given/When/Then/Exit/Source)
   - Source Functions table
3. Ensure coverage of required categories (per `test_surface.rulebook.md § Spec : Minimum Coverage`):
   - Valid minimum value — ≥1 case
   - Valid maximum value — ≥1 case
   - Invalid input below range or wrong type — ≥1 case
   - Invalid input above range or unknown variant — ≥1 case
4. Add a behavioral divergence pair for types with distinct valid variants.
5. Add a row to `readme.md` Overview Table.
6. Update `../readme.md` type tier section if first type spec.

---

### Procedure :: 2. Update an Existing Type Spec

**Trigger:** A type's constraints, valid values, or error messages change.

1. Open the relevant `NN_type_name.md` file.
2. Update affected TC-N sections (Given/When/Then/Exit/Source).
3. Update the Test Coverage Summary counts.
4. Add new TC-N cases if new valid values or error paths were introduced.
5. Update Source Functions table if test implementations changed.

---

### Procedure :: 3. Remove a Type Spec

**Trigger:** A semantic type is removed from `docs/cli/type/`.

1. Delete the `NN_type_name.md` file.
2. Remove the row from `readme.md` Overview Table.
3. Check `../param/` specs that reference this type and update Source links.
