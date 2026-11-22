---
date: 2026-11-22
mode: files
files: ostd/src/mm/frame/unique.rs,ostd/src/mm/frame/meta.rs,ostd/src/mm/frame/linked_list.rs,ostd/src/mm/frame/untyped.rs,ostd/src/mm/frame/test.rs:24-171,ostd/src/mm/heap/slab.rs,ostd/src/mm/heap/slab/tests.rs,ostd/src/mm/page_table/node/mod.rs:291-340,osdk/deps/heap-allocator/src/slab_cache.rs,pin-frame-metadata/proposal.md,pin-frame-metadata/api-tests/src/lib.rs,pin-frame-metadata/check-api.py
head: 6da091ce4-dirty
branch: pin-frame-metadata-prototype
---

# Summary

The prototype keeps frame ownership and generic intrusive lists intact while restricting metadata mutation to pinned references. Four persona passes reviewed the working files and the proposal. This is a files-mode review, so findings include pre-existing code as well as the prototype; line numbers and quoted snippets identify the reviewed snapshot.

The directly relevant security finding was the publicly callable destruction callback. The final prototype makes `AnyFrameMeta::on_drop` unsafe, documents exclusive teardown, reader identity, and single invocation, updates all implementations, and adds a compile-fail regression. The test-cleanup finding is resolved by moving the three deliberately failed teardowns into separate QEMU invocations. The new documentation sentence breaks, incorrect typed accessor link, module metadata requirements, and slab-constructor description have also been corrected.

Three pre-existing issues remain outside the pinning change: arbitrary slab sizes can produce misaligned or out-of-bounds slots; shared-to-unique conversion lacks acquire synchronization; and shared publication during slab construction can race with lookup and cause conversion to panic. All three are present at base commit `6da091ce4`; they require separate fixes. The first two are memory-safety issues, so passing the prototype's tests does not establish soundness of the entire frame/heap subsystem. Other retained style findings concern unchanged code.

