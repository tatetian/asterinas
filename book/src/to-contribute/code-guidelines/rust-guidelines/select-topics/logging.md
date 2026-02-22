# Logging

### Use `log` crate macros exclusively

The project standardizes on the
[`log`](https://docs.rs/log) crate's macros:
`trace!`, `debug!`, `info!`, `warn!`, `error!`.
Custom output functions, `println!`,
and hand-rolled serial print macros
are not acceptable in production code.

### Choose appropriate log levels

| Level | Use for |
|-------|---------|
| `trace!` | High-frequency events: every interrupt, every packet, every page fault. |
| `debug!` | Development diagnostics: state transitions, intermediate values. |
| `info!` | Rare, noteworthy events: subsystem initialization, configuration changes. |
| `warn!` | Recoverable problems: fallback paths taken, deprecated usage detected. |
| `error!` | Serious failures: resource exhaustion, invariant violations caught at runtime. |

A log statement that fires on every syscall
or every timer tick must use `trace!`, not `debug!`.

See also:
PR [#2260](https://github.com/asterinas/asterinas/pull/2260)
and [#2542](https://github.com/asterinas/asterinas/pull/2542).
