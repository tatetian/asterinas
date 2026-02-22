# Performance (P)

Performance on critical paths is taken very seriously.
Changes to hot paths must be benchmarked.
Unnecessary copies, allocations,
and O(n) algorithms are rejected.

### P1. Avoid O(n) algorithms on hot paths

System call dispatch, scheduler enqueue,
and frequent query operations
must not introduce O(n) complexity
where n is a quantity that grows over time
(number of CPUs, number of processes, etc.).
Demand sub-linear alternatives.

```rust
// Bad — O(n) scan on every enqueue
fn select_cpu(&self, cpus: &[CpuState]) -> CpuId {
    cpus.iter()
        .min_by_key(|c| c.load())
        .unwrap()
        .id()
}

// Good — maintain a priority queue
// so selection is O(log n)
fn select_cpu(&self) -> CpuId {
    self.cpu_heap.peek().unwrap().id()
}
```

### P2. Benchmark before changing critical paths

Any change to a syscall dispatch path,
lock-acquisition path, or scheduler path
must be validated with a concrete benchmark
before acceptance.
Use LMbench (`getpid`, `getppid`),
custom microbenchmarks,
or the project's existing benchmark suite.

### P3. Minimize unnecessary copies and allocations

Extra data copies —
serializing to a stack buffer before writing,
cloning an `Arc` when a `&` reference suffices,
collecting into a `Vec` when an iterator would do —
should be avoided.

```rust
// Bad — unnecessary Arc::clone
fn process(&self, stream: Arc<DmaStream>) {
    let s = stream.clone();
    s.sync()?;
}

// Good — borrow when ownership is not needed
fn process(&self, stream: &DmaStream) {
    stream.sync()?;
}
```

### P4. Prefer zero-cost by default, opt-in overhead

Abstractions should carry no cost
when the expensive feature is not used.
The zero-cost case should be the default.

```rust
// Good — no overhead when filter is not needed
pub struct Subject<F: Filter = NoFilter> { ... }

// Bad — every instance pays for Option<F>
// even when filters are never used
pub struct Subject {
    filter: Option<Box<dyn Filter>>,
}
```

### P5. No premature optimization without evidence

Performance optimizations
must be justified with data.
Introducing complexity
to solve a non-existent problem is rejected.
If you claim a change improves performance,
show the numbers.

### P6. Prefer `mem::take`/`mem::replace` over `.clone()`

When the source value is no longer needed,
use `mem::take` or move the value out
instead of cloning.
Use `mem::replace`
when you need to take ownership
from behind a `&mut` reference.

```rust
use core::mem;

// Good — moves the value out without cloning
fn drain_name(entry: &mut Entry) -> String {
    mem::take(&mut entry.name)
}

// Bad — clones when the original is about to be overwritten
fn drain_name(entry: &mut Entry) -> String {
    let name = entry.name.clone();
    entry.name = String::new();
    name
}
```
