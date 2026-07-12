# Behavior S7: `CLAUDE_CODE_ENTRYPOINT=sdk-cli` Does Not Prove Real SDK Usage

### Scope

- **Purpose**: Document that a `claude` process reporting `CLAUDE_CODE_ENTRYPOINT=sdk-cli` is not reliable proof that the real `@anthropic-ai/claude-agent-sdk`/`claude-agent-sdk` library launched it — a hand-rolled subprocess wrapper shaped like `--print --output-format json -c` produces the identical marker.
- **Responsibility**: Authoritative instance for behavior S7 — the specific finding from this workspace's own session ancestry investigation, elevated to a documented contract behavior since it directly bears on how "SDK-mode" should be defined for this crate's own purpose.
- **In Scope**: `CLAUDE_CODE_ENTRYPOINT`'s `sdk-cli`/`sdk-ts` values; the false-positive risk of using this env var alone to detect genuine SDK usage.
- **Out of Scope**: The full `CLAUDE_CODE_ENTRYPOINT` value catalog and its other non-SDK values (`claude-vscode`, `remote*`, `claude-in-teams`) — already documented (→ [`../../../claude_code/docs/param/134_entrypoint.md`](../../../claude_code/docs/param/134_entrypoint.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 90% | **Since**: ≤v2.1.197 (env var), current session (the false-positive finding) | **Evidence**: E3, E5

`contract/claude_code/docs/param/134_entrypoint.md` already documents `CLAUDE_CODE_ENTRYPOINT` as an undocumented-but-binary-confirmed enum including `"sdk-cli"` and `"sdk-ts"` among its values, "set by the launching wrapper... not meant to be hand-set." This session independently confirmed, via direct `ps` ancestry inspection of its own process tree, that its actual parent process is `clr` (this workspace's own Rust CLI) invoking `claude --dangerously-skip-permissions --effort max --model claude-sonnet-5 --print --output-format json -c "<message>"` — a plain subprocess spawn with no `@anthropic-ai/claude-agent-sdk` or `claude-agent-sdk` package anywhere in `clr`'s dependency tree (its source was not found under `~/pro/lib/`, and its invocation is a manual spawn-and-parse-JSON, not a `query()` call).

Despite this, the resulting `claude` process is functionally indistinguishable — at least at the `CLAUDE_CODE_ENTRYPOINT` level — from one launched by the real SDK's `sdk-cli` code path, because the *shape* of the invocation (`--print --output-format json`, no interactive TTY) is exactly what the binary's own entrypoint-classification logic keys on, independent of what actually spawned it. **Practical consequence for this crate and the linked task**: `CLAUDE_CODE_ENTRYPOINT` cannot be used as a reliable signal to detect "is this session using the real SDK" — any invocation matching the same shape self-reports identically, whether or not the calling code ever imported the SDK package. A genuine SDK-mode detection would need to check for the SDK's actual bidirectional `stream-json` control protocol (S2) or its distinctive control-message types, not this single env var.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E3 | S7 | Doc | [`../../../claude_code/docs/param/134_entrypoint.md`](../../../claude_code/docs/param/134_entrypoint.md) | Description | `sdk-cli`/`sdk-ts` values confirmed via binary string/reference inspection; "set by the launching wrapper... not meant to be hand-set" |
| E5 | S7 | Observation | This session, `ps` ancestry walk (see workspace memory `reference_clr_default_invocation_pattern`) | `clr "<msg>"` → `claude --print --output-format json -c "<msg>"` | `clr` manually spawns-and-parses-JSON per turn; no SDK package in its dependency tree; yet produces a process shape matching `sdk-cli`'s classification criteria |

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| entity | [readme.md](readme.md) | Master index |
| behavior | [002_s2_stream_json_control_protocol.md](002_s2_stream_json_control_protocol.md) | The protocol-level signal that would actually distinguish real SDK usage |
| doc | [`../../../claude_code/docs/param/134_entrypoint.md`](../../../claude_code/docs/param/134_entrypoint.md) | Full `CLAUDE_CODE_ENTRYPOINT` value catalog |
| doc | `assistant_kit/task/claude_runner/414_implement_sdk_protocol_run_command.md` | Task this finding directly informs (don't gate "SDK mode" detection on this env var alone) |
