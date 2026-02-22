# Concurrency and Races

Concurrency code is reviewed with extreme rigor.
Lock ordering, atomic correctness, memory ordering,
and race condition analysis are all demanded explicitly.

### Establish and enforce a consistent lock order (`lock-ordering`) {#lock-ordering}

Acquiring two locks in different orders
from different code paths
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

### Never do I/O or blocking operations while holding a spinlock (`no-io-under-spinlock`) {#no-io-under-spinlock}

Holding a spinlock while performing I/O
or blocking operations is a deadlock hazard.
Use a sleeping mutex or restructure
to drop the lock first.

```rust
// Good — lock dropped before I/O
let data = {
    let guard = self.state.lock();
    guard.pending_data.clone()
};
self.device.write(&data)?;

// Bad — I/O under spinlock
let guard = self.state.lock();
self.device.write(&guard.pending_data)?;
```

See also:
PR [#925](https://github.com/asterinas/asterinas/pull/925)
and [#1521](https://github.com/asterinas/asterinas/pull/1521).

### Do not use atomics casually (`careful-atomics`) {#careful-atomics}

When multiple atomic fields
must be updated in concert, use a lock.
Only use atomics when a single value
is genuinely independent.

```rust
// Good — a lock protects correlated fields
struct Stats {
    inner: SpinLock<StatsInner>,
}
struct StatsInner {
    total_bytes: u64,
    total_packets: u64,
}

// Bad — two atomics that must be consistent
// but can be observed in an inconsistent state
struct Stats {
    total_bytes: AtomicU64,
    total_packets: AtomicU64,
}
```

### Critical sections must not be split across lock boundaries (`atomic-critical-sections`) {#atomic-critical-sections}

Operations that must be atomic
(check + conditional action)
must happen under the same lock acquisition.
Moving a comparison outside the critical region
is a correctness bug.

See also:
PR [#2157](https://github.com/asterinas/asterinas/pull/2157)
and [#2277](https://github.com/asterinas/asterinas/pull/2277).

### Be mindful of drop ordering (`drop-ordering`) {#drop-ordering}

When a value holds a lock guard or other RAII resource,
ensure that the drop order
does not create a use-after-free or deadlock.
Consider explicit `drop()` calls
when the default drop order is incorrect.

```rust
// Good — drop the guard before notifying
let data = {
    let guard = self.inner.lock();
    guard.clone_data()
};
// guard is dropped here, before we notify
self.waitqueue.wake_all();

// Bad — guard is held while notifying,
// which may cause a deadlock if the woken thread
// tries to acquire the same lock
let guard = self.inner.lock();
let data = guard.clone_data();
self.waitqueue.wake_all();
// guard dropped here — too late
```
