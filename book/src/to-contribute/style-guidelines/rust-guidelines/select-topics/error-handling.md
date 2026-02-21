# Error Handling (E)

### E1. Propagate errors with `?`

Use the `?` operator
to propagate errors idiomatically.
Using `.unwrap()` in kernel code
where failure is legitimate is rejected.

```rust
pub(super) fn unlink(&self, name: &str) -> Result<()> {
    if is_dot_or_dotdot(name) {
        return_errno_with_message!(Errno::EINVAL, "unlink on . or ..");
    }
    // ... main logic at the top level
}
```

### E2. Use `return_errno_with_message!` for descriptive errors

Use the `return_errno_with_message!` macro
for returning errors with descriptive messages:

```rust
return_errno_with_message!(
    Errno::ENOTDIR,
    "the dentry is not related to a directory inode"
);
```

Prefer early returns to reduce nesting.

### E3. Do not hide bugs with `unwrap_or(false)`

When an operation cannot logically fail
given correct program state,
silently defaulting hides bugs.
Prefer an explicit `unwrap()` that panics loudly.

### E4. Error messages must be informative and consistent

Error messages passed to `Error::with_message`
or `expect()` must be descriptive
and semantically consistent
with the `Errno` they are paired with.

See [General Guidelines — Themes — Formatting](../../general-guidelines/themes.md#f1-format-error-messages-consistently)
for message formatting rules.

### E5. No blocking or fallible operations while holding spinlocks

Holding a spinlock while performing I/O
or fallible allocation risks deadlock.
Restructure code to release locks
before entering fallible code paths.

See also
[Concurrency and Races — CR3](concurrency-and-races.md#cr3-never-do-io-or-blocking-operations-while-holding-a-spinlock).

### E6. Fix root causes, not symptoms

When a bug or invariant violation is identified,
fix the root cause
rather than adding defensive checks
that paper over the symptom.
Unnecessary guards hide real bugs.

See also
[General Guidelines — A7](../../general-guidelines/themes.md#a7-fix-root-causes-not-symptoms).

### E7. Use `debug_assert` for correctness-only checks

Assertions verifying invariants
that should never fail in correct code
belong in `debug_assert!`, not `assert!`.
Release builds should not pay the cost.

```rust
debug_assert!(self.align.is_multiple_of(PAGE_SIZE));
debug_assert!(self.align.is_power_of_two());
```

### E8. Separate internal errors from external errors

Use `debug_assert!` for conditions
that represent programmer errors
(violated internal invariants)
and `Result`/`Errno` for conditions
that arise from user input or hardware behavior.

Syscall entry points are the validation boundary:
validate all user-supplied data at the boundary,
then trust it internally.
See [General Guidelines — A8](../../general-guidelines/themes.md#a8-validate-at-boundaries-trust-internally).
