# Naming

Asterinas enforces strict, Rust-idiomatic naming
across the entire codebase.
Names must be accurate, unabbreviated,
and follow
[Rust API Guidelines on naming](https://rust-lang.github.io/api-guidelines/naming.html).

### Follow Rust CamelCase and acronym capitalization (`camel-case-acronyms`) {#camel-case-acronyms}

Type names follow Rust's CamelCase convention.
Acronyms are title-cased per the Rust API Guidelines:

```rust
// Good
IoMemoryArea
PciDeviceLocation
Nvme
Tcp

// Bad
IOMemoryArea
PCIDeviceLocation
NVMe
TCP
```

### End closure variables with `_fn` (`closure-fn-suffix`) {#closure-fn-suffix}

Variables holding closures or function pointers
must signal they are callable by ending with `_fn`.
Treating a closure variable
as if it were a data object misleads readers.

```rust
// Good — clearly a callable
let task_fn = self.func.take().unwrap();
let thread_fn = move || {
    let _ = oops::catch_panics_as_oops(task_fn);
    current_thread!().exit();
};

let expired_fn = move |_guard: TimerGuard| {
    ticks.fetch_add(1, Ordering::Relaxed);
    pollee.notify(IoEvents::IN);
};
```

See also:
PR [#395](https://github.com/asterinas/asterinas/pull/395#discussion_r1402964415)
and [#783](https://github.com/asterinas/asterinas/pull/783#discussion_r1593335375).

### Use assertion-style boolean names (`rust-bool-names`) {#rust-bool-names}

Boolean variables, fields, and functions
should read as assertions of fact.
Use `is_`, `has_`, `can_`, `should_`,
`was_`, or `needs_` prefixes.
Never use negated names.

```rust
// Good — reads as an assertion
fn is_page_aligned(&self) -> bool { ... }
fn has_permission(&self, perm: Permission) -> bool { ... }
let can_read = mode.is_readable();

// Bad — verb suggests an action, not a query
fn check_permission(&self, perm: Permission) -> bool { ... }
// Bad — double negative
let is_not_empty = !buf.is_empty();
```

See also:
[General Guidelines](../general-guidelines/README.md#bool-names);
PR [#1488](https://github.com/asterinas/asterinas/pull/1488#discussion_r1841827039).
