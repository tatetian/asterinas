# Macros and Attributes (MA)

### MA1. Suppress lints at the narrowest scope

When suppressing lints,
the suppression should affect as little scope as possible.
This makes readers aware
of the exact places where the lint is generated
and makes it easier for subsequent committers
to maintain the suppression.

```rust
// Good — each method is individually marked
trait SomeTrait {
    #[expect(dead_code)]
    fn foo();

    #[expect(dead_code)]
    fn bar();

    fn baz();
}

// Bad — the entire trait is suppressed
#[expect(dead_code)]
trait SomeTrait { ... }
```

There is one exception:
if it is clear enough
that every member will trigger the lint,
it is reasonable to expect the lint at the type level.

```rust
#[expect(non_camel_case_types)]
enum SomeEnum {
    FOO_ABC,
    BAR_DEF,
}
```

### MA2. When to `#[expect(dead_code)]`

In general, dead code should be avoided because
_(i)_ it introduces unnecessary maintenance overhead, and
_(ii)_ its correctness can only be guaranteed
by manual and error-prone review.

Dead code is acceptable only when all of these hold:

1. A _concrete case_ will be implemented in the future
   that turns the dead code into used code.
2. The semantics are _clear_ enough,
   even without the use case.
3. The dead code is _simple_ enough
   that both the committer and the reviewer
   can be confident it is correct without testing.
4. It serves as a counterpart to existing non-dead code.

For example, it is fine to add ABI constants
that are unused because the corresponding feature
is partially implemented.

### MA3. Format with `rustfmt`

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

### MA4. Prefer functions over macros when possible

Use macros only for compile-time code generation,
variadic arguments, conditional compilation,
or syntax that must be expanded before type checking.
Functions are easier to understand, debug, and test.

```rust
// Bad — a macro used where a function suffices
macro_rules! add_one {
    ($x:expr) => { $x + 1 };
}

// Good — a plain function
fn add_one(x: usize) -> usize {
    x + 1
}
```

### MA5. Use `$crate::` paths in `macro_rules!` for hygiene

Always reference items from the defining crate
using `$crate::path::to::item`.
This ensures macros work correctly
when invoked from other crates.

```rust
// Good — $crate:: ensures correct resolution
macro_rules! return_errno {
    ($errno:expr) => {
        return Err($crate::error::Error::new($errno))
    };
}

// Bad — unqualified path breaks
// when macro is used from another crate
macro_rules! return_errno {
    ($errno:expr) => {
        return Err(error::Error::new($errno))
    };
}
```
