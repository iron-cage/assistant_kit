# Invariant Documentation Operations

- **Actor:** Developer
- **Trigger:** A new invariant is identified or an existing constraint changes.
- **Emits:** —

## Rule

Every invariant in this directory must ship with a mechanically runnable check — a `grep`, a
`cargo tree`, or a build command whose expected output is stated. An invariant that can only
be verified by reading the code is a convention, not an invariant, and belongs in `feature/`
or the crate `readme.md` instead. When adding a dependency-set restriction, state explicitly
how it differs from `dream`'s corresponding rule: the two facades constrain opposite layers
and their `Cargo.toml` rules are not interchangeable.

## Add Invariant Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Write the Enforcement Mechanism section with a runnable command and its expected output
4. Register in `readme.md` Overview Table: add row with ID, Name, Purpose, Status
5. Add the node and its edges to `../doc_graph.yml`, updating `node_count` and `edge_count`
6. Increment the `invariant/` instance count in `../entity.md` and add a Master Doc Instances row

## Update Invariant Documentation

1. Edit the target `NNN_*.md` file
2. Re-run the Enforcement Mechanism command and confirm it still produces the documented output
3. If name or purpose changed: update `readme.md` Overview Table row and `../entity.md`

## Example

Adding invariant document `002_no_transitive_binary`:

1. Check `readme.md` Overview Table — current highest ID is `001`
2. Create `002_no_transitive_binary.md` in this directory
3. Enforcement: `cargo tree -p assistant_kit --features full --edges normal | grep -c ' assistant '` → expected `0`
4. Add row: `| 002 | [No Transitive Binary](002_no_transitive_binary.md) | Layer 3 binary never enters the dep tree | ✅ |`
5. Add an `invariant/002` node to `../doc_graph.yml` and bump `node_count` to 3
6. Bump `invariant/` instances to 2 in `../entity.md` and add the Master Doc Instances row
