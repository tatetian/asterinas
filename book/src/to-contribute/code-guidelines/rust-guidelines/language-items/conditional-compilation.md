# Conditional Compilation

### Use `cfg_attr` to select architecture-specific modules (`cfg-arch-modules`) {#cfg-arch-modules}

When a module's implementation differs by architecture,
use `cfg_attr` to select the correct source file:

```rust
#[cfg_attr(target_arch = "x86_64", path = "arch/x86/mod.rs")]
#[cfg_attr(target_arch = "riscv64", path = "arch/riscv/mod.rs")]
#[cfg_attr(target_arch = "loongarch64", path = "arch/loongarch/mod.rs")]
pub mod arch;
```
