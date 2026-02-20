# General Guidelines

This page covers language-agnostic conventions
that apply to all code in the Asterinas project.
For Rust-specific conventions, see
[Rust Guidelines](rust-guidelines.md).

## Naming

### Be Descriptive

Choose names that convey meaning at the point of use.
Avoid single-letter names and ambiguous abbreviations.
Prefer full words over cryptic shorthand
so that readers do not need surrounding context
to understand a variable's purpose.

### Avoid Misleading Names

A name must accurately describe what it refers to.
If the semantics of a function or variable change,
rename it to match the new behavior.

## Documentation

### Sentence Formatting

End every sentence in documentation with a period.
Use [semantic line breaks](https://sembr.org/)
so that each clause occupies its own line in the source.

### Line Length

Keep documentation comment lines at or below 80 characters
to remain readable in side-by-side diffs.

### Source References

When referencing Linux kernel source,
prefer [Bootlin Elixir](https://elixir.bootlin.com/linux/latest/source) links
over raw GitHub links.
Bootlin links are stable across kernel versions
and support cross-referencing.

### TODO and FIXME

Every `TODO` or `FIXME` comment must include
a tracking reference — either a URL to an issue
or a brief explanation of when the work should be done.

### REMINDER Comments

Use `// REMINDER:` comments to flag cross-file dependencies
where a change in one location
requires a corresponding change elsewhere.

## Code Organization

### Keep Sorted

Maintain sorted order in dependency lists,
configuration manifests, and variable declarations.
Sorted lists are easier to scan and produce smaller diffs.

### Extract Common Patterns

When the same pattern appears three or more times,
extract it into a reusable helper function.
Duplication harms readability and maintainability.

### Hide Implementation Details

Do not expose internal implementation details
through public APIs.
A module's public surface
should contain only what its consumers need.

## Formatting and Whitespace

### Blank Lines

Use blank lines to group related items.
Do not insert unnecessary blank lines
within a tightly related block of code.

### Line Breaking

Break lines that are too long.
When a function call or definition spans multiple lines,
align the continuation to improve readability.

## Error Messages

### Formatting Rules

- Start with a lowercase letter
  (unless the first word is a proper noun or identifier).
- Wrap identifiers in backticks:
  `` `new_root` is not a directory ``.
- Be specific:
  prefer "empty file system type"
  over "no fs type specified".
- Follow the style of Linux man page ERRORS sections
  when describing errno conditions.

Note: Some older code may not yet follow this convention.

## Design Principles

### Do Not Expose Implementation Details

This is the most frequently cited principle in reviews.
Internal data structures, helper types, and bookkeeping fields
should remain private.

### Set Correct Values Upfront

Initialize fields to their correct values at construction time
rather than relying on lazy initialization or post-construction fixup.

### Use Types to Encode Invariants

Leverage the type system to make illegal states unrepresentable.
Prefer newtypes and enums
over bare integers and boolean flags.

### Prefer Existing Abstractions

Before adding a new abstraction,
check whether an existing one already serves the purpose.
Unnecessary abstractions add cognitive overhead.

### Runtime Dispatch for Environment-Dependent Features

When a feature's availability depends on the runtime environment
(e.g., hardware capabilities),
use runtime dispatch rather than compile-time feature flags.

### Reference Linux but Justify Deviations

Asterinas aims for Linux compatibility.
When the implementation differs from Linux behavior,
document the reason.

### Provide Evidence for Non-Trivial Decisions

When a design choice is not obvious,
include a comment or commit message
explaining the reasoning and any alternatives considered.

## Comment Style

### Explain Why, Not How

Comments should explain the intent behind the code,
not restate what the code does.

### Consistent Style

Maintain a consistent comment style within each file.
Do not mix comment syntaxes without reason.
