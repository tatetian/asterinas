# Pinned frame metadata prototype validation

The prototype is on branch `pin-frame-metadata-prototype`,
based on `6da091ce4` from `origin/main`.
The [issue proposal](proposal.md) describes the API and safety argument.
Validation used the project container, Rust `nightly-2026-07-21`,
and a locally built OSDK 0.19.0.
The saved logs remove terminal color escapes and replace the checkout path with `$WORKTREE`.

## Results

| Check | Result | Evidence |
|---|---|---|
| Safe allocator example | Compiles with `#![deny(unsafe_code)]` | [Log](evidence/api-safe-allocator.log) |
| Seven forbidden API uses | All rejected with the expected compiler diagnostic | [Summary](api-check.log) |
| OSTD kernel tests | 228 passed, 0 failed; x86-64 QEMU/KVM, 4 CPUs | [Log](ktest-ostd.log) |
| Heap allocator kernel tests | 2 passed, 0 failed; x86-64 QEMU/KVM, 4 CPUs | [Log](ktest-heap-allocator.log) |
| Failed slab drop | Passed in an isolated test kernel | [Log](evidence/teardown-drop.log) |
| Failed slab repurpose | Passed in an isolated test kernel | [Log](evidence/teardown-repurpose.log) |
| Failed slab reset | Passed in an isolated test kernel | [Log](evidence/teardown-reset.log) |
| Full workspace Clippy | Passed with warnings denied | [Log](clippy-workspace.log) |
| OSTD Clippy with kernel tests | Passed with warnings denied | [Log](clippy-tests.log) |
| Full RISC-V kernel check | Passed; compile only | [Log](check-riscv64.log) |

The compile-fail cases cover direct metadata swaps, whole-link swaps,
cursor swaps, mutable type-erased downcasts, unpinned access,
replacement through `Default`, and premature destruction callbacks.
The harness first compiles a positive example,
then checks each negative case independently for its expected diagnostic.

The seven new OSTD unit tests cover ordinary `Unpin` mutation,
an independent address-sensitive metadata type,
unique/shared/type-erased ownership transitions,
intrusive-list transfers, destruction and `Pin::set`,
slot uniqueness and reuse, moving whole slab handles,
and repurposing an empty slab.
The three isolated tests catch the expected teardown panic,
verify that the physical page cannot be reacquired as unused,
and access a detached free-slot list whose backing page must remain alive.

## Reproducing the checks

Build OSDK inside this worktree first.
An OSDK binary built with `OSDK_LOCAL_DEV=1` embeds its source paths;
using one from another checkout can silently test that checkout's OSTD and allocators.

```sh
OSDK_LOCAL_DEV=1 CARGO_TARGET_DIR="$PWD/osdk/target" \
    cargo build --release --manifest-path osdk/Cargo.toml
export CARGO_OSDK="$PWD/osdk/target/release/cargo-osdk"
export CARGO_TARGET_DIR="$PWD/target"
export CARGO_INCREMENTAL=0

python3 pin-frame-metadata/check-api.py
"$CARGO_OSDK" osdk clippy -- -- -D warnings
"$CARGO_OSDK" osdk clippy --ktests -p ostd -- -D warnings
OSDK_TARGET_ARCH=riscv64 "$CARGO_OSDK" osdk check -p asterinas
```

The repository's normal QEMU configuration expects four scratch disks.
These OSTD and allocator tests do not mount filesystems,
so blank sparse images suffice when the worktree has no prepared images.
The following setup preserves existing files:

```sh
python3 - <<'PY'
from pathlib import Path
directory = Path('test/initramfs/build')
directory.mkdir(parents=True, exist_ok=True)
for name in ['ext2.img', 'exfat.img', 'ltp_dev.img', 'nvme0n1.img']:
    path = directory / name
    if not path.exists():
        with path.open('xb') as stream:
            stream.truncate(64 * 1024 * 1024)
PY

export SMP=4 BOOT_METHOD=vmm-direct QEMU_DISPLAY=none CONSOLE=ttyS0
(cd ostd && "$CARGO_OSDK" osdk test \
    --boot-method=vmm-direct --strip-elf --qemu-args='-accel kvm')
(cd osdk/deps/heap-allocator && "$CARGO_OSDK" osdk test \
    --boot-method=vmm-direct --strip-elf --qemu-args='-accel kvm')
```

Run the destructive cases separately using their minimal QEMU configuration,
which requires KVM and no disk images.
Each invocation contains one test and exits after checking the retained page;
the intentionally leaked frame never affects another test process.

```sh
(cd pin-frame-metadata/teardown-tests && "$CARGO_OSDK" osdk test)
(cd pin-frame-metadata/teardown-tests && "$CARGO_OSDK" osdk test --features repurpose)
(cd pin-frame-metadata/teardown-tests && "$CARGO_OSDK" osdk test --features reset)
```

## Review findings and scope

The [four-persona review](review.md) led to an unsafe teardown callback contract,
an additional compile-fail test, isolated teardown tests,
and corrections to the metadata documentation.
The review covers whole files, including unchanged code.

Three pre-existing defects remain outside this prototype:
the slab constructor accepts slot sizes that can create misaligned or partial slots;
the shared-to-unique reference-count transition lacks acquire synchronization;
and slab construction temporarily publishes shared ownership,
allowing another lookup to make the subsequent unique conversion panic.
The first two are memory-safety issues.
The prototype validates the pinning fix for #4252,
not the soundness of the entire frame and heap subsystem.

The runtime checks use valid 64-byte slabs and run on x86-64.
RISC-V was compiled but not booted;
no Miri/KMiri run or performance benchmark was performed.
The no-extra-allocation and no-extra-identity-check properties follow from the implementation:
metadata stays inline, `PhantomPinned` is zero-sized,
and `Pin` changes the access contract rather than adding a runtime check.
