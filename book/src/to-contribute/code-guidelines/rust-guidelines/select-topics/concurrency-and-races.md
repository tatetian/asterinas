# Concurrency and Races

Concurrency code is reviewed with extreme rigor.
Lock ordering, atomic correctness, memory ordering,
and race condition analysis are all demanded explicitly.

### Establish and enforce a consistent lock order

Two locks that can be acquired in different orders
by different code paths
is a potential deadlock.
Hierarchical lock order must be established and documented.

```rust
pub(super) fn set_control(
    self: Arc<Self>,
    process: &Process,
) -> Result<()> {
    // Lock order: group of process -> session inner -> job control
    let process_group_mut = process.process_group.lock();
    // ...
}
```

See also:
PR [#2445](https://github.com/asterinas/asterinas/pull/2445)
and [#2050](https://github.com/asterinas/asterinas/pull/2050).

### Never do I/O or blocking operations while holding a spinlock

Holding a spinlock while performing I/O
or blocking operations is a deadlock hazard.
Use a sleeping mutex or restructure
to drop the lock first.

See also:
PR [#925](https://github.com/asterinas/asterinas/pull/925)
and [#1521](https://github.com/asterinas/asterinas/pull/1521).

### Do not use atomics casually

When multiple atomic fields
must be updated in concert, use a lock.
Only use atomics when a single value
is genuinely independent.

### Critical sections must not be split across lock boundaries

Operations that must be atomic
(check + conditional action)
must happen under the same lock acquisition.
Moving a comparison outside the critical region
is a correctness bug.

See also:
PR [#2157](https://github.com/asterinas/asterinas/pull/2157)
and [#2277](https://github.com/asterinas/asterinas/pull/2277).

### Be mindful of drop ordering

When a value holds a lock guard or other RAII resource,
ensure that the drop order
does not create a use-after-free or deadlock.
Consider explicit `drop()` calls
when the default drop order is incorrect.
