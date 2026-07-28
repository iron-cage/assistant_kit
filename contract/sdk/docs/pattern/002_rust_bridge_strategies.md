# Pattern: Rust Bridge Strategies

### Problem

`claude_runner`/`claude_runner_core` are pure Rust crates. No official Rust package exists for the Agent SDK ([S4](../behavior/004_s4_no_rust_binding.md)) — only TypeScript (npm) and Python (PyPI, ≥3.10). A Rust caller that wants SDK-mode capabilities (live mid-session control via [`Query`](../api/004_query_control_object.md), in-process custom tools, richer permission modes — see [S2](../behavior/002_s2_stream_json_control_protocol.md), [S3](../behavior/003_s3_custom_tools_in_process.md), [S6](../behavior/006_s6_permission_modes_richer_than_cli.md)) beyond what `claude_runner`'s current `--print --output-format json` single-shot mode offers must cross a language boundary. This pattern enumerates the concrete options and their tradeoffs.

### Solution

Two architecturally distinct strategies, not a spectrum — pick one:

**Strategy A — Subprocess Shim.** Ship a small Node.js (or Python) script that imports the real `@anthropic-ai/claude-agent-sdk` (or `claude-agent-sdk`) package, drives `query()` itself, and re-emits the message stream as newline-delimited JSON on its own stdout for a Rust parent process to read — essentially `claude_runner_core` gains a *second* subprocess layer (Rust → Node/Python shim → `claude` binary) instead of today's single layer (Rust → `claude` binary directly). Pros: uses the real, officially-maintained SDK — every `Options` field, every `Query` control method, every SDK bugfix/feature lands automatically on upgrade; zero protocol-reverse-engineering risk. Cons: adds a hard runtime dependency on Node.js or Python being installed wherever `claude_runner` runs (this workspace's own `assistant_kit/rulebook.md` Zero-Knowledge/Container-Only-Testing principles would need this dependency declared and containerized); doubles the process-spawn chain and its failure modes (shim crash, shim-to-Rust pipe breakage, in addition to existing `claude`-binary failure modes already catalogued in `contract/claude_code/docs/fault/`); the shim script itself becomes new, hand-written surface area needing its own tests.

**Strategy B — Native Protocol Reimplementation.** Have `claude_runner_core` speak the same `--input-format stream-json --output-format stream-json` bidirectional protocol directly ([S2](../behavior/002_s2_stream_json_control_protocol.md)) — spawn `claude` with those two flags (exactly as `claude_runner_core` already spawns it today, just with a different flag set and a kept-open stdin instead of one-shot text), and implement the control-message request/response cycle (`SDKControlRequestMessage`/`SDKControlResponseMessage`, per [`../api/005_sdk_message_stream.md`](../api/005_sdk_message_stream.md)) as ordinary Rust structs with `serde`. Pros: no new runtime dependency — the existing single-subprocess architecture (Rust → `claude` binary) is preserved exactly, just with a richer protocol on the wire; full control and understanding of every byte crossing the boundary, consistent with this crate's own contract-testing philosophy (real behavior, verified against the real binary, not a wrapped abstraction). Cons: the `stream-json` control protocol's exact message schemas are not fully published as a formal spec (this crate's [`../api/`](../api/readme.md) instances are transcribed from the *SDK's* TypeScript types, which describe the SDK's own in-memory representation, not necessarily the literal wire JSON — the mapping between the two has not been independently verified against a live capture); any wire-format change in a future Claude Code release could break this reimplementation silently until caught by contract tests, exactly the "goes RED when behavior drifts" risk this whole `contract/` pattern exists to catch early rather than avoid.

**Verification technique usable for either strategy**: point `pathToClaudeCodeExecutable` (or `spawnClaudeCodeProcess`) at a thin logging shim binary that just tees stdin/stdout to a file and execs the real `claude` binary underneath — this captures the literal argv and the literal wire JSON the real SDK sends, turning Strategy B's "not fully published" caveat above into a concretely testable, capturable artifact rather than a permanent unknown.

### Applicability

Strategy A fits if this workspace decides the value of automatic upstream-SDK-feature-parity outweighs a new language runtime dependency, or if a needed capability (a Python-only feature, an as-yet-undocumented control message) can't be reimplemented confidently in Rust. Strategy B fits this workspace's demonstrated preference (per `assistant_kit/rulebook.md`'s Zero-Knowledge Invariant, and `contract/claude_code`'s own existence) for owning and directly verifying external contracts rather than wrapping them, and avoids the new-runtime-dependency cost entirely — at the price of needing to keep the reimplementation in sync with an undocumented, binary-analysis-derived protocol, the same maintenance burden `contract/claude_code`'s `version/` collection (95 instances) already tracks for the CLI's own drift over time.

### Consequences

Whichever strategy is chosen, `assistant_kit/task/claude_runner/414_implement_sdk_protocol_run_command.md` is where that decision should actually be made and tracked — this doc instance stops at "here are the two options and their tradeoffs," deliberately not recommending one, since the choice depends on constraints (is Node.js already an acceptable dependency somewhere in this workspace? how much ongoing protocol-drift maintenance is acceptable?) this crate's documentation-only scope has no authority to resolve.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| entity | [readme.md](readme.md) | Master pattern index |
| behavior | [../behavior/001_s1_sdk_wraps_same_binary.md](../behavior/001_s1_sdk_wraps_same_binary.md) | Confirms a subprocess is spawned either way |
| behavior | [../behavior/002_s2_stream_json_control_protocol.md](../behavior/002_s2_stream_json_control_protocol.md) | The protocol Strategy B would reimplement |
| behavior | [../behavior/004_s4_no_rust_binding.md](../behavior/004_s4_no_rust_binding.md) | The constraint this pattern exists to address |
| param | [../param/012_path_to_claude_code_executable.md](../param/012_path_to_claude_code_executable.md) | Field used by the logging-shim verification technique |
| doc | `assistant_kit/task/claude_runner/414_implement_sdk_protocol_run_command.md` | Where the actual strategy decision is tracked |
