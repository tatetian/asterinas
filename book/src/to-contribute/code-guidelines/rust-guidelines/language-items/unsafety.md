# Unsafety (U)

### U1. Justify every use of `unsafe`

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

### U2. Document safety requirements

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

### U3. Deny unsafe code in `kernel/`

All crates under `kernel/` must deny unsafe:

```rust
#![deny(unsafe_code)]
```

Only OSTD (`ostd/`) crates may contain `unsafe` code.
If a kernel crate requires an unsafe operation,
the functionality should be provided as a safe API in OSTD.

### U4. Minimize and encapsulate `unsafe` scope

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

### U5. Soundness must be argued, not assumed

Implementors must reason explicitly about soundness.
A `// SAFETY:` comment that says "this is safe" without evidence
is not acceptable.

### U6. Unsafe traits must have well-defined invariants

When a trait is `unsafe` to implement,
the doc must clearly state the invariants
the implementor must uphold.
Safety requirements should be
stricter than all types that implement the trait,
rather than claiming the requirement
depends on the concrete type.

### U7. Do not expose `unsafe` functions in the public API

Unsafe functions must not appear
in the public API surface of OSTD
or any crate with external users.
Give such methods `pub(crate)` visibility at most.

### U8. Reason about safety at the module boundary

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

### U9. Ensure panic safety when temporarily violating invariants

If an `unsafe` block temporarily violates an invariant,
ensure a panic at any point
does not leave data in an invalid state.
Use drop guards
(a local struct implementing `Drop`)
to restore invariants
if a closure or `?` causes early exit.
This is critical in kernel code where panics may be caught.

```rust
// Good — drop guard restores the length
// even if the closure panics
struct SetLenOnDrop<'a> {
    vec: &'a mut Vec<u8>,
    len: usize,
}

impl Drop for SetLenOnDrop<'_> {
    fn drop(&mut self) {
        // SAFETY: `len` is always <= capacity.
        unsafe { self.vec.set_len(self.len); }
    }
}
```

For more on writing sound unsafe code,
see [The Rustonomicon](https://doc.rust-lang.org/nomicon/).
