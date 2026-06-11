# Test: noun:: contracts

NC-N noun contract test specs for clp CLI domain nouns. Each spec covers lifecycle
correctness, output schema fidelity, and error code contract as defined in
`docs/cli/command_noun/`.

**NC- extension note:** NC- (Noun Contract) is a project-local element type extension not
registered in `test_surface.rulebook.md` (that file is outside the `module/claude_profile/`
package scope). This directory is the authorizing source for the NC- prefix.

### Responsibility Table

| File | Noun | Commands Covered | NC-N Cases |
|------|------|-----------------|-----------|
| `01_account.md` | account | `.accounts`, `.account.{save,use,delete,limits,relogin,rotate,renewal,inspect,assign}` | NC-1/2/3 |
| `02_token.md` | token | `.token.status` | NC-1/2/3 |
| `03_credentials.md` | credentials | `.credentials.status` | NC-1/2/3 |

### Coverage Summary

| Noun Files | Min Cases |
|------------|-----------|
| 3 | ≥ 3 each (9 total) |

### See Also

- [docs/cli/command_noun/](../../../../docs/cli/command_noun/readme.md) — noun source docs
