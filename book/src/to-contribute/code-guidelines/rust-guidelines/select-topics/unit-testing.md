# Unit Testing

For general testing principles, see
[Testing Guidelines](../../testing-guidelines.md).
This page covers Rust-specific conventions
using `#[test]` and `#[ktest]`.

### Name tests after observable behavior (`behavior-test-names`) {#behavior-test-names}

Name tests after the behavior
or specification concept being verified,
not after internal implementation details.
Use userspace-visible or specification-level terminology.

```rust
// Good — describes the observable behavior
#[test]
fn read_from_closed_pipe_returns_zero() { /* ... */ }

// Bad — exposes kernel internals
#[test]
fn pipe_channel_ref_count_drops() { /* ... */ }
```

See also:
[Testing Guidelines](../../testing-guidelines.md#behavior-names)
for the language-agnostic formulation;
PR [#2959](https://github.com/asterinas/asterinas/pull/2959)
and [#2962](https://github.com/asterinas/asterinas/pull/2962).

### Require regression tests for every bug fix (`regression-tests`) {#regression-tests}

When a bug is fixed,
a test that would have caught the bug
must accompany the fix.
Name regression tests after the issue
or behavior being verified.
Include a reference to the issue number
in a comment.

```rust
// Regression test for
// https://github.com/asterinas/asterinas/issues/1234
#[test]
fn mmap_fixed_does_not_unmap_adjacent_region() {
    // ...
}
```

See also:
[Testing Guidelines](../../testing-guidelines.md#require-regression-tests)
for the language-agnostic formulation;
PR [#2962](https://github.com/asterinas/asterinas/pull/2962)
and [#2450](https://github.com/asterinas/asterinas/pull/2450).

### Test user-visible behavior, not implementation internals (`test-visible-behavior`) {#test-visible-behavior}

Unit tests should validate observable, user-facing outcomes.
Testing implementation details makes tests brittle.
Avoid exposing kernel-internal constants in test code;
use userspace-visible equivalents instead
(e.g., `PROT_NONE` instead of internal `MAY_READ` flags).

See also:
[Testing Guidelines](../../testing-guidelines.md#test-public-apis)
for the language-agnostic formulation;
PR [#2926](https://github.com/asterinas/asterinas/pull/2926).
