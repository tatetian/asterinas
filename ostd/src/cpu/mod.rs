// SPDX-License-Identifier: MPL-2.0

//! CPU-related definitions.

mod id;
pub mod local;
pub mod set;

pub use id::{all_cpus, num_cpus, CpuId, CpuIdFromIntError, PinCurrentCpu};

/// The CPU privilege level: user mode or kernel mode.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PrivilegeLevel {
    /// User mode.
    User = 0,
    /// Kernel mode.
    Kernel = 1,
}

/// Initializes the CPU module (the BSP part).
///
/// # Safety
///
/// The caller must ensure that
/// 1. We're in the boot context of the BSP and APs have not yet booted.
/// 2. The number of available processors is available.
/// 3. No CPU-local objects have been accessed.
pub(crate) unsafe fn init_on_bsp() {
    let num_cpus = crate::arch::boot::smp::count_processors().unwrap_or(1);

    // SAFETY: The safety conditions of `init_on_bsp` correspond to
    // whose of `local::copy_bsp_for_ap`.
    unsafe {
        local::copy_bsp_for_ap(num_cpus as usize);
    }

    // SAFETY:
    // 1. The #1 and #2 safety conditions of `init_on_bsp` correspond to
    // the first two of `id::init_on_bsp`.
    // 2. The previous step of `copy_bsp_for_ap` ensures that
    // the CPU-local storage is safe to use.
    unsafe {
        id::init_on_bsp(num_cpus);
    }
}

/// Initializes the CPU module (the AP part).
///
/// # Safety
///
/// The caller must ensure that:
/// 1. We're in the boot context of an AP.
/// 2. The given CPU ID of the AP is correct.
/// 3. The `init_on_bsp` is already invoked on the BSP.
pub(crate) unsafe fn init_on_ap(cpu_id: u32) {
    // SAFETY:
    // 1. The #1 and #2 safety conditions of `init_on_ap` correspond to
    // the first two of `id::init_on_ap`.
    // 2. The #3 safety condition of `init_on_ap` implies that
    // the CPU-local storage is safe to use.
    unsafe { id::init_on_ap(cpu_id) };
}
