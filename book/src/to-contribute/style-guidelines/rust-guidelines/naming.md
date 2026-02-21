# Naming (RN)

Asterinas enforces strict, Rust-idiomatic naming
across the entire codebase.
Names must be accurate, unabbreviated,
and follow
[Rust API Guidelines on naming](https://rust-lang.github.io/api-guidelines/naming.html).

### RN1. Omit `get_` prefix on simple getters

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

### RN2. Use full English words, not abbreviations

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

### RN3. Names must accurately reflect behavior

If a name can be misread
to imply the wrong behavior, side effect, or ownership,
it must be corrected immediately.

```rust
// Good — clearly a count
removed_nr_subscribers

// Bad — looks like a collection of watches
deleted_watches
```

### RN4. Follow Rust CamelCase and acronym capitalization

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

### RN5. End closure variables with `_fn`

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

### RN6. Avoid plural type names

Types representing a single conceptual unit
should not be named in the plural form,
as it creates awkward names
when creating collections of the type.

```rust
// Good
LocalWorkerPool

// Bad — "a Vec<LocalWorkers>" is confusing
LocalWorkers
```

### RN7. Prefer domain-accurate terminology

Choose accurate English or Rust-ecosystem terminology
rather than inheriting poorly-chosen Linux names.
Think about what is best for Asterinas,
not what Linux happened to call it.

```rust
// Good — descriptive and accurate
pub struct SoftIrqLine { ... }

// Bad — inherited jargon with no clear meaning
pub struct Tasklet { ... }
```

### RN8. Use assertion-style boolean names

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

See also
[General Guidelines — N7](../general-guidelines/themes.md#n7-use-assertion-style-boolean-names).
