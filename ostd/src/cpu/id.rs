// SPDX-License-Identifier: MPL-2.0

//! CPU identification numbers.

pub use current::PinCurrentCpu;

/// The ID of a CPU in the system.
///
/// If converting from/to an integer, the integer must start from 0 and be less
/// than the number of CPUs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuId(u32);

impl CpuId {
    /// Creates a new instance.
    ///
    /// # Panics
    ///
    /// The given number must be within the range `[0..ostd::cpu::num_cpus()]`;
    /// otherwise, the method will panic.
    pub fn new(raw_id: u32) -> Self {
        assert!(raw_id < num_cpus() as u32);
        // SAFETY: the above assertion ensures that the raw number is valid.
        unsafe { Self::new_unchecked(raw_id) }
    }

    /// Creates a new instance.
    ///
    /// # Safety
    ///
    /// The given number must be within the range `[0..ostd::cpu::num_cpus()]`.
    pub(super) unsafe fn new_unchecked(raw_id: u32) -> Self {
        // Is this a good idea to mark this function as const?
        //
        // Not really. This is because
        // the number of CPU cores can only be determined at the run time.
        // As a result, it is impossible to use `CpuId::new_unchecked`
        // both statically and safely.

        Self(raw_id)
    }

    /// Returns the CPU ID of the bootstrap processor (BSP).
    ///
    /// The number for the BSP is always zero.
    pub const fn bsp() -> Self {
        // SAFETY:
        // (1) there is at least one CPU;
        // (2) BSP's `CURRENT_CPU` is assigned a value of 0.
        Self(0)
    }

    /// Converts the CPU ID to an `usize`.
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Converts the CPU ID to an `u32`.
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// The error type returned when converting an out-of-range integer to [`CpuId`].
#[derive(Debug, Clone, Copy)]
pub struct CpuIdFromIntError;

impl TryFrom<usize> for CpuId {
    type Error = CpuIdFromIntError;

    fn try_from(raw_id: usize) -> Result<Self, Self::Error> {
        if raw_id < num_cpus() {
            // SAFETY: The value is within the valid range of CPU IDs
            let new_self = unsafe { CpuId::new_unchecked(raw_id as u32) };
            Ok(new_self)
        } else {
            Err(CpuIdFromIntError)
        }
    }
}

/// Returns the number of CPUs.
pub fn num_cpus() -> usize {
    // SAFETY: As far as the safe APIs are concerned, `NUM_CPUS` is
    // read-only, so it is always valid to read.
    (unsafe { NUM_CPUS }) as usize
}

static mut NUM_CPUS: u32 = 1;

/// Returns an iterator over all CPUs.
pub fn all_cpus() -> impl Iterator<Item = CpuId> {
    (0..num_cpus()).map(|raw_id| {
        // SAFETY: the value is within the valid range of CPU IDs
        unsafe { CpuId::new_unchecked(raw_id as u32) }
    })
}

mod current {
    //! The current CPU ID.

    use super::CpuId;
    use crate::{cpu_local_cell, task::atomic_mode::InAtomicMode};

    /// A marker trait for guard types that can "pin" the current task to the
    /// current CPU.
    ///
    /// Such guard types include [`DisabledLocalIrqGuard`] and
    /// [`DisabledPreemptGuard`]. When such guards exist, the CPU executing the
    /// current task is pinned. So getting the current CPU ID or CPU-local
    /// variables are safe.
    ///
    /// # Safety
    ///
    /// The implementor must ensure that the current task is pinned to the current
    /// CPU while any one of the instances of the implemented structure exists.
    ///
    /// [`DisabledLocalIrqGuard`]: crate::irq::DisabledLocalIrqGuard
    /// [`DisabledPreemptGuard`]: crate::task::DisabledPreemptGuard
    pub unsafe trait PinCurrentCpu {
        /// Returns the ID of the current CPU.
        fn current_cpu(&self) -> CpuId {
            CpuId::current_racy()
        }
    }

