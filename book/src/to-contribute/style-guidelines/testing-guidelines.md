# Testing Guidelines

## Test Naming

Name tests after the behavior or specification concept being verified,
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

## Assertions

Use `assert_eq!`, `assert_ne!`, and `assert!` macros
(or their C-test-framework equivalents like `TEST_SUCC`)
instead of printing values and inspecting output manually.

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

## Test Cleanup

Always clean up resources after a test:
close file descriptors, unlink temporary files,
and call `waitpid` on child processes.
Leftover resources can cause flaky failures
in subsequent tests.

```c
// Good — cleanup after use
int fd = open("/tmp/test_file", O_CREAT | O_RDWR, 0644);
// ... test logic ...
close(fd);
unlink("/tmp/test_file");
```

## Avoid Duplication

Do not repeat identical assertions across test functions.
Extract shared setup, teardown, and common checks
into helper functions.

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

## Regression Tests

Name regression tests after the issue or behavior being verified.
Include a reference to the issue number or description
in a comment.

```rust
// Regression test for https://github.com/asterinas/asterinas/issues/1234
#[test]
fn mmap_fixed_does_not_unmap_adjacent_region() {
    // ...
}
```

Avoid exposing kernel-internal constants in test code.
Use userspace-visible equivalents instead
(e.g., `PROT_NONE` instead of internal `MAY_READ` flags).

## Test Code Style

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
