# Decision: Expose --system-prompt

**ID:** D12 · **Category:** Parameter Conventions · **Status:** ✅ Adopted

### Scope

- **Purpose**: Record why the destructive `--system-prompt` (replace) variant is exposed at all, despite stripping Claude Code's behavioral guardrails.
- **Responsibility**: Rationale for exposing the replace variant alongside the additive one, what actually survives replacement, and the documentation posture that steers users toward the safer default.
- **In Scope**: Why both flags exist; what tool definitions survive a full replace; the CLI-vs-SDK semantic split.
- **Out of Scope**: Flag reference syntax (→ [`../cli/param/`](../cli/param/readme.md)); the capability table itself (→ `command/01_run.md` Notes).

### Decision

Both `--system-prompt` (replace) and `--append-system-prompt` (extend) are exposed, even though `--system-prompt` strips Claude Code's behavioral guardrails.

### Rationale

Specialized single-purpose agents need complete control. A coding assistant locked to Rust, a JSON-only responder, a domain-specific tool — these require a clean prompt slate, not additions on top of general-purpose coding instructions. The flag is not a mistake; it's a deliberate escape hatch.

**What actually survives replacement:** tool definitions (~12,000 tokens: Bash, Read, Write, Edit, Glob, Grep, WebFetch, etc.) are injected into the assembled system prompt *before* the replacement is applied. Tools remain fully operational. What is lost is the behavioral layer — coding guidelines, git safety rules, CLAUDE.md-handling instructions, output style. Claude gets raw tool access with no behavioral scaffolding.

### Consequence

`--append-system-prompt` is documented as the default recommendation. `--system-prompt` is documented as an explicit opt-in for full-control scenarios, with the capability table in `command/01_run.md` (Notes) making the tradeoffs visible.

### Scope Boundary

This behavior applies to the **CLI** `--system-prompt` flag specifically. The Agent SDK `systemPrompt:` parameter has different semantics — tools may not be automatically preserved without using `preset: "claude_code"`. The CLI always preserves tool definitions regardless of replacement; the SDK does not make that guarantee under the same conditions.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Decision collection index |
| cli | `../cli/command/01_run.md` | Capability table (Notes) making the replace/append tradeoff visible |
