# Memory and Resource Management

### Use RAII for all resource acquisition and release

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

See also:
PR [#164](https://github.com/asterinas/asterinas/pull/164)
and [#720](https://github.com/asterinas/asterinas/pull/720).

### Mark memory-safety-affecting functions `unsafe`

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

See also:
[Unsafety](../language-items/unsafety.md#document-safety-requirements).
