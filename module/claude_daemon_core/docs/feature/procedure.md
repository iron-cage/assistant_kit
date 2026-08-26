# Feature Documentation Operations

- **Actor:** Developer
- **Trigger:** A new feature is added or an existing one is significantly changed.
- **Emits:** —

## Add Feature Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Overview Table: add row with ID, Name, Purpose, Status
4. Register in `../entity.md` Master Doc Instances Table and increment the `feature/` row's `Instances` count

## Update Feature Documentation

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update `readme.md` Overview Table row and the `../entity.md` instance row

## Rule

A change to `Request` or `Response` is a wire-format change, not an implementation detail. Record the new shape in [002_wire_protocol.md](002_wire_protocol.md) in the same change that makes it — a client on the other side of a socket cannot be recompiled along with the daemon.

## Example

Adding feature document `004_client_reconnect`:

1. Check `readme.md` Overview Table — current highest ID is `003`
2. Create `004_client_reconnect.md` in this directory
3. Add row: `| 004 | Client Reconnect | Reattach to a hosted session after a client restart | ✅ |`
4. Add the matching `../entity.md` instance row and bump `feature/` Instances from 3 to 4
