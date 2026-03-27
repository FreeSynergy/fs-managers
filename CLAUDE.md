# CLAUDE.md – fs-managers

## What is this?

FreeSynergy Managers — CLI tools for managing FreeSynergy components:
language packs, bots, containers, and other runtime resources.

## Rules

- Language in files: **English** (comments, code, variable names)
- Language in chat: **German**
- OOP everywhere: traits over match blocks, types carry their own behavior
- No CHANGELOG.md
- After every feature: commit directly

## Quality Gates (before every commit)

```
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo test
```

Every lib.rs / main.rs must have:
```rust
#![deny(clippy::all, clippy::pedantic, warnings)]
```

## Workspace

| Crate | Binary | Description |
|---|---|---|
| `bots` | `fs-bot` | Bot instance manager CLI |
| `language` | `fs-lang` | Language pack manager CLI |

## Known Issues

- `language`: uses gix API (pre-0.65). `prepare_push` and `SignatureRef` need
  migration to the new gix API. Tracked as a separate fix.
