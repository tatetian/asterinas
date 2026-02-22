# Concurrency and Races (CR)

Concurrency code is reviewed with extreme rigor.
Lock ordering, atomic correctness, memory ordering,
and race condition analysis are all demanded explicitly.

### CR1. Establish and enforce a consistent lock order

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

### CR2. Protect concurrent state with appropriate locks

Using atomics without careful analysis
of all race conditions is rejected.
Shared mutable state must be protected
by a proper lock or justified atomics
with correct ordering.

### CR3. Never do I/O or blocking operations while holding a spinlock

Holding a spinlock while performing I/O
or blocking operations is a deadlock hazard.
Use a sleeping mutex or restructure
to drop the lock first.

### CR4. Do not use atomics casually

When multiple atomic fields
must be updated in concert, use a lock.
Only use atomics when a single value
is genuinely independent.

### CR5. Use the weakest correct memory ordering

Atomic memory orderings
should be the weakest that is provably correct.
`Relaxed` for simple counters;
`Acquire`/`Release` only for establishing
happens-before relationships.

### CR6. Critical sections must not be split across lock boundaries

Operations that must be atomic
(check + conditional action)
must happen under the same lock acquisition.
Moving a comparison outside the critical region
is a correctness bug.

### CR7. `volatile` does not fix data races

`read_volatile`/`write_volatile`
do not resolve data races
and do not provide acquire/release memory ordering.
Proper synchronization is required.

### CR8. Be mindful of drop ordering

When a value holds a lock guard or other RAII resource,
ensure that the drop order
does not create a use-after-free or deadlock.
Consider explicit `drop()` calls
when the default drop order is incorrect.

### CR9. Check-before-wait fast paths must not introduce TOCTOU races

The optimization of checking a condition
before registering a waiter
must also check after registering the waiter
to avoid losing wakeup events.

### CR10. Prevent false sharing on per-CPU data

When per-CPU counters or state
are stored in adjacent memory,
they may share a cache line.
Pad with `#[repr(align(64))]`
when benchmarks show contention.

```rust
// Good — each CPU's counter occupies its own cache line
#[repr(align(64))]
struct PerCpuCounter {
    value: AtomicU64,
}

static COUNTERS: [PerCpuCounter; MAX_CPUS] = /* ... */;
```

### CR11. Use `compare_exchange_weak` in retry loops

`compare_exchange_weak` is more efficient
on architectures with LL/SC (e.g., ARM, RISC-V).
When already in a retry loop,
always prefer the weak variant.

```rust
// Good — weak variant in a loop
loop {
    let current = counter.load(Ordering::Relaxed);
    match counter.compare_exchange_weak(
        current,
        current + 1,
        Ordering::AcqRel,
        Ordering::Relaxed,
    ) {
        Ok(_) => break,
        Err(_) => continue,
    }
}

// Bad — strong variant wastes cycles
// when the loop already retries
loop {
    let current = counter.load(Ordering::Relaxed);
    if counter
        .compare_exchange(
            current,
            current + 1,
            Ordering::AcqRel,
            Ordering::Relaxed,
        )
        .is_ok()
    {
        break;
    }
}
```
