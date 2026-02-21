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
