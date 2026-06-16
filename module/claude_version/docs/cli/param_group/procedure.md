# param_group/ — Procedure

### Scope

- **Purpose**: Operational steps for managing parameter group instance files in this directory.
- **Responsibility**: Procedures for adding, updating, or removing parameter group documentation.

---

### Procedure :: 1. Add a New Parameter Group

**Trigger:** A new logical grouping of clv parameters is identified.

1. Assign the next sequential number (NN) to the new group.
2. Create `NN_group_name.md` with:
   - Summary Block (`-- **Label:** value` lines)
   - Coherence test statement
   - Parameters table (link each param to `../param/NN_name.md`)
   - Referenced Commands table (all commands that use any group member)
   - Referenced User Stories table
3. Update the `### All Groups` index table in `readme.md`.
4. Add `### Referenced Parameter Groups` links from each member param file.
5. Update `../readme.md` Completion Matrix if level changes.

---

### Procedure :: 2. Update Group Membership

**Trigger:** A parameter is added to or removed from a group.

1. Open the relevant `NN_group_name.md` file.
2. Update the Parameters table.
3. Update the Referenced Commands table if command applicability changes.
4. Update bidirectional `### Referenced Parameter Groups` links in param files.

---

### Procedure :: 3. Remove a Group

**Trigger:** A parameter group is dissolved.

1. Delete the `NN_group_name.md` file.
2. Remove the group row from the `### All Groups` table in `readme.md`.
3. Remove `### Referenced Parameter Groups` links from all member param files.
