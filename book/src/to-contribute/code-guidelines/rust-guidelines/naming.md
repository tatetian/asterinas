# Naming

Asterinas enforces strict, Rust-idiomatic naming
across the entire codebase.
Names must be accurate, unabbreviated,
and follow
[Rust API Guidelines on naming](https://rust-lang.github.io/api-guidelines/naming.html).

### Omit `get_` prefix on simple getters (`no-get-prefix`) {#no-get-prefix}

Simple accessor methods use the bare noun,
not a `get_` prefix.
Only methods that perform computation
or have side effects use a verb prefix.

```rust
// Good — bare noun for simple accessors
pub fn map_to_addr(&self) -> Vaddr {
    self.map_to_addr
}

pub fn map_size(&self) -> usize {
    self.map_size.get()
}

// Bad
pub fn get_map_to_addr(&self) -> Vaddr { ... }
```

See also:
PR [#170](https://github.com/asterinas/asterinas/pull/170#discussion_r1154020166)
and [#424](https://github.com/asterinas/asterinas/pull/424#discussion_r1387565451).

### Use full English words, not abbreviations (`no-abbreviations`) {#no-abbreviations}

Abbreviated or jargon-derived names
must be replaced with accurate, full English words.
Linux-inherited shorthand is not acceptable.

```rust
// Good
superblock_options
descendant

// Bad
sb_options
subdir
```

See also:
PR [#170](https://github.com/asterinas/asterinas/pull/170#discussion_r1154551016).

### Names must accurately reflect behavior (`accurate-names`) {#accurate-names}

If a name can be misread
to imply the wrong behavior, side effect, or ownership,
it must be corrected immediately.

```rust
// Good — clearly a count
removed_nr_subscribers

// Bad — looks like a collection of watches
deleted_watches
```

See also:
PR [#1488](https://github.com/asterinas/asterinas/pull/1488#discussion_r1825441287)
and [#2964](https://github.com/asterinas/asterinas/pull/2964#discussion_r2789739882).

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
or `was_` prefixes.
Never use double negatives.

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
