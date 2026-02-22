# Unsafety

### Justify every use of `unsafe` (`safety-comments`) {#safety-comments}

Every `unsafe` block must have a preceding `// SAFETY:` comment
that justifies why the operation is sound.
For multi-condition invariants,
use a numbered list:

```rust
// SAFETY:
// 1. We have exclusive access to both the current context
//    and the next context (see above).
// 2. The next context is valid (because it is either
//    correctly initialized or written by a previous
//    `context_switch`).
unsafe {
    context_switch(next_task_ctx_ptr, current_task_ctx_ptr);
}
```

See also:
PR [#2958](https://github.com/asterinas/asterinas/pull/2958)
and [#836](https://github.com/asterinas/asterinas/pull/836).

### Document safety requirements (`safety-docs`) {#safety-docs}

All `unsafe` functions and traits
must include a `# Safety` section in their doc comments
describing the properties or invariants that callers must uphold.
State exactly what the caller must guarantee —
not implementation details or side effects.

```rust
/// Reads a value from the given physical address.
///
/// # Safety
///
/// The caller must ensure that `addr` points to
/// a valid, mapped physical memory region of at least
/// `size_of::<T>()` bytes.
pub unsafe fn read_phys<T>(addr: usize) -> T { ... }
```

### Deny unsafe code in `kernel/` (`deny-unsafe-kernel`) {#deny-unsafe-kernel}

All crates under `kernel/` must deny unsafe:

```rust
#![deny(unsafe_code)]
```

Only OSTD (`ostd/`) crates may contain `unsafe` code.
If a kernel crate requires an unsafe operation,
the functionality should be provided as a safe API in OSTD.

See also:
PR [#2498](https://github.com/asterinas/asterinas/pull/2498)
and [#2012](https://github.com/asterinas/asterinas/pull/2012).

### Minimize and encapsulate `unsafe` scope (`minimize-unsafe`) {#minimize-unsafe}

Repeated `unsafe` patterns
should be wrapped in a single safe or unsafe helper.
Callers should never repeat
the same safety reasoning in multiple places.

```rust
// Bad — same unsafe block repeated four times
unsafe { port.read_u8() } // SAFETY: ...
unsafe { port.read_u8() } // SAFETY: ...

// Good — wrap in a safe method, justify once
fn read_port(&self) -> u8 {
    // SAFETY: The port address is validated at construction.
    unsafe { self.port.read_u8() }
}
```

See also:
PR [#2958](https://github.com/asterinas/asterinas/pull/2958)
and [#2498](https://github.com/asterinas/asterinas/pull/2498).

### Reason about safety at the module boundary (`module-boundary-safety`) {#module-boundary-safety}

The safety of an `unsafe` block
depends on ALL code that can access the same private state.
Encapsulate unsafe abstractions
in the smallest possible module
to minimize the "audit surface."
Any code in the same module
that can modify relied-upon fields
is part of the safety argument.

```rust
// Good — small, focused module limits the audit surface
mod frame_allocator {
    /// Invariant: `next` is always a valid frame index.
    struct FrameAlloc {
        next: usize,
        // ...
    }

    impl FrameAlloc {
        pub fn alloc(&mut self) -> PhysAddr {
            // SAFETY: `next` is always valid (see invariant above).
            // Only code in this module can modify `next`.
            unsafe { self.alloc_frame_unchecked(self.next) }
        }
    }
}
```
