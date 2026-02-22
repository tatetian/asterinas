# Conditional Compilation

### Use `cfg` attributes for architecture-specific modules

When a module's implementation differs by architecture,
use `cfg_attr` to select the correct source file:

```rust
#[cfg_attr(target_arch = "x86_64", path = "arch/x86/mod.rs")]
#[cfg_attr(target_arch = "riscv64", path = "arch/riscv/mod.rs")]
#[cfg_attr(target_arch = "loongarch64", path = "arch/loongarch/mod.rs")]
pub mod arch;
```
