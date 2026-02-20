# Rust Guidelines

## API Documentation Guidelines

API documentation describes the meanings and usage of APIs,
and will be rendered into web pages by rustdoc.

It is necessary to add documentation to all public APIs,
including crates, modules, structs, traits, functions, macros, and more.
The use of the `#[warn(missing_docs)]` lint enforces this rule.

Asterinas adheres to the API style guidelines of the Rust community.
The recommended API documentation style are specified by two official resources:
1. The rustdoc book: [How to write documentation](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html);
2. The Rust RFC book: [API Documentation Conventions](https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.html#appendix-a-full-conventions-text).

## Lint Guidelines

Lints help us improve the code quality and find more bugs.
When suppressing lints, the suppression should affect as little scope as possible,
to make readers aware of the exact places where the lint is generated,
and to make it easier for subsequent committers to maintain such lint.

For example, if some methods in a trait are dead code,
marking the entire trait as dead code is unnecessary and
can easily be misinterpreted as the trait itself being dead code.
Instead, the following pattern is preferred:
```rust
trait SomeTrait {
    #[expect(dead_code)]
    fn foo();

    #[expect(dead_code)]
    fn bar();

    fn baz();
}
```

There is one exception:
If it is clear enough that every member will trigger the lint,
it is reasonable to expect the lint at the type level.
For example, in the following code,
we add `#[expect(non_camel_case_types)]` for the type `SomeEnum`,
instead of for each variant of the type:
```rust
#[expect(non_camel_case_types)]
enum SomeEnum {
    FOO_ABC,
    BAR_DEF,
}
```

### When to `#[expect(dead_code)]`

In general, dead code should be avoided because
_(i)_ it introduces unnecessary maintenance overhead, and
_(ii)_ its correctness can only be guaranteed by
manual and error-pruned review of the code.

In the case where expecting dead code is necessary,
it should fulfill the following requirements:
 1. We have a _concrete case_ that will be implemented in the future and
    will turn the dead code into used code.
 2. The semantics of the dead code are _clear_ enough
    (perhaps with the help of some comments),
    _even if the use case has not been added_.
 3. The dead code is _simple_ enough that
    both the committer and the reviewer can be confident that
    the code must be correct _without even testing it_.
 4. It serves as a counterpart to existing non-dead code.

For example, it is fine to add ABI constants that are unused because
the corresponding feature (_e.g.,_ a system call) is partially implemented.
This is a case where all of the above requirements are met,
so adding them as dead code is perfectly acceptable.

## Formatting

Asterinas uses [rustfmt](https://rust-lang.github.io/rustfmt/)
with a project-wide `rustfmt.toml`:

```toml
imports_granularity="Crate"
group_imports="StdExternalCrate"
reorder_imports=true
skip_macro_invocations = ["chmod", "mkmod", "ioc"]
```

- **`imports_granularity = "Crate"`** —
  Merge imports from the same crate into a single `use` statement.
- **`group_imports = "StdExternalCrate"`** —
  Separate imports into three groups
  (standard library, external crates, crate-local),
  divided by blank lines.
- **`reorder_imports = true`** —
  Sort imports alphabetically within each group.
- **`skip_macro_invocations`** —
  Skip formatting inside `chmod`, `mkmod`, and `ioc` macro calls.

Run `cargo fmt` before submitting a pull request.

## Import Conventions

Imports follow a three-group pattern
enforced by the `rustfmt.toml` settings above:

1. Standard library (`core`, `alloc`, `std`)
2. External crates
3. Crate-local (`super::`, `crate::`)

Each group is separated by a blank line.
Items from the same crate are merged
into a single `use` statement with nested braces.

Example from `kernel/src/fs/path/dentry.rs`:

```rust
use core::{
    ops::Deref,
    sync::atomic::{AtomicU32, Ordering},
};

use hashbrown::HashMap;
use ostd::sync::RwMutexWriteGuard;

use super::{is_dot, is_dot_or_dotdot, is_dotdot};
use crate::{
    fs::{
        self,
        utils::{Inode, InodeExt, InodeMode, InodeType, MknodType},
    },
    prelude::*,
};
```

## Unsafe Code

Asterinas kernel crates deny unsafe code by default:

```rust
#![deny(unsafe_code)]
```

Only OSTD (`ostd/`) crates may contain `unsafe` code.
If a kernel crate requires an unsafe operation,
the functionality should be provided as a safe API in OSTD.

### SAFETY Comments

Every `unsafe` block must have a preceding `// SAFETY:` comment
that justifies why the operation is sound.
For multi-condition invariants,
use a numbered list:

```rust
// SAFETY:
// 1. We have exclusive access to both the current context
//    and the next context (see above).
// 2. The next context is valid (because it is either
//    correctly initialized or written by a previous
//    `context_switch`).
unsafe {
    context_switch(next_task_ctx_ptr, current_task_ctx_ptr);
}
```

### Safety Documentation

Public `unsafe` functions and traits
must include a `# Safety` section in their doc comments
describing the invariants that callers must uphold.

```rust
/// Reads a value from the given physical address.
///
/// # Safety
///
/// The caller must ensure that `addr` points to
/// a valid, mapped physical memory region of at least
/// `size_of::<T>()` bytes.
pub unsafe fn read_phys<T>(addr: usize) -> T { ... }
```

For more on writing sound unsafe code,
see [The Rustonomicon](https://doc.rust-lang.org/nomicon/).

## Error Handling

Use the `return_errno_with_message!` macro
for returning errors with descriptive messages:

```rust
return_errno_with_message!(
    Errno::ENOTDIR,
    "the dentry is not related to a directory inode"
);
```

Prefer early returns to reduce nesting:

```rust
pub(super) fn unlink(&self, name: &str) -> Result<()> {
    if is_dot_or_dotdot(name) {
        return_errno_with_message!(Errno::EINVAL, "unlink on . or ..");
    }
    // ... main logic at the top level
}
```

See [General Guidelines — Error Messages](general-guidelines.md#error-messages)
for message formatting rules.

## Concurrency

### Lock Ordering

When acquiring multiple locks,
always acquire them in a consistent order
to prevent deadlocks.
Document the required ordering
in comments near the lock declarations.

### Race Condition Awareness

Consider whether operations on shared state are atomic.
Use appropriate synchronization primitives
and reason explicitly about concurrent access.

### Checked Arithmetic

Use checked or saturating arithmetic in debug builds
for operations that could overflow.
The `debug_assert!` macro is useful
for catching overflow conditions during development:

```rust
debug_assert!(self.align.is_multiple_of(PAGE_SIZE));
debug_assert!(self.align.is_power_of_two());
```

### Drop Ordering

Be mindful of the order in which values are dropped.
When a value holds a lock guard or other RAII resource,
ensure that the drop order
does not create a use-after-free or deadlock.
Consider explicit `drop()` calls
when the default drop order is incorrect.
