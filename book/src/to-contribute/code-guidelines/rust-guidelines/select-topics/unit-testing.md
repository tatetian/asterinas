# Unit Testing (UT)

For general testing principles, see
[Testing Guidelines](../../testing-guidelines.md).

### UT1. Name tests after observable behavior

Name tests after the behavior
or specification concept being verified,
not after internal implementation details.
Use userspace-visible or specification-level terminology.

```rust
// Good — describes the observable behavior
#[test]
fn read_from_closed_pipe_returns_zero()

// Bad — exposes kernel internals
#[test]
fn pipe_channel_ref_count_drops()
```

### UT2. Use assertion macros

Use `assert_eq!`, `assert_ne!`, and `assert!` macros
instead of printing values
and inspecting output manually.
Add a descriptive panic message to `#[should_panic]` tests
so that an unexpected panic from a different location
does not silently pass the test:

```rust
#[test]
#[should_panic(expected = "index out of bounds")]
fn access_beyond_capacity_panics() {
    let buf = Buffer::new(16);
    let _ = buf[16];
}
```

### UT3. Require regression tests for every bug fix

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

### UT4. New syscalls need tests

When a new syscall is implemented:
(1) enable matching gvisor test suite entries;
(2) if none exist, add a manual test program
under `regression/apps/`.

### UT5. Test user-visible behavior, not implementation internals

Unit tests should validate observable, user-facing outcomes.
Testing implementation details makes tests brittle.
Avoid exposing kernel-internal constants in test code;
use userspace-visible equivalents instead
(e.g., `PROT_NONE` instead of internal `MAY_READ` flags).

### UT6. De-duplicate test helper code

Repeated boilerplate in test setup
should be extracted into shared helpers,
just like production code.

```rust
fn create_test_vmo(pages: usize) -> Vmo {
    VmoOptions::new(pages)
        .flags(VmoFlags::RESIZABLE)
        .alloc()
        .unwrap()
}

#[test]
fn vmo_read_after_write() {
    let vmo = create_test_vmo(1);
    // ... test-specific logic
}
```

### UT7. Structure tests with setup-action-assertion blocks

Group related setup, action, and assertion blocks
with blank lines between them.
Add brief phase comments for multi-step tests:

```rust
#[test]
fn truncate_extends_file_with_zeroes() {
    // Setup
    let file = create_temp_file();
    file.write_all(b"hello").unwrap();

    // Action
    file.set_len(4096).unwrap();

    // Assertion
    let mut buf = vec![0u8; 4096];
    file.read_exact(&mut buf).unwrap();
    assert_eq!(&buf[..5], b"hello");
    assert!(buf[5..].iter().all(|&b| b == 0));
}
```

### UT8. Use compile-time assertions for critical type invariants

Use `const _: () = assert!(...)` to verify at compile time
that types have expected sizes, alignments,
or other properties.
This catches layout regressions immediately
rather than at runtime.

```rust
// Good — compile-time check that the type fits in a page
const _: () = assert!(
    core::mem::size_of::<TaskContext>() <= 4096,
    "TaskContext must fit in a single page"
);

// Good — verify alignment matches hardware requirement
const _: () = assert!(
    core::mem::align_of::<PerCpuData>() >= 64,
    "PerCpuData must be cache-line aligned"
);
```
