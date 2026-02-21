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
