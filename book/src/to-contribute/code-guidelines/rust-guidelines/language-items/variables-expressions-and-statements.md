# Variables, Expressions, and Statements

### Use checked or saturating arithmetic (`checked-arithmetic`) {#checked-arithmetic}

Use checked or saturating arithmetic
for operations that could overflow.
Prefer explicit overflow handling
over silent wrapping:

```rust
// Good — overflow is handled explicitly
let total = base.checked_add(offset)
    .ok_or(Error::new(Errno::EOVERFLOW))?;

// Good — clamps instead of wrapping
let remaining = budget.saturating_sub(cost);

// Bad — may silently wrap in release builds
let total = base + offset;
```

### Prefer immutable bindings (`prefer-immutable`) {#prefer-immutable}

Declare variables with `let` rather than `let mut`
whenever possible.
Variables that never change after initialization
are far easier to reason about.
Use mutable bindings only when mutation
is genuinely required.

```rust
// Good — computed once, never modified
let offset = base_addr + page_index * PAGE_SIZE;

// Bad — mutable but never mutated after initialization
let mut offset = base_addr + page_index * PAGE_SIZE;
```

### Introduce explaining variables (`explain-variables`) {#explain-variables}

Break down complex expressions
by assigning intermediate results to well-named variables.
An explaining variable turns an opaque expression
into self-documenting code:

```rust
// Good — intent is clear
let is_page_aligned = addr % PAGE_SIZE == 0;
let is_within_range = addr < max_addr;
debug_assert!(is_page_aligned && is_within_range);

// Bad — reader must parse the whole expression
debug_assert!(addr % PAGE_SIZE == 0 && addr < max_addr);
```

See also:
_The Art of Readable Code_, Chapter 8 "Breaking Down Giant Expressions";
PR [#2083](https://github.com/asterinas/asterinas/pull/2083#discussion_r2512772091)
and [#643](https://github.com/asterinas/asterinas/pull/643#discussion_r1497243812).
