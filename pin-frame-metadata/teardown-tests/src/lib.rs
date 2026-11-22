// SPDX-License-Identifier: MPL-2.0

#![no_std]
#![deny(unsafe_code)]

//! Run each intentionally failing teardown in its own QEMU instance.

use ostd::{
    mm::{
        Frame, HasPaddr, PAGE_SIZE,
        frame::meta::GetFrameError,
        heap::{Slab, SlabSlotList},
    },
    panic::catch_unwind,
    prelude::ktest,
};

#[ktest]
fn outstanding_slot_keeps_its_page_after_failed_teardown() {
    let mut slab = Slab::<64>::new().unwrap();
    let paddr = slab.paddr();
    let slot = slab.meta_mut().project().alloc().unwrap();
    let mut detached_slots = SlabSlotList::<64>::new();
    detached_slots.push(slot);

    let result = catch_unwind(|| {
        if cfg!(feature = "repurpose") {
            drop(slab.repurpose(()));
        } else if cfg!(feature = "reset") {
            slab.reset_as_unused();
        } else {
            drop(slab);
        }
    });
    assert!(result.is_err());
    assert!(matches!(
        Frame::<()>::from_unused(paddr, ()),
        Err(GetFrameError::Busy | GetFrameError::Unique)
    ));
    assert_eq!(
        detached_slots.pop().unwrap().paddr() / PAGE_SIZE,
        paddr / PAGE_SIZE
    );

    // Failed teardown deliberately retains the frame. Ending this dedicated
    // kernel avoids leaving unrecoverable pages in a shared unit-test process.
}
