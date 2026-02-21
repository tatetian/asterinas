# Functions and Methods (FM)

### FM1. Minimize nesting; use early returns and `let-else`

Minimize nesting depth.
Code nested more than three levels deep
should be reviewed for refactoring opportunities.
Each nesting level multiplies the reader's cognitive load.

Techniques for flattening nesting:
- Early returns and guard clauses for error paths.
- `let...else` to collapse `if let` chains.
- The `?` operator for error propagation.
- `continue` to skip loop iterations.
- Extracting the nested body into a helper function.

The normal/expected code path
should be the first visible path;
error and edge cases
should be handled early and gotten out of the way.

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

For each function, ask:
"What is the high-level goal?"
Code not directly related to that goal
is an _unrelated subproblem_
and a candidate for extraction —
even if it appears only once.

```rust
// Good — syscall handler reads like a specification
pub fn sys_mmap(addr: Vaddr, len: usize, ...) -> Result<Vaddr> {
    let options = validate_mmap_params(addr, len, prot, flags)?;
    let vmar = current_process().root_vmar();
    vmar.create_mapping(options)
}
```

### FM4. Keep functions small and focused

Each function should do one thing,
do it well, and do it only.
If you can extract another function from it
with a name that is not merely a restatement
of its implementation,
the original function is doing more than one thing.

Do not mix levels of abstraction:
a syscall handler should read like a specification;
byte-level manipulation belongs in a helper.

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

### FM7. Limit function length

Functions should generally be under 50 lines.
Functions over 50 lines
should be reviewed for extraction opportunities.
Functions over 100 lines
require summary comments for each logical section.

Legitimate exceptions include
initialization sequences, state machines,
and `match` expressions on large enums.
These are guidelines, not hard limits.

### FM8. Avoid flag arguments

A boolean parameter that selects between
two behaviors signals the function does two things.
Split it into two functions
or use a typed enum.

```rust
// Good — two separate functions
fn read_blocking(&self, buf: &mut [u8]) -> Result<usize> { ... }
fn read_nonblocking(&self, buf: &mut [u8]) -> Result<usize> { ... }

// Good — typed enum
enum ReadMode { Blocking, NonBlocking }
fn read(&self, buf: &mut [u8], mode: ReadMode) -> Result<usize> { ... }

// Bad — boolean flag
fn read(&self, buf: &mut [u8], blocking: bool) -> Result<usize> { ... }
```
