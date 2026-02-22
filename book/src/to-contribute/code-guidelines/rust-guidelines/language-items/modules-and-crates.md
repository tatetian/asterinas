# Modules and Crates

### Default to the narrowest visibility (`narrow-visibility`) {#narrow-visibility}

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

See also:
PR [#2951](https://github.com/asterinas/asterinas/pull/2951)
and [#2605](https://github.com/asterinas/asterinas/pull/2605#discussion_r2720506912).

### Encapsulate fields behind getters (`getter-encapsulation`) {#getter-encapsulation}

Do not make fields public
when a simple getter method would do.
The getter provides the right place
for naming conventions and future evolution.

```rust
// Good — field is private, accessed via getter
pub struct Vma {
    perms: VmPerms,
}

impl Vma {
    pub fn perms(&self) -> VmPerms {
        self.perms
    }
}

// Bad — public field exposes representation
pub struct Vma {
    pub perms: VmPerms,
}
```

### Use workspace dependencies (`workspace-deps`) {#workspace-deps}

Always declare shared dependencies
in the workspace `[workspace.dependencies]` table
and reference them with `.workspace = true`
in member crates.

```toml
# In the workspace root Cargo.toml
[workspace.dependencies]
ostd = { version = "0.17.0", path = "ostd" }
bitflags = "2.6"

# In a member crate's Cargo.toml
[dependencies]
ostd.workspace = true
bitflags.workspace = true
```

### Follow the three-group import convention (`import-groups`) {#import-groups}

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
