# Feature Documentation Operations

- **Actor:** Developer
- **Trigger:** A new feature is added or an existing one is significantly changed.
- **Emits:** —

## Rule

`assistant_kit` is a pure facade, so nearly every feature change here is a change to the
feature graph in `Cargo.toml` — a new domain module, a renamed dependency activation feature,
or a change to what `full`/`enabled` bundle. Record the new graph in
[001_aggregation.md](001_aggregation.md) rather than creating a new instance: the feature
graph is one requirement set, not one per domain. Create a new instance only for behavior
that is not expressible as a feature-to-module mapping.

## Add Feature Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Overview Table: add row with ID, Name, Purpose, Status
4. Add the node and its edges to `../doc_graph.yml`, updating `node_count` and `edge_count`
5. Increment the `feature/` instance count in `../entity.md` and add a Master Doc Instances row

## Update Feature Documentation

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update `readme.md` Overview Table row and `../entity.md`
3. If cross-references changed: update `../doc_graph.yml` edges

## Example

Adding feature document `002_msrv_policy`:

1. Check `readme.md` Overview Table — current highest ID is `001`
2. Create `002_msrv_policy.md` in this directory
3. Add row: `| 002 | [MSRV Policy](002_msrv_policy.md) | Minimum supported Rust version guarantees | ✅ |`
4. Add a `feature/002` node to `../doc_graph.yml` and bump `node_count` to 3
5. Bump `feature/` instances to 2 in `../entity.md` and add the Master Doc Instances row
