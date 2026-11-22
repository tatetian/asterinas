// SPDX-License-Identifier: MPL-2.0

//! Regression tests for pinned slab metadata (GitHub issue #4252).

use alloc::vec::Vec;

use super::*;
use crate::{mm::frame::linked_list::LinkedList, prelude::ktest};

#[ktest]
fn slab_slots_remain_unique_and_reusable() {
    let mut slab = Slab::<64>::new().unwrap();
    let mut slots = Vec::new();
    for _ in 0..slab.meta().capacity() {
        let slot = slab.meta_mut().project().alloc().unwrap();
        assert!((slab.paddr()..slab.paddr() + PAGE_SIZE).contains(&slot.paddr()));
        assert!(
            slots
                .iter()
                .all(|other: &HeapSlot| other.paddr() != slot.paddr())
        );
        slots.push(slot);
    }
    assert!(slab.meta_mut().project().alloc().is_err());
    assert_eq!(slab.meta().nr_allocated(), slab.meta().capacity());
    for slot in slots {
        slab.dealloc(slot).unwrap();
    }
    assert_eq!(slab.meta().nr_allocated(), 0);
    let slot = slab.meta_mut().project().alloc().unwrap();
    slab.dealloc(slot).unwrap();
}

#[ktest]
fn swapping_slab_handles_keeps_metadata_with_its_page() {
    let mut first = Slab::<64>::new().unwrap();
    let mut second = Slab::<64>::new().unwrap();
    let first_paddr = first.paddr();
    let slot = first.meta_mut().project().alloc().unwrap();
    core::mem::swap(&mut first, &mut second);
    assert_eq!(second.paddr(), first_paddr);
    assert_eq!(second.meta().nr_allocated(), 1);
    assert_eq!(first.meta().nr_allocated(), 0);
    drop(first);
    second.dealloc(slot).unwrap();
}

#[ktest]
fn slab_allocation_survives_list_transfers() {
    let slab = Slab::<64>::new().unwrap();
    let paddr = slab.paddr();
    let mut first = LinkedList::new();
    let mut second = LinkedList::new();
    first.push_back(slab);
    let mut cursor = first.cursor_mut_at(paddr).unwrap();
    let mut meta = cursor.current_meta().unwrap();
    let slot = meta.as_mut().alloc().unwrap();
    assert_eq!(meta.nr_allocated(), 1);
    second.push_front(cursor.take_current().unwrap());
    assert!(first.is_empty());
    let mut slab = second.pop_front().unwrap();
    assert_eq!(slab.paddr(), paddr);
    slab.dealloc(slot).unwrap();
}

#[ktest]
fn empty_slab_can_be_repurposed() {
    let mut slab = Slab::<64>::new().unwrap();
    let slot = slab.meta_mut().project().alloc().unwrap();
    slab.dealloc(slot).unwrap();
    let paddr = slab.paddr();
    let frame = slab.repurpose(());
    assert_eq!(frame.paddr(), paddr);
}
