# Types and Traits

### Prefer enum over trait objects for closed sets (`enum-over-dyn`) {#enum-over-dyn}

When the set of variants is known and closed,
an enum is preferable to `Box<dyn Trait>`
for both performance and pattern-matching expressiveness.

```rust
// Good — closed set modeled as an enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermStatus {
    Exited(u8),
    Killed(SigNum),
}
```

### Use types to encode invariants (`rust-type-invariants`) {#rust-type-invariants}

Leverage the type system
to make illegal states unrepresentable.
Prefer newtypes and enums
over bare integers and boolean flags.

```rust
// Good — access mode is enforced by the type system
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessMode {
    O_RDONLY = 0,
    O_WRONLY = 1,
    O_RDWR = 2,
}

impl AccessMode {
    pub fn is_readable(&self) -> bool {
        matches!(*self, AccessMode::O_RDONLY | AccessMode::O_RDWR)
    }
}
```

See also:
[General Guidelines](../../general-guidelines/README.md#type-invariants)
for the language-agnostic formulation;
PR [#2265](https://github.com/asterinas/asterinas/pull/2265#discussion_r2266214191)
and [#2514](https://github.com/asterinas/asterinas/pull/2514).

### Collapse redundant state into simpler types (`simplify-state`) {#simplify-state}

Redundant wrapper types,
unnecessary `Inner` structs,
or intermediate state holders
that track what could be modeled
with existing types should be removed.

```rust
// Bad — unnecessary Inner wrapper
pub struct Socket {
    inner: SpinLock<SocketInner>,
}
struct SocketInner {
    state: SocketState,
}

// Good — simplified when Inner adds no value
pub struct Socket {
    state: SpinLock<SocketState>,
}
```

### Eliminate redundant `Option` wrapping (`no-redundant-option`) {#no-redundant-option}

When a type is always present,
wrapping it in `Option`
adds unnecessary complexity at every usage site.

```rust
// Bad — name is always set, but every use site
// must handle None
pub struct Thread {
    name: Option<ThreadName>,
}

// Good — name is always present
pub struct Thread {
    name: ThreadName,
}
```

See also:
PR [#2887](https://github.com/asterinas/asterinas/pull/2887#discussion_r2692231741)
and [#2151](https://github.com/asterinas/asterinas/pull/2151).
