# TSK-063: Update spec.md — FR-16 `.account.status name::` extension

## Goal

Update `spec.md` in `claude_profile` to add FR-16 (account status by name), the
`OAuthAccount Profile` vocabulary entry, `.claude.json` in the File Topology table,
`name::` in the CLI command table for `.account.status`, and an FR-16 row in the
Conformance Matrix.

## Motivation

The spec must precede implementation. FR-16 defines the contract — optional `name::`,
backward-compatible no-name path, N/A email/org for non-active accounts, NotFound error
for unknown names — that Phase 3 (TSK-065) will implement.

## In Scope

- `module/claude_profile/spec.md`:
  - `## Vocabulary` — add `OAuthAccount Profile` entry
  - `### File Topology` table — add `.claude.json` row
  - CLI command table — add `name::` to `.account.status` row
  - After FR-15 — add `FR-16: Account Status by Name` section with acceptance criteria
  - Conformance Matrix — add FR-16 row (❌ pending implementation)

## Out of Scope

- CLI documentation files (covered in TSK-066)
- Implementation changes (covered in TSK-065)
- `src/persist.rs` Known Pitfalls (covered in TSK-064)

## Work Procedure

1. Open `spec.md`. Navigate to `## Vocabulary` — append entry:
   `- **OAuthAccount Profile**: Per-session profile metadata stored in \`~/.claude/.claude.json\`
   under the \`oauthAccount\` key. Contains \`emailAddress\`, \`organizationName\`,
   \`displayName\`, \`organizationRole\`. Populated by Claude Code on authentication.
   Available ONLY for the currently active session — not in per-account credential snapshots.`

2. Navigate to `### File Topology` table — add row:
   `| \`~/.claude/.claude.json\` | Claude Code session config: account profile (email, org), user preferences |`

3. Navigate to CLI command table — update `.account.status` row params:
   change `\`v::\`, \`format::\`` → `\`name::\`, \`v::\`, \`format::\``

4. After FR-15 section — insert FR-16 section:
   ```
   #### FR-16: Account Status by Name

   `.account.status` must accept an optional `name::` parameter. When provided:
   - Show status of the named account's credential snapshot instead of the active account
   - Token state computed from the named account's own `expiresAt` field
   - Email and organization shown as `N/A` if the named account is not the active one
   - Error with `NotFound` if the named account does not exist in the account store

   When `name::` is omitted: behavior is identical to the current implementation (backward compatible).
   At `v::1` and above, show `subscriptionType`, `rateLimitTier`, and (for the active account only)
   email and organization from `~/.claude/.claude.json`.
   ```

5. Navigate to Conformance Matrix — add row:
   `| ❌ | **FR-16:** Account status by name | \`name::\` param selects account, backward-compat no-name path | \`account_status_name_test.rs\` (to create) |`

## Validation List

Desired answer for every question is YES.

- [ ] Does `spec.md` contain a `FR-16` section after FR-15?
- [ ] Does FR-16 state the backward-compatibility requirement explicitly?
- [ ] Does FR-16 state N/A behavior for email/org on non-active accounts?
- [ ] Does FR-16 state NotFound error for non-existent account name?
- [ ] Does the File Topology table have a `.claude.json` row?
- [ ] Does the CLI command table show `name::` for `.account.status`?
- [ ] Does `## Vocabulary` have the `OAuthAccount Profile` entry?
- [ ] Does the Conformance Matrix have an FR-16 row (❌)?

## Validation Procedure

### Measurements

**M1 — FR-16 added**
```bash
grep -c "FR-16" module/claude_profile/spec.md
```
Before: 0. Expected: ≥2 (definition + conformance table).

**M2 — .claude.json in topology**
```bash
grep -c "claude\.json" module/claude_profile/spec.md
```
Before: 0. Expected: ≥1.

**M3 — CLI table has name:: for account.status**
```bash
grep "account.status" module/claude_profile/spec.md
```
Expected: line contains `name::` alongside `v::` and `format::`.

### Anti-faking checks

**AF1 — FR-16 has testable AC**
```bash
grep -A 15 "FR-16" module/claude_profile/spec.md | grep -c "Error\|NotFound\|N/A\|backward"
```
Expected: ≥2 (NotFound + N/A + backward-compat all mentioned).

## Outcomes

**Completed:** 2026-03-31
**Result:** Done — added FR-16 section, OAuthAccount Profile vocabulary, `.claude.json` in File Topology, `name::` in CLI command table, and FR-16 conformance row to `spec.md`.
**Files changed:** `module/claude_profile/spec.md`
