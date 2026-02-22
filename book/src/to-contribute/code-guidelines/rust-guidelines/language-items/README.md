# Language Items

This section covers conventions
for specific Rust language constructs.
Each page addresses one category of language item
with actionable guidelines.

- **[Variables, Expressions, and Statements](variables-expressions-and-statements.md)** —
  Checked arithmetic, immutability,
  and explaining variables.
- **[Functions and Methods](functions-and-methods.md)** —
  Nesting and early returns, helper extraction,
  function size, and flag arguments.
- **[Types and Traits](types-and-traits.md)** —
  Enums vs. trait objects, type-level invariants,
  state simplification, and redundant `Option` elimination.
- **[Comments and Documentation](comments-and-documentation.md)** —
  Doc comment formatting, RFC 505, rustdoc links,
  module-level docs, dead code, and public API coverage.
- **[Unsafety](unsafety.md)** —
  SAFETY comments, `# Safety` sections,
  `deny(unsafe_code)` in kernel crates,
  scope minimization, and module-boundary reasoning.
- **[Modules and Crates](modules-and-crates.md)** —
  Visibility, encapsulation, workspace dependencies,
  and import conventions.
- **[Macros and Attributes](macros-and-attributes.md)** —
  Lint suppression, `#[expect]`, dead code policy,
  `rustfmt` configuration, and functions vs. macros.
- **[Conditional Compilation](conditional-compilation.md)** —
  Architecture selection with `cfg` attributes.
