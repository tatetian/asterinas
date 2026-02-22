# Logging (LG)

### LG1. Use `log` crate macros exclusively

The project standardizes on the
[`log`](https://docs.rs/log) crate's macros:
`trace!`, `debug!`, `info!`, `warn!`, `error!`.
Custom output functions, `println!`,
and hand-rolled serial print macros
are not acceptable in production code.

### LG2. Choose appropriate log levels

| Level | Use for |
|-------|---------|
| `trace!` | High-frequency events: every interrupt, every packet, every page fault. |
| `debug!` | Development diagnostics: state transitions, intermediate values. |
| `info!` | Rare, noteworthy events: subsystem initialization, configuration changes. |
| `warn!` | Recoverable problems: fallback paths taken, deprecated usage detected. |
| `error!` | Serious failures: resource exhaustion, invariant violations caught at runtime. |

A log statement that fires on every syscall
or every timer tick must use `trace!`, not `debug!`.

### LG3. Format log messages consistently

- Start with a lowercase letter
  (unless the first word is a proper noun or identifier).
- Include relevant identifiers
  and state values.
- Match the format
  of neighboring log statements in the same file.

```rust
// Good — lowercase, informative, consistent
log::debug!(
    "i8042 keyboard unmapped scancode {:?} dropped",
    scancode
);

// Bad — inconsistent casing, vague
log::debug!("Dropped something");
```

### LG4. Remove debug prints before merging

Code containing `[DEBUG]` string prefixes,
raw `println!`,
or other temporary output
must be cleaned up before merge.

### LG5. Use feature-gated logging for verbose subsystem output

Logging too expensive or too verbose
for even debug builds
should be behind a compile-time feature flag
for zero overhead in production.

```rust
macro_rules! sched_debug {
    ($($arg:tt)*) => {
        #[cfg(feature = "sched_debug")]
        log::debug!($($arg)*);
    };
}
```
