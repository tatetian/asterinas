# Modules and Crates (MC)

### MC1. Default to the narrowest visibility

Start private,
then widen to `pub(super)`, `pub(crate)`, or `pub`
only when an actual external consumer requires it.

```rust
// Good — restricted to the parent module
pub(super) static I8042_CONTROLLER:
    Once<SpinLock<I8042Controller, LocalIrqDisabled>> = Once::new();

pub(super) fn init() -> Result<(), I8042ControllerError> {
    // ...
}

// Bad — unnecessarily wide
pub static I8042_CONTROLLER: ...
```

### MC2. Avoid redundant visibility qualifiers

Adding `pub(crate)` to methods
of a `pub(crate)` type is redundant noise.
Similarly, `pub` on items inside a private module
only affects accessibility within the item,
not from outside.

### MC3. Encapsulate fields behind getters

Do not make fields public
when a simple getter method would do.
The getter provides the right place
for naming conventions and future evolution.

### MC4. Use workspace dependencies

Always declare shared dependencies
in the workspace `[workspace.dependencies]` table
and reference them with `.workspace = true`
in member crates.

```toml
# In the workspace root Cargo.toml
[workspace.dependencies]
ostd = { version = "0.17.0", path = "ostd" }
bitflags.workspace = true

# In a member crate's Cargo.toml
[dependencies]
ostd.workspace = true
bitflags.workspace = true
```

### MC5. Follow the three-group import convention

Imports follow a three-group pattern
enforced by the `rustfmt.toml` settings:

1. Standard library (`core`, `alloc`, `std`)
2. External crates
3. Crate-local (`super::`, `crate::`)

Each group is separated by a blank line.
Items from the same crate are merged
into a single `use` statement with nested braces.

```rust
use core::{
    ops::Deref,
    sync::atomic::{AtomicU32, Ordering},
};

use hashbrown::HashMap;
use ostd::sync::RwMutexWriteGuard;

use super::{is_dot, is_dot_or_dotdot, is_dotdot};
use crate::{
    fs::{
        self,
        utils::{Inode, InodeExt, InodeMode, InodeType, MknodType},
    },
    prelude::*,
};
```

### MC6. One logical concept per file

When a file grows long
or contains multiple distinct concepts, split it.
Each major data structure or subsystem entry point
deserves its own file.

### MC7. Module hierarchy mirrors logical relationships

Directory and module structure
should reflect conceptual relationships.
Tightly coupled types
belong under a shared parent module.

### MC8. Place symbols at the correct abstraction layer

Functions belonging to a specific subsystem
should be placed in that subsystem's module,
not hoisted to the crate root.
