# Idiomatics (I)

The project strongly favors standard Rust idioms:
iterators, early returns, the `?` operator,
builder patterns, and established crate conventions
over bespoke solutions.

### I1. Use the `?` operator, not chains of `.unwrap()`

The `?` operator should propagate errors idiomatically.
`.unwrap()` without justification is a code smell.

```rust
// Good
let tsc_info = cpuid.get_tsc_info()?;

// Bad
let tsc_info = cpuid.get_tsc_info().unwrap();
```

### I2. Avoid unnecessary `Box::leak` and `'static` lifetimes

`Box::leak` to obtain a `'static` reference is a last resort.
Proper lifetime management should be used,
and the `'static` bound removed
when a shorter lifetime suffices.

### I3. Prefer `impl Trait` over custom trait definitions for simple closures

Creating a custom trait
to replace what `impl Fn(...)` already expresses
adds unnecessary complexity.
Use `impl FnOnce`, `impl Fn`, etc., directly.

```rust
// Good
fn map_pages(mapper: impl FnOnce(MapInfo) -> MapProperty) { ... }

// Bad — unnecessary custom trait
trait MapOp { fn apply(&self, info: MapInfo) -> MapProperty; }
fn map_pages(mapper: impl MapOp) { ... }
```

### I4. Use `loop { break value }` over auxiliary flag variables

Auxiliary boolean or flag variables
used to signal loop exit
should be replaced with the `loop { break value; }` idiom.

```rust
// Good
let next_task = loop {
    match action {
        ReschedAction::SwitchTo(next_task) => break next_task,
        // ...
    }
};

// Bad
let mut result = None;
let mut found = false;
while !found { ... }
```

### I5. Follow established codebase conventions

When a convention already exists in the codebase, follow it.
Consistency matters more than personal preference.
Do similar things the same way throughout the codebase —
consistency reduces surprise and cognitive load
even when neither approach is objectively superior.
Do not introduce a competing convention
without compelling justification.

### I6. Prefer `Drop` for automatic cleanup

Cleanup logic that always executes
when a struct goes out of scope
should use the `Drop` trait,
eliminating the need
for explicit exit/cleanup methods.

### I7. Avoid "magical" abstractions

Closures returning closures,
custom traits that duplicate simple function signatures,
and other layered indirections
make code harder to understand for no gain.
Prefer explicit, straightforward code.

### I8. Align API naming with Rust standard library

APIs should follow the naming, phrasing,
and structural conventions
established by Rust's `std` library,
not conventions from Linux, Intel SDM, etc.

```rust
// Good — follows Rust std convention
fn wait_timeout_while(&self, ...) { ... }

// Bad — non-standard naming
fn wait_until_or_timeout(&self, ...) { ... }
```

### I9. Choose the right interior mutability primitive

Use `Cell<T>` for `Copy` types
needing single-threaded mutation through shared references.
Use `RefCell<T>` only in single-threaded contexts.
Use `Mutex`/`RwLock` for multi-threaded shared mutation.
Never use `RefCell` in a type
that might be shared across threads.

```rust
// Good — Cell for a simple Copy counter
// in a single-threaded context
struct LocalStats {
    hits: Cell<u64>,
}

// Good — Mutex for shared mutable state
// accessed from multiple threads
struct SharedStats {
    hits: Mutex<u64>,
}

// Bad — RefCell in a Sync type risks
// panics from concurrent borrow_mut() calls
struct SharedStats {
    hits: RefCell<u64>, // NOT thread-safe!
}
```

### I10. Prefer elided lifetimes; annotate only when clarifying

Do not add unnecessary lifetime parameters.
Annotate explicitly when:
(a) the return borrows from a non-obvious input,
(b) a struct holds a reference
that must outlive something specific, or
(c) the code would be confusing without it.

```rust
// Good — lifetimes elided; the signature is clear
fn name(&self) -> &str { ... }

// Good — explicit lifetime clarifies
// which input the return borrows from
fn longest<'a>(a: &'a str, b: &str) -> &'a str { ... }

// Bad — unnecessary lifetime annotation
fn name<'a>(&'a self) -> &'a str { ... }
```
