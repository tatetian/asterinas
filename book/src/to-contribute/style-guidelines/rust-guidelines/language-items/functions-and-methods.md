# Functions and Methods (FM)

### FM1. Use early returns and `let-else` to flatten nesting

Multiple consecutive checks or redundant `if let` chains
should be collapsed into a single early return
using `let...else`.
Flat code is easier to follow
than deeply nested code.

```rust
pub(crate) fn init() {
    let Some(framebuffer_arg) = boot_info().framebuffer_arg else {
        log::warn!("Framebuffer not found");
        return;
    };
    // ... main logic at the top level
}
```

### FM2. Prefer iterators over collecting into `Vec`

Expose an `iter()` method
rather than a method that collects all items into a `Vec`.
Iterators give callers flexibility
to choose between holding a lock while iterating
or collecting into a container.

```rust
// Good — returns an iterator
pub fn fds_and_files(
    &self,
) -> impl Iterator<Item = (FileDesc, &'_ Arc<dyn FileLike>)> {
    self.table
        .idxes_and_items()
        .map(|(idx, entry)| (idx as FileDesc, entry.file()))
}

// Bad — forces allocation
pub fn fds_and_files(&self) -> Vec<(FileDesc, Arc<dyn FileLike>)> {
    // ...
}
```

### FM3. Extract coherent logic into named helpers

Long or complex sequences forming a coherent sub-task
should be extracted into named helper functions
with clear signatures.
Even two or three lines are worth extracting
if they represent a distinct concept
with a descriptive name.

### FM4. Keep functions small and focused

Each function should do one thing,
do it well, and do it only.
If you can extract another function from it
with a name that is not merely a restatement
of its implementation,
the original function is doing more than one thing.

### FM5. Use the builder pattern for complex construction

When constructing a struct involves many fields
or optional configuration,
the builder pattern avoids requiring callers
to fill every field directly.

```rust
let vmo = VmoOptions::new(page_count)
    .flags(VmoFlags::RESIZABLE)
    .alloc()
    .unwrap();
```

### FM6. Place new methods at a logical position

Insert new methods at a logical position
within the existing `impl` block —
not at the very end by default.
Group related methods together
and separate groups with blank lines.
