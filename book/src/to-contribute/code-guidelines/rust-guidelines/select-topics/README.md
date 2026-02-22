# Select Topics

This section covers cross-cutting concerns
that span multiple language items.
Each page addresses a specific topic
with numbered, actionable guidelines.

- **[Idiomatics](idiomatics.md)** —
  The `?` operator, `Drop` cleanup, `impl Trait`,
  and established conventions.
- **[Concurrency and Races](concurrency-and-races.md)** —
  Lock ordering, atomics, memory ordering,
  spinlock discipline, and TOCTOU.
- **[Error Handling](error-handling.md)** —
  Error propagation, `return_errno_with_message!`,
  informative messages, and root-cause fixes.
- **[Performance](performance.md)** —
  Hot-path complexity, benchmarking,
  zero-cost abstractions, and avoiding premature optimization.
- **[Logging](logging.md)** —
  Log levels, message formatting,
  the `log` crate, and feature-gated verbose output.
- **[Memory and Resource Management](memory-and-resource-management.md)** —
  RAII, `Box::leak` avoidance, typed vs. untyped memory,
  and `unsafe` marking for memory-affecting functions.
- **[Unit Testing](unit-testing.md)** —
  Test naming, assertions, regression tests,
  and test code style.
