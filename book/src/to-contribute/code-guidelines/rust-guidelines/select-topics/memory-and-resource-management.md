# Memory and Resource Management (MR)

### MR1. Use RAII for all resource acquisition and release

Resources — IRQ enable/disable state, port numbers,
file handles, DMA buffers, lock guards —
must use the `Drop` trait for automatic cleanup.
Manual `enable()`/`disable()` call pairs are rejected.

```rust
// Good — RAII guard ensures IRQs are re-enabled
fn disable_local() -> DisabledLocalIrqGuard { ... }

impl Drop for DisabledLocalIrqGuard {
    fn drop(&mut self) {
        enable_local_irqs();
    }
}

// Bad — caller can forget to re-enable
fn disable_local_irqs() { ... }
fn enable_local_irqs() { ... }
```

Prefer lexical lifetimes
so the Rust compiler inserts `drop` automatically,
rather than calling `drop()` manually.

### MR2. Avoid `Box::leak` and `Arc` cycles

`Box::leak` to produce `'static` references
is a last resort.
Proper lifetime management should be used,
and the `'static` bound removed
when a shorter lifetime suffices.

`Arc` references in collections
that prevent `Drop` from running
(creating memory leaks) must use `Weak` instead.

```rust
// Bad — leaked memory is never freed
let config: &'static Config = Box::leak(Box::new(Config::new()));

// Good — use a Once<T> or LazyLock<T>
static CONFIG: LazyLock<Config> = LazyLock::new(Config::new);
```

### MR3. Distinguish typed and untyped memory

The codebase distinguishes
typed memory (kernel objects, metadata)
from untyped memory (DMA buffers, page cache pages).
APIs must not blur this distinction.

Functions that could
access or corrupt typed kernel memory
through raw addresses
must be marked `unsafe`.

### MR4. Mark memory-safety-affecting functions `unsafe`

Functions that access raw I/O memory,
write to MSRs, map physical addresses,
or could otherwise corrupt kernel state
must be marked `unsafe`
with appropriate `# Safety` documentation.

```rust
/// Reads a value from the given physical address.
///
/// # Safety
///
/// - `addr` must point to a valid, mapped physical memory region
///   of at least `size_of::<T>()` bytes.
/// - The memory region must not be typed kernel memory.
pub unsafe fn read_phys<T>(addr: usize) -> T { ... }
```

See also
[Unsafety — U2](../language-items/unsafety.md#u2-document-safety-requirements).

### MR5. Use `read_volatile`/`write_volatile` for all MMIO

Hardware registers and memory-mapped I/O
must be accessed via
`core::ptr::read_volatile`/`write_volatile`.
Normal reads and writes
may be elided or reordered by the compiler.
Note: volatile does NOT provide synchronization
(see [CR7](concurrency-and-races.md#cr7-volatile-does-not-fix-data-races)).

```rust
// Good — volatile access for MMIO registers
unsafe {
    let status = core::ptr::read_volatile(mmio_base.add(STATUS_OFFSET));
    core::ptr::write_volatile(mmio_base.add(COMMAND_OFFSET), cmd);
}

// Bad — compiler may optimize away or reorder
unsafe {
    let status = *mmio_base.add(STATUS_OFFSET);
    *mmio_base.add(COMMAND_OFFSET) = cmd;
}
```

### MR6. Provide fallible (`try_`) versions of allocating APIs

In kernel context, allocation can fail.
APIs that internally allocate
should provide a `try_` variant
that returns `Result` instead of panicking on OOM.
Use `Vec::try_reserve`, `Box::try_new` (when stabilized),
or manual allocation with error propagation.

```rust
// Good — caller can handle allocation failure
pub fn try_create(size: usize) -> Result<Self> {
    let mut buf = Vec::new();
    buf.try_reserve(size)?;
    Ok(Self { buf })
}

// Bad — panics on OOM, unacceptable in kernel
pub fn create(size: usize) -> Self {
    Self { buf: vec![0u8; size] }
}
```
