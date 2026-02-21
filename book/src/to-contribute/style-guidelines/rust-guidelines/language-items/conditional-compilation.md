# Conditional Compilation (CC)

### CC1. Use `cfg` attributes for architecture-specific modules

When a module's implementation differs by architecture,
use `cfg_attr` to select the correct source file:

```rust
#[cfg_attr(target_arch = "x86_64", path = "arch/x86/mod.rs")]
#[cfg_attr(target_arch = "riscv64", path = "arch/riscv/mod.rs")]
#[cfg_attr(target_arch = "loongarch64", path = "arch/loongarch/mod.rs")]
pub mod arch;
```

### CC2. Use feature gates for verbose logging

Logging too expensive or verbose
for even debug builds
uses feature-flag-gated compile-time macros
for zero-overhead compilation in production.

```rust
#[cfg(feature = "log_color")]
fn format_log(record: &Record) {
    // ... color formatting logic ...
}

#[cfg(not(feature = "log_color"))]
fn format_log(record: &Record) {
    // ... plain formatting logic ...
}
```

### CC3. Prefer runtime dispatch for environment-dependent features

When a feature's availability
depends on the runtime environment
(e.g., hardware capabilities),
use runtime dispatch rather than compile-time feature flags.
Feature flags are appropriate
for build-time configuration choices,
not for hardware detection.

### CC4. Declare per-architecture drivers with macros

When multiple architectures
provide different driver implementations,
use a declarative macro
to select the correct driver at compile time:

```rust
declare_rtc_drivers! {
    #[cfg(target_arch = "x86_64")] cmos::RtcCmos,
    #[cfg(target_arch = "riscv64")] goldfish::RtcGoldfish,
    #[cfg(target_arch = "loongarch64")] loongson::RtcLoongson,
}
```
