# Types and Traits (TT)

### TT1. Use generics to allow both owned and borrowed access

Parameterize over the container type
so callers can choose between `Arc`, `Box`,
or a plain reference:

```rust
pub struct DmaStreamSlice<Dma: Deref<Target = DmaStream>> {
    dma_stream: Dma,
    offset: usize,
    len: usize,
}
```

This avoids forcing an `Arc::clone`
when a `&DmaStream` reference suffices.

### TT2. Prefer enum over trait objects for closed sets

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

### TT3. Use `Box<dyn Trait>` over `Arc<dyn Trait>` when sharing is unnecessary

When a value has a single owner,
`Box<dyn Trait>` avoids reference-counting overhead
and makes ownership clearer.

### TT4. Use types to encode invariants

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

### TT5. Collapse redundant state into simpler types

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

### TT6. Eliminate redundant `Option` wrapping

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

### TT7. Traits must provide compile-time guarantees

A trait should only be implemented
when it can guarantee the invariant
for all instances.
If the property is not enforced by the type system,
a trait is the wrong tool.

### TT10. Derive standard traits proactively

Every public type should derive
all applicable standard traits:
`Debug` (always),
`Clone` (when logically copyable),
`Default` (when a natural default exists),
`PartialEq`/`Eq` (when equality is meaningful).
Missing derives force downstream code
into unnecessary workarounds.

```rust
// Good — all applicable standard traits derived
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PageFaultInfo {
    pub address: usize,
    pub flags: PageFaultFlags,
}
```

### TT11. Use `#[non_exhaustive]` on public enums and structs that may grow

Mark public enums and structs
with `#[non_exhaustive]`
when new variants or fields may be added in the future.
This reserves the right to extend the type
without breaking downstream code.

```rust
// Good — new variants can be added
// without a semver-breaking change
#[derive(Debug)]
#[non_exhaustive]
pub enum VmEvent {
    PageFault(PageFaultInfo),
    AccessFault(AccessFaultInfo),
}
```
