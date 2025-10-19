// SPDX-License-Identifier: MPL-2.0

//! This module contains the implementation of the CPU set and atomic CPU set.

use core::sync::atomic::Ordering;

use inherit_methods_macro::inherit_methods;

use super::CpuId;
use crate::util::id_set::{AtomicIdSet, IdSet};

/// A subset of all CPUs in the system.
#[derive(Clone, Debug, Default)]
pub struct CpuSet(IdSet<CpuId>);

impl CpuSet {
    /// Creates a new `CpuSet` with all CPUs in the system.
    #[inline]
    pub fn new_full() -> Self {
        Self(IdSet::new_full())
    }

    /// Creates a new `CpuSet` with no CPUs in the system.
    #[inline]
    pub fn new_empty() -> Self {
        Self(IdSet::new_empty())
    }
}

#[inherit_methods(from = "self.0")]
impl CpuSet {
    /// Adds a CPU to the set.
    #[inline]
    pub fn add(&mut self, cpu_id: CpuId);

    /// Removes a CPU from the set.
    #[inline]
    pub fn remove(&mut self, cpu_id: CpuId);

    /// Returns true if the set contains the specified CPU.
    #[inline]
    pub fn contains(&self, cpu_id: CpuId) -> bool;

    /// Returns the number of CPUs in the set.
    #[inline]
    pub fn count(&self) -> usize;

    /// Returns true if the set is empty.
    #[inline]
    pub fn is_empty(&self) -> bool;

    /// Returns true if the set is full.
    #[inline]
    pub fn is_full(&self) -> bool;

    /// Adds all CPUs to the set.
    #[inline]
    pub fn add_all(&mut self);

    /// Removes all CPUs from the set.
    #[inline]
    pub fn clear(&mut self);

    /// Iterates over the CPUs in the set.
    ///
    /// The order of the iteration is guaranteed to be in ascending order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = CpuId> + '_;
}

impl From<CpuId> for CpuSet {
    fn from(cpu_id: CpuId) -> Self {
        let mut set = Self::new_empty();
        set.add(cpu_id);
        set
    }
}

/// A subset of all CPUs in the system with atomic operations.
///
/// It provides atomic operations for each CPU in the system. When the
/// operation contains multiple CPUs, the ordering is not guaranteed.
#[derive(Debug)]
pub struct AtomicCpuSet(AtomicIdSet<CpuId>);

impl AtomicCpuSet {
    /// Creates a new `AtomicCpuSet` with an initial value.
    #[inline]
    pub fn new(value: CpuSet) -> Self {
        let CpuSet(id_set) = value;
        Self(AtomicIdSet::new(id_set))
    }

    /// Loads the value of the set with the given ordering.
    ///
    /// This operation is not atomic. When racing with a [`Self::store`]
    /// operation, this load may return a set that contains a portion of the
    /// new value and a portion of the old value. Load on each specific
    /// word is atomic, and follows the specified ordering.
    ///
    /// Note that load with [`Ordering::Release`] is a valid operation, which
    /// is different from the normal atomic operations. When coupled with
    /// [`Ordering::Release`], it actually performs `fetch_or(0, Release)`.
    #[inline]
    pub fn load(&self, ordering: Ordering) -> CpuSet {
        CpuSet(self.0.load(ordering))
    }

    /// Stores a new value to the set with the given ordering.
    ///
    /// This operation is not atomic. When racing with a [`Self::load`]
    /// operation, that load may return a set that contains a portion of the
    /// new value and a portion of the old value. Load on each specific
    /// word is atomic, and follows the specified ordering.
    #[inline]
    pub fn store(&self, value: &CpuSet, ordering: Ordering) {
        self.0.store(&value.0, ordering)
    }
}

#[inherit_methods(from = "self.0")]
impl AtomicCpuSet {
    /// Atomically adds a CPU with the given ordering.
    #[inline]
    pub fn add(&self, cpu_id: CpuId, ordering: Ordering);

    /// Atomically removes a CPU with the given ordering.
    #[inline]
    pub fn remove(&self, cpu_id: CpuId, ordering: Ordering);

    /// Atomically checks if the set contains the specified CPU.
    #[inline]
    pub fn contains(&self, cpu_id: CpuId, ordering: Ordering) -> bool;
}
