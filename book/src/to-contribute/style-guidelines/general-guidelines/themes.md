# Themes

Concrete, actionable advice organized into themes.
Each item applies regardless of programming language.
For Rust-specific advice, see
[Rust Guidelines](../rust-guidelines/README.md).

## Names (N)

### N1. Be descriptive

Choose names that convey meaning at the point of use.
Avoid single-letter names and ambiguous abbreviations.
Prefer full words over cryptic shorthand
so that readers do not need surrounding context
to understand a variable's purpose.
The optimal name length is 10–16 characters on average;
shorter names lack meaning,
longer names obscure visual structure.

### N2. Avoid misleading names

A name must accurately describe what it refers to.
If the semantics of a function or variable change,
rename it to match the new behavior.
Actively ask yourself
"What else could this name mean?"

### N3. Scale name length to scope

Short names are acceptable for tiny scopes
(a three-line loop body).
For variables visible across many lines
or accessible from other modules,
use descriptive, unambiguous names.

### N4. Use domain-accurate terminology

Choose accurate English or ecosystem terminology
rather than inheriting poorly-chosen jargon.
Names should make sense
to someone reading the code for the first time.

### N5. Pick one word per concept

Choose a single word for each abstract concept
and use it consistently across the codebase.
Do not use the same word for two different purposes.

### N6. Encode units and important attributes in names

When the type does not encode the unit,
the name must.
Kernel code deals with bytes, pages, frames,
nanoseconds, ticks, and sectors —
ambiguous units are a source of real bugs.

```
// Good — unit is unambiguous
timeout_ns
offset_bytes
size_pages
delay_ms

// Bad — unit is ambiguous
timeout
offset
size
delay
```

Where Rust's type system can enforce units (newtypes),
prefer that.
Where it cannot, the name must carry the information.

### N7. Use assertion-style boolean names

Boolean variables and functions
should read as assertions of fact.
Use `is_`, `has_`, `can_`, `should_`, `was_`,
or `needs_` prefixes.
Never use double negatives
(`is_not_empty`, `no_error`).
A bare name like `found`, `done`, or `ready`
is acceptable when the context is unambiguous.

### N8. Use precise opposites consistently

When naming paired operations or variables,
use established opposite pairs consistently:
`begin`/`end`, `first`/`last`, `open`/`close`,
`acquire`/`release`, `source`/`destination`,
`push`/`pop`, `next`/`previous`, `old`/`new`.
Do not mix pairs
(e.g., `start`/`end` or `begin`/`finish`).

### N9. Place computed-value qualifiers at the end

Qualifiers like `count`, `total`, `max`, `min`,
`index`, and `size` go at the end of names.
This creates natural alphabetical grouping
of related variables.

```
// Good — qualifiers at the end
page_count
buffer_size
offset_max
frame_index

// Bad — qualifiers at the beginning
count_pages
max_offset
```

## Comments (C)

### C1. Explain why, not how

Comments should explain the intent behind the code,
not restate what the code does.
If a comment merely paraphrases the code,
it adds noise without insight.

### C2. Avoid redundant comments

Comments that state what the code already makes obvious
are worse than no comment.
Fix bad names and confusing structure
instead of papering over them with comments.

### C3. Track TODOs and FIXMEs

Every `TODO` or `FIXME` comment must include
a tracking reference —
either a URL to an issue
or a brief explanation of when the work should be done.

### C4. Use REMINDER comments for cross-file dependencies

Use `// REMINDER:` comments to flag dependencies
where a change in one location
requires a corresponding change elsewhere.

### C5. Follow sentence formatting conventions

