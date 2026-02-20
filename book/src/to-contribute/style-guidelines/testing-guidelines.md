# Testing Guidelines

This page covers language-agnostic testing conventions.
For Rust-specific testing conventions, see
[Rust Guidelines — Testing](rust-guidelines.md#testing).

## Test Naming

Name tests after the behavior or specification concept being verified,
not after internal implementation details.
Use userspace-visible or specification-level terminology
so that a failing test name immediately conveys
what went wrong.

## Assertions

Use assertion macros or framework-provided assertion functions
instead of printing values and inspecting output manually.
Assertions provide clear failure messages
and make tests self-checking.

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

## Regression Tests

Name regression tests after the issue or behavior being verified.
Include a reference to the issue number or description
in a comment so that future readers can find the original context.
Prefer testing through public APIs
rather than exposing internal constants in test code.

## Test Code Style

Group related setup, action, and assertion blocks
with blank lines between them.
Add brief phase comments for multi-step tests
to make the structure of each test obvious.
