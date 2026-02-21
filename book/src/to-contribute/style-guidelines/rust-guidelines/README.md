# Rust Guidelines

This section covers Rust-specific conventions
for the Asterinas project.
For language-agnostic principles, see
[General Guidelines](../general-guidelines/README.md).

Asterinas adheres to the
[Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
and the conventions described in the pages below.

## Pages

- **[Naming](naming.md)** —
  Getter naming, acronym capitalization,
  closure variable suffixes, boolean naming,
  and domain terminology.
- **Language Items**
  - **[Variables, Expressions, and Statements](language-items/variables-expressions-and-statements.md)** —
    Checked arithmetic, immutability, initialization,
    and conditional encapsulation.
  - **[Functions and Methods](language-items/functions-and-methods.md)** —
    Nesting, early returns, iterators, builders,
    helper extraction, function length, and flag arguments.
  - **[Types and Traits](language-items/types-and-traits.md)** —
    Generics, enums vs. trait objects,
    type-level invariants, and simplification.
  - **[Comments and Documentation](language-items/comments-and-documentation.md)** —
    Doc comment formatting, RFC 505, rustdoc links,
    module-level docs, dead code, and public API coverage.
  - **[Unsafety](language-items/unsafety.md)** —
    SAFETY comments, `# Safety` sections,
    scope minimization, and soundness reasoning.
  - **[Modules and Crates](language-items/modules-and-crates.md)** —
    Visibility, workspace dependencies, imports,
    and module organization.
  - **[Macros and Attributes](language-items/macros-and-attributes.md)** —
    Lint suppression, `#[expect]`, dead code policy,
    and `rustfmt` configuration.
  - **[Conditional Compilation](language-items/conditional-compilation.md)** —
    Architecture selection, feature gates,
    and runtime dispatch.
- **Select Topics**
  - **[Idiomatics](select-topics/idiomatics.md)** —
    The `?` operator, `Drop` cleanup, `impl Trait`,
    and established conventions.
  - **[Concurrency and Races](select-topics/concurrency-and-races.md)** —
    Lock ordering, atomics, memory ordering,
    spinlock discipline, and TOCTOU.
  - **[Error Handling](select-topics/error-handling.md)** —
    Error propagation, `return_errno_with_message!`,
    informative messages, and root-cause fixes.
  - **[Performance](select-topics/performance.md)** —
    Hot-path complexity, benchmarking,
    zero-cost abstractions, and avoiding premature optimization.
  - **[Logging](select-topics/logging.md)** —
    Log levels, message formatting,
    the `log` crate, and feature-gated verbose output.
  - **[Memory and Resource Management](select-topics/memory-and-resource-management.md)** —
    RAII, `Box::leak` avoidance, typed vs. untyped memory,
    and `unsafe` marking for memory-affecting functions.
  - **[Unit Testing](select-topics/unit-testing.md)** —
    Test naming, assertions, regression tests,
    and test code style.
