# Error Handling

### Propagate errors with `?` (`propagate-errors`) {#propagate-errors}

Use the `?` operator
to propagate errors idiomatically.
Using `.unwrap()` in kernel code
where failure is legitimate is rejected.

```rust
// Good — propagate with ?
let tsc_info = cpuid.get_tsc_info()?;
let frequency = tsc_info.nominal_frequency()?;

// Bad — unwrap hides the failure path
let tsc_info = cpuid.get_tsc_info().unwrap();
```

See also:
[Idiomatics](../select-topics/idiomatics.md#question-mark-operator)
for additional context on the `?` operator.

### Error messages must be informative and consistent (`informative-errors`) {#informative-errors}

Error messages passed to `Error::with_message`
or `expect()` must be descriptive
and semantically consistent
with the `Errno` they are paired with.

See [General Guidelines — Formatting](../../general-guidelines/README.md#error-message-format)
for message formatting rules.

### Fix root causes, not symptoms (`rust-fix-root-causes`) {#rust-fix-root-causes}

When a bug or invariant violation is identified,
fix the root cause
rather than adding defensive checks
that paper over the symptom.
Unnecessary guards hide real bugs.

See also:
[General Guidelines](../../general-guidelines/README.md#fix-root-causes)
and PR [#2741](https://github.com/asterinas/asterinas/pull/2741).

### Use `debug_assert` for correctness-only checks (`debug-assert`) {#debug-assert}

Assertions verifying invariants
that should never fail in correct code
belong in `debug_assert!`, not `assert!`.
Release builds should not pay the cost.

```rust
debug_assert!(self.align.is_multiple_of(PAGE_SIZE));
debug_assert!(self.align.is_power_of_two());
```
