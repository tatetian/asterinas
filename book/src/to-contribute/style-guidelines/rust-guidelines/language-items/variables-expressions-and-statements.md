# Variables, Expressions, and Statements (VE)

### VE1. Use checked or saturating arithmetic

Use checked or saturating arithmetic
for operations that could overflow.
The `debug_assert!` macro is useful
for catching overflow conditions during development:

```rust
debug_assert!(self.align.is_multiple_of(PAGE_SIZE));
debug_assert!(self.align.is_power_of_two());
```

### VE2. Prefer immutable bindings

Declare variables with `let` rather than `let mut`
whenever possible.
Variables that never change after initialization
are far easier to reason about.
Use mutable bindings only when mutation
is genuinely required.

### VE3. Initialize close to first use

Declare and initialize each variable
as close as possible to where it is first used.
A large gap between declaration and usage
widens the window for errors
and forces readers to hold more context in memory.

### VE4. Use each variable for exactly one purpose

Do not reuse a variable for a different purpose
later in the same scope.
If you need a new value, create a new binding.
Reusing names for unrelated data
is a common source of subtle bugs.

### VE5. Introduce explaining variables

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
