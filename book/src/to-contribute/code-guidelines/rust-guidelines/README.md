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
  Getter naming, abbreviations, acronym capitalization,
  closure variable suffixes, and boolean naming.
- **[Language Items](language-items/README.md)**
  - **[Variables, Expressions, and Statements](language-items/variables-expressions-and-statements.md)** —
    Checked arithmetic, immutability,
    and explaining variables.
  - **[Functions and Methods](language-items/functions-and-methods.md)** —
    Nesting and early returns, helper extraction,
    function size, and flag arguments.
  - **[Types and Traits](language-items/types-and-traits.md)** —
    Enums vs. trait objects, type-level invariants,
    state simplification, and redundant `Option` elimination.
  - **[Comments and Documentation](language-items/comments-and-documentation.md)** —
    Doc comment formatting, RFC 505, rustdoc links,
    module-level docs, dead code, and public API coverage.
  - **[Unsafety](language-items/unsafety.md)** —
    SAFETY comments, `# Safety` sections,
    `deny(unsafe_code)` in kernel crates,
    scope minimization, and module-boundary reasoning.
  - **[Modules and Crates](language-items/modules-and-crates.md)** —
    Visibility, encapsulation, workspace dependencies,
    and import conventions.
  - **[Macros and Attributes](language-items/macros-and-attributes.md)** —
    Lint suppression, `#[expect]`, dead code policy,
    `rustfmt` configuration, and functions vs. macros.
  - **[Conditional Compilation](language-items/conditional-compilation.md)** —
    Architecture selection with `cfg` attributes.
- **[Select Topics](select-topics/README.md)**
  - **[Idiomatics](select-topics/idiomatics.md)** —
    The `?` operator and interior mutability primitives.
  - **[Concurrency and Races](select-topics/concurrency-and-races.md)** —
    Lock ordering, spinlock discipline, atomics,
    critical sections, and drop ordering.
  - **[Error Handling](select-topics/error-handling.md)** —
    Error propagation, informative messages,
    root-cause fixes, and `debug_assert`.
  - **[Performance](select-topics/performance.md)** —
    Hot-path complexity, unnecessary copies,
    and evidence-based optimization.
  - **[Logging](select-topics/logging.md)** —
    The `log` crate and appropriate log levels.
  - **[Memory and Resource Management](select-topics/memory-and-resource-management.md)** —
    RAII and `unsafe` marking
    for memory-safety-affecting functions.
  - **[Unit Testing](select-topics/unit-testing.md)** —
    Test naming, regression tests,
    and testing user-visible behavior.
