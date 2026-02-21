# Comments and Documentation (D)

API documentation describes the meanings and usage of APIs
and will be rendered into web pages by rustdoc.
It is necessary to add documentation to all public APIs,
including crates, modules, structs, traits, functions, macros, and more.
The use of the `#[warn(missing_docs)]` lint enforces this rule.

Asterinas adheres to the API style guidelines of the Rust community.
The recommended documentation style is specified by two official resources:
1. The rustdoc book:
   [How to write documentation](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html);
2. The Rust RFC book:
   [API Documentation Conventions](https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.html#appendix-a-full-conventions-text).

### D1. Use `///` doc comments with semantic line breaks

Use `///` doc comments
so that each clause occupies its own line in the source.

```rust
/// Creates a new virtual memory mapping
/// at the specified address range.
///
/// The caller must ensure that the range
/// does not overlap with existing mappings.
```

### D2. Follow RFC 505 summary line conventions

The first line of a doc comment is
third-person singular present indicative
("Returns", "Creates", "Acquires"), concise, one sentence.

```rust
/// Returns the mapping's start address.
pub fn map_to_addr(&self) -> Vaddr {
    self.map_to_addr
}
```

### D3. Wrap identifiers in backticks

Type names, method names,
and code identifiers in doc comments
must be wrapped in backticks for rustdoc rendering.
Make type names into rustdoc hyperlinks
using square-bracket syntax (`[TypeName]`).

### D4. Library crate documentation

Library crates should re-export the crate-level README
as the top-level rustdoc page:

```rust
#![doc = include_str!("../README.md")]
```

### D5. Do not disclose implementation details in doc comments

Doc comments describe _what_ and _how to use_,
not _how it works internally_.

### D6. `TODO`/`FIXME` in `//` comments, not `///` doc comments

Internal notes like `TODO` and `FIXME`
are not user-facing documentation
and must use regular `//` comments, not `///` doc comments.

```rust
// TODO: Address the issue of negative dentry bloating.
// See https://lwn.net/Articles/894098/ for more details.
```

### D7. `# Safety` sections use `-` bullet points

The project convention
is to use `-` bullets in `# Safety` sections,
not `*` asterisks.

### D8. Comprehensive doc coverage for all public APIs

All public items —
especially syscall handlers, public structs/enums,
and macro-generated methods —
must have doc comments.

### D9. Add module-level documentation for major components

Every module file that serves as
a significant kernel component
(a subsystem entry point, a major data structure,
a driver)
should begin with a `//!` comment explaining:
(1) what this module does,
(2) the key types it exposes, and
(3) how it relates to neighboring modules.

```rust
//! Virtual memory area (VMA) management.
//!
//! This module defines [`VmMapping`] and its associated types,
//! which represent contiguous regions of a process's
//! virtual address space.
//! VMAs are managed by the [`Vmar`] tree
//! defined in the parent module.
```

### D10. Delete commented-out code

Never commit commented-out code blocks.
Version control preserves history.
Commented-out code confuses readers, rots,
and creates the false impression
that it is still relevant.

See also
[General Guidelines — C9](../../general-guidelines/themes.md#c9-never-commit-commented-out-code).

### D11. Delete dead code aggressively

Unreachable functions, unused imports,
obsolete branches, and orphaned types
should be deleted.
Every line of code has a maintenance cost.
The most readable code is code that is not there.

`#[expect(dead_code)]` (per
[MA2](macros-and-attributes.md#ma2-when-to-expectdead_code))
is the narrow exception, not the norm.
