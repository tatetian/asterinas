// SPDX-License-Identifier: MPL-2.0

#![no_std]
#![deny(unsafe_code)]

//! Compile-time regression cases for GitHub issue #4252.

use ostd::mm::{
    frame::linked_list::LinkedList,
    heap::{Slab, SlabMeta},
};

// Always compiled first so dependency or toolchain failures cannot masquerade
// as a successful compile-fail regression.
pub fn safe_allocator() {
    let mut slab = Slab::<64>::new().unwrap();
    let slot = slab.meta_mut().project().alloc().unwrap();
    slab.dealloc(slot).unwrap();
    let mut slabs = LinkedList::<SlabMeta<64>>::new();
    slabs.push_front(slab);
    let mut cursor = slabs.cursor_front_mut();
    let mut meta = cursor.current_meta().unwrap();
    let slot = meta.as_mut().alloc().unwrap();
    assert_eq!(meta.nr_allocated(), 1);
    let mut slab = cursor.take_current().unwrap();
    slab.dealloc(slot).unwrap();
}

#[cfg(feature = "direct-swap")]
pub fn direct_swap(a: &mut Slab<64>, b: &mut Slab<64>) {
    core::mem::swap(&mut **a.meta_mut(), &mut **b.meta_mut());
}

#[cfg(feature = "whole-link-swap")]
pub fn whole_link_swap(a: &mut Slab<64>, b: &mut Slab<64>) {
    core::mem::swap(a.meta_mut().get_mut(), b.meta_mut().get_mut());
}

#[cfg(feature = "cursor-swap")]
pub fn cursor_swap(a: &mut LinkedList<SlabMeta<64>>, b: &mut LinkedList<SlabMeta<64>>) {
    let mut a = a.cursor_front_mut();
    let mut b = b.cursor_front_mut();
    core::mem::swap(
        &mut *a.current_meta().unwrap(),
        &mut *b.current_meta().unwrap(),
    );
}

#[cfg(feature = "dynamic-downcast")]
pub fn dynamic_downcast(slab: &mut Slab<64>) {
    let mut pinned = slab.dyn_meta_mut();
    let any = &mut *pinned as &mut dyn core::any::Any;
    let _ = any.downcast_mut::<ostd::mm::frame::linked_list::Link<SlabMeta<64>>>();
}

#[cfg(feature = "unpinned-access")]
pub fn unpinned_access(slab: &mut Slab<64>) -> &mut SlabMeta<64> {
    slab.meta_mut().project().get_mut()
}

#[cfg(feature = "replacement")]
pub fn replacement(slab: &mut Slab<64>) {
    slab.meta_mut().project().set(Default::default());
}

#[cfg(feature = "premature-teardown")]
pub fn premature_teardown(
    slab: &mut Slab<64>,
    reader: &mut ostd::mm::VmReader<ostd::mm::Infallible>,
) {
    slab.dyn_meta_mut().on_drop(reader);
}
