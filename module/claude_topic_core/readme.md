# claude_topic_core

Pure library for Claude Code topics: what they are called, which exist, and which one to use.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Crate manifest and dependency configuration |
| `src/` | Core library implementation |
| `tests/` | Test suite for identity, registry, enumeration, selection, pooling, and locking |
| `docs/` | Behavioral requirements: features, invariants, api |
| `verb/` | Shell scripts for each `do` protocol verb. |

## overview

A *topic* is a named, isolated conversation belonging to a base directory. This
crate owns the questions that identity and discovery raise, and runs nothing:
creating or continuing a topic means invoking Claude Code, which belongs to the
layer above. Everything here is computation over paths, one registry file, and a
process list.

It is deliberately **not** `claude_storage_core`. That crate owns session paths and
the `UUIDv5` rule this one calls into; this crate owns what a *name* means on top
of them — the two mechanisms, the registry that makes fork topics listable, and
the policy for choosing among the results.

## features

- **`( name, mode )` as the unit**: the two mechanisms can hold the same name at
  once, and are not interchangeable — the mode travels with every topic
- **Merged enumeration**: dir-mode directories scanned off disk and fork-mode
  names read from the registry, in one sorted list
- **Addressable-subset filter**: `enumerate_live` keeps only topics that hold a
  conversation, which is also what keeps fan-out out of `-`-prefixed scratch
  directories
- **Idle-first selection**: a seeded draw that prefers topics with no turn in
  flight, because handing a prompt to a busy topic is the problem topics exist to
  avoid
- **Idempotent pool naming**: "make sure N exist", not "add N more", with gaps
  filled before the range extends
- **Advisory per-topic locking**: compare-and-delete reclaim of a dead owner's
  lock, honest about the window it does not close

## usage

```toml
[dependencies]
claude_topic_core = { workspace = true }
```

```rust,no_run
use claude_topic_core::{ enumerate_live, select, Pick, TopicMode };
use std::path::Path;

let base = Path::new( "/home/me/project" );

for topic in enumerate_live( base )
{
  println!( "{} ({}) {} sessions", topic.name, topic.mode, topic.sessions );
}

// Hand a prompt to one of them — idle-first, reproducible under a fixed seed.
let topics = enumerate_live( base );
if let Some( chosen ) = select( &topics, Pick::Idle, 7 )
{
  // The mode has to travel with the name: `--topic NAME --topic-mode MODE`.
  assert!( matches!( chosen.topic.mode, TopicMode::Fork | TopicMode::Dir ) );
  println!( "delegating to {} ({})", chosen.topic.name, chosen.topic.mode );
}
```

## two things that look like authorities and are not

**The registry.** Fork topics are named by `UUIDv5( canonical base, name )`, which
is one-way, so a side-channel index exists to remember the names. Entries outlive
the sessions they name, and a name containing a newline is never recorded at all.
The session file is the authority; `enumerate` consults both.

**The `-` prefix.** `topic_name_of` accepts any `-`-prefixed directory name, and
this workspace marks generated and ignored directories the same way. `-daemon/`
and `./-0007_verb_test.log`'s siblings are indistinguishable from dir-mode topics
by name alone. `enumerate_live` separates them by looking for sessions instead —
a strong heuristic, not a guarantee.
