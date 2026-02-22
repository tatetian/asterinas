# Functions and Methods

### Minimize nesting; use early returns and `let-else`

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

See also:
PR [#2877](https://github.com/asterinas/asterinas/pull/2877#discussion_r2685861741)
and [#2445](https://github.com/asterinas/asterinas/pull/2445#discussion_r2769320458).

### Extract coherent logic into named helpers

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

See also:
PR [#2929](https://github.com/asterinas/asterinas/pull/2929#discussion_r2757234577)
and [#2265](https://github.com/asterinas/asterinas/pull/2265#discussion_r2266214191).

### Keep functions small and focused

Each function should do one thing,
do it well, and do it only.
If you can extract another function from it
with a name that is not merely a restatement
of its implementation,
the original function is doing more than one thing.

Do not mix levels of abstraction:
a syscall handler should read like a specification;
byte-level manipulation belongs in a helper.

See also:
PR [#639](https://github.com/asterinas/asterinas/pull/639#discussion_r1524629393).

### Avoid flag arguments

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