Verification confirmed the slot arithmetic directly (a 4095-byte stride places the second pointer write beyond a 4096-byte page) and the callback/ownership paths against the source. The missing synchronization follows from [Rust's documented Release/Acquire ordering](https://doc.rust-lang.org/core/sync/atomic/enum.Ordering.html): a relaxed successful exchange does not acquire prior owners' writes. The replacement distinction follows from [`Pin::set` running the ordinary destructor](https://doc.rust-lang.org/core/pin/struct.Pin.html#method.set), independently of the frame callback. No findings were removed as false positives.

The two slot-size findings share one follow-up: require pointer-aligned slot sizes and initialize only complete slots, or require sizes that divide the page. Their comments below describe separate observable symptoms of that same constructor defect.

## Maintainability

### `osdk/deps/heap-allocator/src/slab_cache.rs` line 93

> ```diff
>         let mut extracted_slab = None;
>
>         if self.partial.contains(which) {
>             extracted_slab = self.partial.cursor_mut_at(which).unwrap().take_current();
>         } else if self.full.contains(which) {
>             extracted_slab = self.full.cursor_mut_at(which).unwrap().take_current();
>         }
> ```

`block-expressions` (nit): `extracted_slab` is initialized to `None` and subsequently reassigned solely to produce the value used to initialize `slab`. This introduces mutable temporary state for a straightforward selection.

**Fix.** Produce the option with an `if`/`else if`/`else` expression and apply `ok_or_else` to that result.

### `ostd/src/mm/frame/linked_list.rs` line 114

> ```diff
>     /// Gets the number of frames in the linked list.
>     pub fn size(&self) -> usize {
>         self.size
>     }
> ```

`least-surprise` (minor): `LinkedList::size()` returns an element count, whereas Rust collections conventionally expose that operation as `len()`. The surrounding frame APIs also use `size()` for byte sizes, making the competing meaning harder to recognize at call sites.

**Fix.** Rename this accessor to `len()` and update its callers.

### `ostd/src/mm/frame/linked_list.rs` line 242

> ```diff
>     /// move it to the first element of the [`LinkedList`]. If it is pointing
>     /// to the last element of the LinkedList then this will move it to the
> ```

`backtick-identifiers` (nit): `LinkedList` appears as unformatted prose in the cursor movement documentation, despite being linked in the preceding sentence. The same omission occurs in `move_prev`.

**Fix.** Use a rustdoc link for both remaining occurrences of `LinkedList`.

### `ostd/src/mm/frame/meta.rs` line 61

> ```diff
>         paddr_to_vaddr, page_size,
>         page_table::boot_pt,
>     },
>     panic::abort,
>     util::ops::range_difference,
> ```

`qualified-fn-imports` (minor): `paddr_to_vaddr`, `page_size`, `abort`, and `range_difference` are imported directly and called without their parent modules. This obscures which operations belong to memory management, panic handling, and range utilities, contrary to the required import convention.

**Fix.** Import the parent modules and use qualified calls such as `mm::paddr_to_vaddr`, `mm::page_size`, `panic::abort`, and `ops::range_difference`.

### `ostd/src/mm/frame/meta.rs` line 169

> ```diff
>     /// Whether the metadata's associated frame is untyped.
> ```

`rfc1574-summary` (nit): `AnyFrameMeta::is_untyped` begins with a noun fragment rather than the third-person verb required for method summaries.

**Fix.** Use `Returns whether the metadata's associated frame is untyped.`

### `ostd/src/mm/frame/meta.rs` line 252

> ```diff
>     pub(super) fn get_from_unused<M: AnyFrameMeta>(
>         paddr: Paddr,
>         metadata: M,
>         as_unique_ptr: bool,
>     ) -> Result<*const Self, GetFrameError> {
> ```

`no-bool-args` (minor): `as_unique_ptr` selects different ownership states and publication behavior. Calls such as `MetaSlot::get_from_unused(paddr, metadata, true)` hide that significant choice behind a boolean literal.

**Fix.** Replace the boolean with an ownership enum whose variants explicitly select shared or unique ownership, or expose separate constructors.

### `ostd/src/mm/frame/meta.rs` line 605

> ```diff
> macro_rules! mark_ranges {
>     ($region: expr, $typ: expr) => {{
>         debug_assert!($region.base().is_multiple_of(PAGE_SIZE));
>         debug_assert!($region.len().is_multiple_of(PAGE_SIZE));
>
>         let start = $region.base().max(super::min_paddr());
>         let seg = Segment::from_unused(start..$region.end(), |_| $typ).unwrap();
>         let _ = ManuallyDrop::new(seg);
>     }};
> }
> ```

`macros-as-last-resort` (minor): `mark_ranges!` performs ordinary runtime work and substitutes only a memory region and a metadata constructor. It needs neither generated declarations nor special syntax, so the macro adds an unnecessary expansion layer to the initialization flow.

**Fix.** Use a generic `mark_range` function taking the region and a metadata factory, then pass closures such as `|| UnusableMemoryMeta` from the match arms.

### `ostd/src/mm/heap/slab.rs` line 89

> ```diff
>         let Some(allocated) = this.free_list.pop() else {
>             crate::error!("Allocating a slot from a full slab");
>             return Err(AllocError);
>         };
> ```

`error-message-format` (nit): The allocation error starts with uppercase prose and describes an attempted action rather than stating the failure. The deallocation error in this module follows the same pattern.

**Fix.** Use specific lowercase messages, such as `"the slab is full"` and `"the slot does not belong to this slab"`.

## Correctness

### `osdk/deps/heap-allocator/src/slab_cache.rs` line 66

> ```diff
>         let Some(mut allocated_empty) = Self::alloc_slab() else {
>             log::error!("Failed to allocate a new slab");
>             return Err(AllocError);
>         };
> ```

`ostd-log-only` (minor): The allocation failure and both deallocation error paths use `log::error!` directly in a first-party OSTD-based crate, bypassing the required OSTD logging macros.

**Fix.** Replace all three calls with `ostd::error!`, using the crate's existing `__log_prefix` definition.

### `ostd/src/mm/heap/slab.rs` line 117

> ```diff
>             }))?
>             .try_into()
>             .unwrap();
> ```

`propagate-errors` (major): `alloc_frame_with()` publishes a shared frame before this conversion. If another CPU calls `Frame::<dyn AnyFrameMeta>::from_in_use(paddr)` after publication and retains the returned handle, the reference count becomes `2`, so `try_into()` returns `Err` and this `unwrap()` panics. The physical address need not have been exposed by this constructor: another caller can retain its address from an earlier allocation.

**Fix.** Allocate the slab as a `UniqueFrame` directly, keeping its metadata inaccessible to shared lookup throughout initialization. If shared allocation remains necessary, propagate conversion failure through the constructor's `Result` instead of panicking.

### `ostd/src/mm/heap/slab.rs` line 127

> ```diff
> for slot_offset in (0..PAGE_SIZE).step_by(SLOT_SIZE) {
>     let slot_ptr = unsafe { NonNull::new_unchecked((head_vaddr + slot_offset) as *mut u8) };
> ```

Off by one (major): `Slab::<24>::new()` passes the constructor checks, but with `PAGE_SIZE == 4096`, this loop creates `171` slots while `capacity()` reports `170`. Consequently, `alloc()` succeeds after the advertised capacity is exhausted, and the first returned slot starts at offset `4080`, where only `16` bytes remain.

**Fix.** Shared with the out-of-bounds slot initialization finding in Security: enforce pointer alignment and initialize only complete slots, or require sizes that divide the page. Add coverage for capacity and slot extent.

### `ostd/src/mm/heap/slab/tests.rs` line 106

> ```diff
>         // The failing teardown deliberately retains ownership of the page.
>         // These three frames cannot be recovered through the public API.
> ```

`test-cleanup` (minor): Every successful execution permanently strands `3` physical frames. Catching the expected panic preserves the pages, but the test then discards the remaining slot handles without reclaiming them. The test runner continues executing other tests in the same kernel, so this changes their available memory.

**Fix.** Run these destructive cases in isolated test-kernel invocations, or provide a narrowly scoped test-only recovery helper that restores ownership after the assertions, returns the outstanding slot, and releases each frame.

## Security

### `ostd/src/mm/frame/meta.rs` line 167

> ```diff
> fn on_drop(self: Pin<&mut Self>, _reader: &mut VmReader<Infallible>) {}
> ```

Unvalidated destruction callback (critical): `on_drop` is publicly callable from safe code with an arbitrary `VmReader`. A caller can retain a populated page-table frame through `Frame::from_in_use`, drop its `VmSpace`, convert the remaining handle to `UniqueFrame<dyn AnyFrameMeta>`, and invoke `dyn_meta_mut().on_drop()` with an aligned buffer containing fabricated PTEs. `PageTablePageMeta::on_drop` treats those entries as owned references and calls `PageTableNode::from_raw`, allowing it to release a victim frame whose real owner remains alive. Pinning the receiver does not validate the reader or authorize destruction.

**Fix.** Make `AnyFrameMeta::on_drop` unsafe and document that the reader must cover the associated frame and that the callback may run only once during teardown. Update its implementations and keep invocation inside the validated teardown path.

### `ostd/src/mm/frame/unique.rs` line 197

> ```diff
> match frame.slot().ref_count.compare_exchange(
>     1,
>     REF_COUNT_UNIQUE,
>     Ordering::Relaxed,
>     Ordering::Relaxed,
> ) {
> ```

Missing synchronization (critical): The successful `Ordering::Relaxed` exchange does not acquire preceding owners' accesses. For example, one thread updates metadata containing a `SpinLock<usize>` and drops its shared frame, decrementing the reference count from `2` to `1` with `Ordering::Release`. Another thread can successfully convert its remaining handle and access the value through `meta_mut().get_mut()` and `SpinLock::get_mut()`. There is no acquire operation establishing a happens-before relationship, so these non-atomic accesses constitute a data race. Immediate destruction through `UniqueFrame` has the same missing synchronization.

**Fix.** Use `Ordering::Acquire` for the successful exchange, pairing with shared-frame release decrements before exposing exclusive access or destroying metadata. The failure ordering can remain `Ordering::Relaxed`.

### `ostd/src/mm/heap/slab.rs` line 127

> ```diff
> for slot_offset in (0..PAGE_SIZE).step_by(SLOT_SIZE) {
>     // SAFETY: The slot is within the slab so it can't be NULL.
>     let slot_ptr = unsafe { NonNull::new_unchecked((head_vaddr + slot_offset) as *mut u8) };
>     // SAFETY: The slot is newly allocated in the slab.
>     meta.free_list
>         .push(unsafe { HeapSlot::new(slot_ptr, super::SlotInfo::SlabSlot(SLOT_SIZE)) });
> }
> ```

Out of bounds write (critical): Safe code can call `Slab::<4095>::new()` with `PAGE_SIZE == 4096`. The loop creates slots at offsets `0` and `4095`; inserting the latter into `SlabSlotList` writes a `usize` at an unaligned address and seven bytes beyond the owned page. The existing assertions accept this size, allowing safe code to corrupt adjacent memory.

**Fix.** Shared with the slot-count finding in Correctness: enforce pointer alignment and initialize only complete slots, or require sizes that divide the page. Add coverage for rejected sizes and slot extent.

## Documentation

### `ostd/src/mm/frame/linked_list.rs` line 28

> ```diff
> /// Two key features that [`LinkedList`] is different from
> /// [`alloc::collections::LinkedList`] is that:
> ```

Inconsistent feature count (nit): The introduction announces two distinguishing features, but the numbered list contains three.

**Fix.** Introduce the list as “The following features distinguish this type from `alloc::collections::LinkedList`:” without specifying a count.

### `ostd/src/mm/frame/meta.rs` line 9

> ```diff
> //! There would be a dedicated small
> //! "heap" space in each slot for dynamic metadata. You can store anything as
> //! the metadata of a frame as long as it's [`Sync`].
> ```

Incomplete metadata requirements (minor): The module documentation says that implementing `Sync` is sufficient for frame metadata. A type that implements `Sync` but not `Send` cannot implement `AnyFrameMeta`, and metadata must also satisfy the documented size and alignment limits.

**Fix.** Describe supported metadata in terms of `AnyFrameMeta` and link to its requirements instead of presenting `Sync` as the sole condition.

### `ostd/src/mm/frame/meta.rs` line 141

> ```diff
> /// Metadata is pinned in its slot after installation. Address-sensitive metadata
> /// must be `!Unpin` and establish its address dependency only after installation.
> /// Mutable access through frame handles and list cursors preserves this pinning.
> ///
> /// [`Pin::set`] can replace metadata in place if a caller can construct another
> /// value of the same type. It runs [`Drop`], but not [`Self::on_drop`]. Types must
> /// either make such replacement safe or prevent constructing replacement values.
> ```

`semantic-line-breaks` (nit): Several documentation lines combine the end of one sentence with the beginning of another, including the new pinning and replacement contracts. This violates the requirement to break prose at sentence boundaries.

**Fix.** Insert sentence-boundary line breaks after “installation”, “type”, and the sentence ending with `Self::on_drop`. Apply the same correction to the similar sentence joins elsewhere in the reviewed doc comments.

### `ostd/src/mm/frame/unique.rs` line 77

> ```diff
> /// Gets the dynamically-typed metadata of this frame.
> ///
> /// If the type is known at compile time, use [`Frame::meta`] instead.
> ```

Incorrect accessor link (minor): The documentation directs a caller holding a `UniqueFrame<M>` to `Frame::meta`, which belongs to a different handle type. The corresponding typed accessor here is `UniqueFrame::meta`.

**Fix.** Replace the link with `Self::meta`.

### `ostd/src/mm/heap/slab.rs` line 100

> ```diff
> /// Allocates a new slab of the given size.
> ///
> /// If the size is less than `SLOT_SIZE` or [`PAGE_SIZE`], the size will be
> /// the maximum of the two.
> pub fn new() -> crate::prelude::Result<Self> {
> ```

Incorrect allocation documentation (minor): The constructor documentation describes rounding a supplied size up, but `Slab::new` takes no size argument and allocates one page. Invalid values of `SLOT_SIZE` fail compile-time assertions instead of being adjusted.

**Fix.** Document that `Slab::new` allocates one `PAGE_SIZE` slab with slots of `SLOT_SIZE` bytes, and state the compile-time constraints enforced by the constructor.