End every sentence in documentation with a period.
Use [semantic line breaks](https://sembr.org/)
so that each clause occupies its own line in the source.
Keep documentation comment lines
at or below 80 characters
to remain readable in side-by-side diffs.

### C6. Keep comments up to date

An obsolete or inaccurate comment
is worse than no comment at all.
When you change code, update or delete
the comments that describe it.

### C7. Document design decisions

When the code makes a non-obvious choice —
a particular data structure, a locking strategy,
a deviation from Linux behavior —
add a comment explaining the rationale
and any alternatives considered.
Design-decision comments ("director's commentary")
are the most valuable kind of comment.

```
// We use a radix tree rather than a HashMap
// because lookups must be O(log n) worst-case
// for the page fault handler.
// A HashMap gives O(1) amortized
// but O(n) worst-case due to rehashing,
// which is unacceptable on the page fault path.
```

### C8. Cite specifications and algorithm sources

When implementing behavior defined by
an external specification or a non-trivial algorithm,
cite the source:
the relevant POSIX section, Linux man page,
hardware reference manual, or academic paper.

```
// Per POSIX.1-2017 Section 2.9.7,
// write() on a pipe is atomic when nbyte <= PIPE_BUF.

// Implements the buddy allocation algorithm
// as described in Knuth TAOCP Vol. 1, Section 2.5.
```

### C9. Never commit commented-out code

Commented-out code confuses readers,
rots over time,
and creates the false impression
that it is still relevant.
Version control preserves history —
if the code might be needed later,
it lives in git history, not in the source file.

## Layout (L)

### L1. One concept per file

When a file grows long or contains multiple distinct concepts,
split it.
Each major data structure, each subsystem entry point,
each significant abstraction
deserves its own file.

### L2. Primary type first

Within a file, place the primary type or trait at the top.
Supporting enums, configuration types,
and helper functions follow.

### L3. Group related things together

Keep related declarations vertically close.
Separate groups with blank lines.
Do not insert unnecessary blank lines
within a tightly related block of code.

### L4. Keep sorted

Maintain sorted order in dependency lists,
configuration manifests, and similar declarations.
Sorted lists are easier to scan
and produce smaller diffs.

### L5. Organize code for top-down reading

Within functions,
group related statements into logical "paragraphs"
separated by blank lines.
For long functions,
add a one-line summary comment
at the start of each paragraph.

Within `impl` blocks,
place public methods before private helpers.
Within each visibility group,
order methods so that callers appear before callees
where possible,
enabling the file to be read top to bottom.

### L6. Do not interleave unrelated tasks

Even within a single function,
do not interleave logically separate tasks.
Complete one task before starting the next.
If a function performs input validation,
resource lookup, core logic, and result formatting,
these should appear as distinct sequential blocks,
not interwoven statements.

## Formatting (F)

### F1. Format error messages consistently

- Start with a lowercase letter
  (unless the first word is a proper noun or identifier).
- Wrap identifiers in backticks:
  `` `new_root` is not a directory ``.
- Be specific:
  prefer "empty file system type"
  over "no fs type specified".
- Follow the style of Linux man page ERRORS sections
  when describing errno conditions.

### F2. Break long lines

Break lines that are too long.
When a function call or definition spans multiple lines,
align the continuation to improve readability.

### F3. Use Bootlin for Linux references

When referencing Linux kernel source,
prefer [Bootlin Elixir](https://elixir.bootlin.com/linux/latest/source) links
over raw GitHub links.
Bootlin links are stable across kernel versions
and support cross-referencing.

## API Design (A)

### A1. Hide implementation details

Do not expose internal implementation details
through public APIs.
A module's public surface
should contain only what its consumers need.
This is the most frequently cited principle in reviews.

Avoid transitive navigation (Law of Demeter):
`process.thread_group().session().terminal().write()`
couples the caller to four levels of internal structure.
Provide a direct method on the immediate object
or pass only the specific object needed as a parameter.

See also
[Modules and Crates — MC1–MC3](../rust-guidelines/language-items/modules-and-crates.md#mc1-default-to-the-narrowest-visibility)
for Rust-specific visibility rules.

### A2. Set correct values upfront

Initialize fields to their correct values at construction time
rather than relying on lazy initialization
or post-construction fixup.

### A3. Use types to encode invariants

Leverage the type system
to make illegal states unrepresentable.
Prefer newtypes and enums
over bare integers and boolean flags.

### A4. Prefer existing abstractions

Before adding a new abstraction,
check whether an existing one already serves the purpose.
Unnecessary abstractions add cognitive overhead.

### A5. Provide evidence for non-trivial decisions

When a design choice is not obvious,
include a comment or commit message
explaining the reasoning
and any alternatives considered.

### A6. Reference Linux but justify deviations

Asterinas aims for Linux compatibility.
When the implementation differs from Linux behavior,
document the reason.

### A7. Fix root causes, not symptoms

When a bug or invariant violation is identified,
fix the root cause
rather than adding defensive checks
that paper over the symptom.
Unnecessary guards hide real bugs.

See also
[Error Handling — E6](../rust-guidelines/select-topics/error-handling.md#e6-fix-root-causes-not-symptoms)
for the Rust-specific formulation.

### A8. Validate at boundaries, trust internally

Designate certain interfaces as validation boundaries.
In Asterinas, syscall entry points
are the primary barricade:
all user-supplied data
(pointers, file descriptors, sizes, flags, strings)
must be validated at the syscall boundary.
Once validated, internal kernel functions
may trust these values without re-validation.

Document this contract:
functions that accept already-validated data
should note it in their doc comments;
functions that accept raw user data
must validate before use.

### A9. Provide services in opposites

If an API provides `lock()`, provide `unlock()`
(or, better, an RAII guard).
If it provides `open()`, provide `close()`.
An API that offers only one side
of a paired operation
is incomplete and error-prone.
