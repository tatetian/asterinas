# General Guidelines

Concrete, actionable advice organized into themes.
Each item applies regardless of programming language.
For the underlying philosophy, see
[How These Guidelines Are Written](../how-these-guidelines-are-written.md).
For Rust-specific advice, see
[Rust Guidelines](../rust-guidelines/README.md).

## Names

### Be descriptive (`descriptive-names`) {#descriptive-names}

Choose names that convey meaning at the point of use.
Avoid single-letter names and ambiguous abbreviations.
Prefer full words over cryptic shorthand
so that readers do not need surrounding context
to understand a variable's purpose.
The optimal name length is 10–16 characters on average;
shorter names lack meaning,
longer names obscure visual structure.

### Avoid misleading names (`no-misleading-names`) {#no-misleading-names}

A name must accurately describe what it refers to.
If the semantics of a function or variable change,
rename it to match the new behavior.
Actively ask yourself
"What else could this name mean?"

### Encode units and important attributes in names (`encode-units`) {#encode-units}

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

See also:
PR [#2796](https://github.com/asterinas/asterinas/pull/2796#discussion_r2646889913).

### Use assertion-style boolean names (`bool-names`) {#bool-names}

Boolean variables and functions
should read as assertions of fact.
Use `is_`, `has_`, `can_`, `should_`, `was_`,
or `needs_` prefixes.
Never use double negatives
(`is_not_empty`, `no_error`).
A bare name like `found`, `done`, or `ready`
is acceptable when the context is unambiguous.

See also:
PR [#1488](https://github.com/asterinas/asterinas/pull/1488#discussion_r1841827039).

## Comments

### Explain why, not how (`explain-why`) {#explain-why}

Comments should explain the intent behind the code,
not restate what the code does.
If a comment merely paraphrases the code,
it adds noise without insight.

### Document design decisions (`design-decisions`) {#design-decisions}

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

See also:
PR [#2265](https://github.com/asterinas/asterinas/pull/2265#discussion_r2266220943)
and [#2050](https://github.com/asterinas/asterinas/pull/2050#discussion_r2224106025).

### Cite specifications and algorithm sources (`cite-sources`) {#cite-sources}

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

### Never commit commented-out code (`no-commented-code`) {#no-commented-code}

Commented-out code confuses readers,
rots over time,
and creates the false impression
that it is still relevant.
Version control preserves history —
if the code might be needed later,
it lives in git history, not in the source file.

See also:
PR [#1880](https://github.com/asterinas/asterinas/pull/1880#discussion_r1982942684).

## Layout

### One concept per file (`one-concept-per-file`) {#one-concept-per-file}

When a file grows long or contains multiple distinct concepts,
split it.
Each major data structure, each subsystem entry point,
each significant abstraction
deserves its own file.

### Organize code for top-down reading (`top-down-reading`) {#top-down-reading}

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

## Formatting

### Format error messages consistently (`error-message-format`) {#error-message-format}

- Start with a lowercase letter
  (unless the first word is a proper noun or identifier).
- Wrap identifiers in backticks:
  `` `new_root` is not a directory ``.
- Be specific:
  prefer "empty file system type"
  over "no fs type specified".
- Follow the style of Linux man page ERRORS sections
  when describing errno conditions.

## API Design

### Hide implementation details (`hide-impl-details`) {#hide-impl-details}

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
[Modules and Crates](../rust-guidelines/language-items/modules-and-crates.md#narrow-visibility)
for Rust-specific visibility rules.
PR [#2951](https://github.com/asterinas/asterinas/pull/2951#discussion_r2766432889).

### Use types to encode invariants (`type-invariants`) {#type-invariants}

Leverage the type system
to make illegal states unrepresentable.
Prefer newtypes and enums
over bare integers and boolean flags.

See also:
PR [#2265](https://github.com/asterinas/asterinas/pull/2265#discussion_r2266214191)
and [#2514](https://github.com/asterinas/asterinas/pull/2514).

### Fix root causes, not symptoms (`fix-root-causes`) {#fix-root-causes}

When a bug or invariant violation is identified,
fix the root cause
rather than adding defensive checks
that paper over the symptom.
Unnecessary guards hide real bugs.

See also
[Error Handling](../rust-guidelines/select-topics/error-handling.md#fix-root-causes-not-symptoms)
for the Rust-specific formulation.
PR [#2741](https://github.com/asterinas/asterinas/pull/2741).

### Validate at boundaries, trust internally (`validate-at-boundaries`) {#validate-at-boundaries}

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

See also:
PR [#2806](https://github.com/asterinas/asterinas/pull/2806).
