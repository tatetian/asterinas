# Testing Guidelines

This page covers language-agnostic testing conventions.
For Rust-specific testing conventions, see
[Rust Guidelines — Unit Testing](rust-guidelines/select-topics/unit-testing.md).

## Naming

### Name tests after observable behavior (`behavior-names`) {#behavior-names}

Name tests after the behavior or specification concept
being verified,
not after internal implementation details.
Use userspace-visible or specification-level terminology
so that a failing test name immediately conveys
what went wrong.

See also:
PR [#2959](https://github.com/asterinas/asterinas/pull/2959)
and [#2962](https://github.com/asterinas/asterinas/pull/2962).

## Structure

### Use assertion macros, not manual inspection (`use-assertions`) {#use-assertions}

Use assertion macros or framework-provided assertion functions
instead of printing values and inspecting output manually.
Assertions provide clear failure messages
and make tests self-checking.

See also:
PR [#2877](https://github.com/asterinas/asterinas/pull/2877)
and [#2926](https://github.com/asterinas/asterinas/pull/2926).

### Clean up resources after every test (`test-cleanup`) {#test-cleanup}

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

See also:
PR [#2926](https://github.com/asterinas/asterinas/pull/2926)
and [#2969](https://github.com/asterinas/asterinas/pull/2969).

## Coverage

### Require regression tests for every bug fix (`require-regression-tests`) {#require-regression-tests}

When a bug is fixed,
a test that would have caught the bug
must accompany the fix.
Name regression tests after the issue
or behavior being verified.
Include a reference to the issue number
in a comment so that future readers
can find the original context.

See also:
PR [#2962](https://github.com/asterinas/asterinas/pull/2962)
and [#2450](https://github.com/asterinas/asterinas/pull/2450).

### Test user-visible behavior, not internals (`test-public-apis`) {#test-public-apis}

Tests should validate observable, user-facing outcomes.
Prefer testing through public APIs
rather than exposing internal constants in test code.
Using kernel-internal names in user-space regression tests
creates unnecessary coupling.

See also:
PR [#2926](https://github.com/asterinas/asterinas/pull/2926).