    // SAFETY: A guard that enforces the atomic mode requires disabling any
    // context switching. So naturally, the current task is pinned on the CPU.
    unsafe impl<T: InAtomicMode> PinCurrentCpu for T {}
    unsafe impl PinCurrentCpu for dyn InAtomicMode + '_ {}

    impl CpuId {
        /// Returns the ID of the current CPU.
        ///
        /// This function is safe to call, but is vulnerable to races. The returned CPU
        /// ID may be outdated if the task migrates to another CPU.
        ///
        /// To ensure that the CPU ID is up-to-date, do it under any guards that
        /// implement the [`PinCurrentCpu`] trait.
        pub fn current_racy() -> Self {
            #[cfg(debug_assertions)]
            assert!(IS_CURRENT_CPU_INITED.load());

            let current_raw_id = CURRENT_CPU.load();
            // SAFETY: The CPU-local value is initialized to a correct one.
            unsafe { Self::new_unchecked(current_raw_id) }
        }
    }

    /// Initializes the module on the current CPU.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// 1. This method is called on each CPU during the early boot phase.
    /// 2. The provided CPU ID is correct.
    /// 3. The CPU-local storage on this CPU is safe to access.
    pub(super) unsafe fn init_on_cpu(current_cpu_id: u32) {
        // FIXME: If there are safe APIs that rely on the correctness of
        // the CPU ID for soundness, we'd better make the CPU ID a global
        // invariant and initialize it before entering `ap_early_entry`.
        CURRENT_CPU.store(current_cpu_id);

        #[cfg(debug_assertions)]
        IS_CURRENT_CPU_INITED.store(true);
    }

    cpu_local_cell! {
        /// The current CPU ID.
        pub(super) static CURRENT_CPU: u32 = 0;
        /// The initialization state of the current CPU ID.
        #[cfg(debug_assertions)]
        pub(super) static IS_CURRENT_CPU_INITED: bool = false;
    }
}

/// Initializes the CPU ID module (the BSP part).
///
/// # Safety
///
/// The caller must ensure that
/// 1. We're in the boot context of the BSP and APs have not yet booted.
/// 2. The given number of CPUs is correct.
/// 3. The CPU-local storage for BSP is safe to use.
pub(super) unsafe fn init_on_bsp(num_cpus: u32) {
    // SAFETY:
    // 1. The
    unsafe {
        current::init_on_cpu(0);
    }

    // SAFETY:
    // 1.
    unsafe {
        init_num_cpus(num_cpus);
    }
}

/// Initializes the number of CPUs.
///
/// # Safety
///
/// The caller must ensure that
/// 1. We're in the boot context of the BSP and APs have not yet booted.
/// 2. The argument is the correct value of the number of CPUs (which
///    is a constant, since we don't support CPU hot-plugging anyway).
unsafe fn init_num_cpus(num_cpus: u32) {
    assert!(num_cpus >= 1);

    // SAFETY: It is safe to mutate this global variable because we
    // are in the boot context.
    unsafe { NUM_CPUS = num_cpus };

    // Note that decreasing the number of CPUs may break existing
    // `CpuId`s (which have a type invariant to say that the ID is
    // less than the number of CPUs).
    //
    // However, this never happens: due to the safety conditions
    // it's only legal to call this function to increase the number
    // of CPUs from one (the initial value) to the actual number of
    // CPUs.
}

/// Initializes the CPU ID module (the AP part).
///
/// # Safety
///
/// The caller must ensure that:
/// 1. We're in the boot context of an AP.
/// 2. The given CPU ID of the AP is correct.
/// 3. The CPU-local storage for APs is safe to access.
pub(super) unsafe fn init_on_ap(cpu_id: u32) {
    // SAFETY: The required safety conditions are inherited from `init_on_ap`.
    unsafe {
        current::init_on_cpu(cpu_id);
    };
}
