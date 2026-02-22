# Idiomatics

The project strongly favors standard Rust idioms:
iterators, early returns, the `?` operator,
builder patterns, and established crate conventions
over bespoke solutions.

### Use the `?` operator, not chains of `.unwrap()`

The `?` operator should propagate errors idiomatically.
`.unwrap()` without justification is a code smell.

```rust
// Good
let tsc_info = cpuid.get_tsc_info()?;

// Bad
let tsc_info = cpuid.get_tsc_info().unwrap();
```

See also:
PR [#2784](https://github.com/asterinas/asterinas/pull/2784).

### Choose the right interior mutability primitive

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
