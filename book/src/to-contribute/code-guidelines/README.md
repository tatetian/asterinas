# Coding Guidelines

This section describes the coding and collaboration conventions
for the Asterinas project.
These guidelines aim to promote high-quality code
that are clean, consistent, maintainable, correct, and efficient.

The coding guidelines are organized into the following pages:

- **[General Guidelines](general-guidelines/README.md)** —
  Language-agnostic actionable advice
  covering naming, comments, layout, formatting,
  and API design.
- **[Rust Guidelines](rust-guidelines/README.md)** —
  Rust-specific conventions organized into
  naming, language items (variables, functions, types,
  comments, unsafety, modules, macros, conditional compilation),
  and select topics (idiomatics, concurrency,
  error handling, performance, logging,
  memory and resource management, unit testing).
- **[Git Guidelines](git-guidelines.md)** —
  Commit messages, atomic commits,
  refactoring separation, and focused PRs.
- **[Testing Guidelines](testing-guidelines.md)** —
  Test naming, assertions, cleanup,
  regression tests, and testing public APIs.
- **[Assembly Guidelines](asm-guidelines.md)** —
  Section directives, function attributes,
  label prefixes, and cross-architecture conventions.

The guidelines represent the _desired_ state.
If conflicting cases were found in the codebase,
they should be fixed according to the guidelines.

## Index

| Category | Guideline | Short Name |
|----------|-----------|-----|
| General | Be descriptive | [`descriptive-names`](general-guidelines/README.md#descriptive-names) |
| General | Avoid misleading names | [`no-misleading-names`](general-guidelines/README.md#no-misleading-names) |
| General | Encode units and important attributes in names | [`encode-units`](general-guidelines/README.md#encode-units) |
| General | Use assertion-style boolean names | [`bool-names`](general-guidelines/README.md#bool-names) |
| General | Explain why, not how | [`explain-why`](general-guidelines/README.md#explain-why) |
| General | Document design decisions | [`design-decisions`](general-guidelines/README.md#design-decisions) |
| General | Cite specifications and algorithm sources | [`cite-sources`](general-guidelines/README.md#cite-sources) |
| General | Never commit commented-out code | [`no-commented-code`](general-guidelines/README.md#no-commented-code) |
| General | One concept per file | [`one-concept-per-file`](general-guidelines/README.md#one-concept-per-file) |
| General | Organize code for top-down reading | [`top-down-reading`](general-guidelines/README.md#top-down-reading) |
| General | Format error messages consistently | [`error-message-format`](general-guidelines/README.md#error-message-format) |
| General | Hide implementation details | [`hide-impl-details`](general-guidelines/README.md#hide-impl-details) |
| General | Use types to encode invariants | [`type-invariants`](general-guidelines/README.md#type-invariants) |
| General | Fix root causes, not symptoms | [`fix-root-causes`](general-guidelines/README.md#fix-root-causes) |
| General | Validate at boundaries, trust internally | [`validate-at-boundaries`](general-guidelines/README.md#validate-at-boundaries) |
| Rust | Omit `get_` prefix on simple getters | [`no-get-prefix`](rust-guidelines/naming.md#no-get-prefix) |
| Rust | Use full English words, not abbreviations | [`no-abbreviations`](rust-guidelines/naming.md#no-abbreviations) |
| Rust | Names must accurately reflect behavior | [`accurate-names`](rust-guidelines/naming.md#accurate-names) |
| Rust | Follow Rust CamelCase and acronym capitalization | [`camel-case-acronyms`](rust-guidelines/naming.md#camel-case-acronyms) |
| Rust | End closure variables with `_fn` | [`closure-fn-suffix`](rust-guidelines/naming.md#closure-fn-suffix) |
| Rust | Use assertion-style boolean names | [`rust-bool-names`](rust-guidelines/naming.md#rust-bool-names) |
| Rust | Use checked or saturating arithmetic | [`checked-arithmetic`](rust-guidelines/language-items/variables-expressions-and-statements.md#checked-arithmetic) |
| Rust | Prefer immutable bindings | [`prefer-immutable`](rust-guidelines/language-items/variables-expressions-and-statements.md#prefer-immutable) |
| Rust | Introduce explaining variables | [`explain-variables`](rust-guidelines/language-items/variables-expressions-and-statements.md#explain-variables) |
| Rust | Minimize nesting; use early returns and `let-else` | [`early-returns`](rust-guidelines/language-items/functions-and-methods.md#early-returns) |
| Rust | Extract coherent logic into named helpers | [`extract-helpers`](rust-guidelines/language-items/functions-and-methods.md#extract-helpers) |
| Rust | Keep functions small and focused | [`small-functions`](rust-guidelines/language-items/functions-and-methods.md#small-functions) |
| Rust | Avoid flag arguments | [`no-flag-args`](rust-guidelines/language-items/functions-and-methods.md#no-flag-args) |
| Rust | Prefer enum over trait objects for closed sets | [`enum-over-dyn`](rust-guidelines/language-items/types-and-traits.md#enum-over-dyn) |
| Rust | Use types to encode invariants | [`rust-type-invariants`](rust-guidelines/language-items/types-and-traits.md#rust-type-invariants) |
| Rust | Collapse redundant state into simpler types | [`simplify-state`](rust-guidelines/language-items/types-and-traits.md#simplify-state) |
| Rust | Eliminate redundant `Option` wrapping | [`no-redundant-option`](rust-guidelines/language-items/types-and-traits.md#no-redundant-option) |
| Rust | Use `///` doc comments with semantic line breaks | [`semantic-doc-comments`](rust-guidelines/language-items/comments-and-documentation.md#semantic-doc-comments) |
| Rust | Follow RFC 505 summary line conventions | [`rfc505-summary`](rust-guidelines/language-items/comments-and-documentation.md#rfc505-summary) |
| Rust | Wrap identifiers in backticks | [`backtick-identifiers`](rust-guidelines/language-items/comments-and-documentation.md#backtick-identifiers) |
| Rust | Do not disclose implementation details in doc comments | [`no-impl-in-docs`](rust-guidelines/language-items/comments-and-documentation.md#no-impl-in-docs) |
| Rust | `TODO`/`FIXME` in `//` comments, not `///` doc comments | [`todo-in-regular-comments`](rust-guidelines/language-items/comments-and-documentation.md#todo-in-regular-comments) |
| Rust | Comprehensive doc coverage for all public APIs | [`doc-all-public`](rust-guidelines/language-items/comments-and-documentation.md#doc-all-public) |
| Rust | Add module-level documentation for major components | [`module-docs`](rust-guidelines/language-items/comments-and-documentation.md#module-docs) |
| Rust | Delete dead code aggressively | [`delete-dead-code`](rust-guidelines/language-items/comments-and-documentation.md#delete-dead-code) |
| Rust | Justify every use of `unsafe` | [`safety-comments`](rust-guidelines/language-items/unsafety.md#safety-comments) |
| Rust | Document safety requirements | [`safety-docs`](rust-guidelines/language-items/unsafety.md#safety-docs) |
| Rust | Deny unsafe code in `kernel/` | [`deny-unsafe-kernel`](rust-guidelines/language-items/unsafety.md#deny-unsafe-kernel) |
| Rust | Minimize and encapsulate `unsafe` scope | [`minimize-unsafe`](rust-guidelines/language-items/unsafety.md#minimize-unsafe) |
| Rust | Reason about safety at the module boundary | [`module-boundary-safety`](rust-guidelines/language-items/unsafety.md#module-boundary-safety) |
| Rust | Default to the narrowest visibility | [`narrow-visibility`](rust-guidelines/language-items/modules-and-crates.md#narrow-visibility) |
| Rust | Encapsulate fields behind getters | [`getter-encapsulation`](rust-guidelines/language-items/modules-and-crates.md#getter-encapsulation) |
| Rust | Use workspace dependencies | [`workspace-deps`](rust-guidelines/language-items/modules-and-crates.md#workspace-deps) |
| Rust | Follow the three-group import convention | [`import-groups`](rust-guidelines/language-items/modules-and-crates.md#import-groups) |
| Rust | Suppress lints at the narrowest scope | [`narrow-lint-suppression`](rust-guidelines/language-items/macros-and-attributes.md#narrow-lint-suppression) |
| Rust | [When to `#[expect(dead_code)]`](rust-guidelines/language-items/macros-and-attributes.md#expect-dead-code) | [`expect-dead-code`](rust-guidelines/language-items/macros-and-attributes.md#expect-dead-code) |
| Rust | Format with `rustfmt` | [`rustfmt`](rust-guidelines/language-items/macros-and-attributes.md#rustfmt) |
| Rust | Prefer functions over macros when possible | [`functions-over-macros`](rust-guidelines/language-items/macros-and-attributes.md#functions-over-macros) |
| Rust | Use `cfg` attributes for architecture-specific modules | [`cfg-arch-modules`](rust-guidelines/language-items/conditional-compilation.md#cfg-arch-modules) |
| Rust | Use the `?` operator, not chains of `.unwrap()` | [`question-mark-operator`](rust-guidelines/select-topics/idiomatics.md#question-mark-operator) |
| Rust | Choose the right interior mutability primitive | [`interior-mutability`](rust-guidelines/select-topics/idiomatics.md#interior-mutability) |
| Rust | Establish and enforce a consistent lock order | [`lock-ordering`](rust-guidelines/select-topics/concurrency-and-races.md#lock-ordering) |
| Rust | Never do I/O or blocking operations while holding a spinlock | [`no-io-under-spinlock`](rust-guidelines/select-topics/concurrency-and-races.md#no-io-under-spinlock) |
| Rust | Do not use atomics casually | [`careful-atomics`](rust-guidelines/select-topics/concurrency-and-races.md#careful-atomics) |
| Rust | Critical sections must not be split across lock boundaries | [`atomic-critical-sections`](rust-guidelines/select-topics/concurrency-and-races.md#atomic-critical-sections) |
| Rust | Be mindful of drop ordering | [`drop-ordering`](rust-guidelines/select-topics/concurrency-and-races.md#drop-ordering) |
| Rust | Propagate errors with `?` | [`propagate-errors`](rust-guidelines/select-topics/error-handling.md#propagate-errors) |
| Rust | Error messages must be informative and consistent | [`informative-errors`](rust-guidelines/select-topics/error-handling.md#informative-errors) |
| Rust | Fix root causes, not symptoms | [`rust-fix-root-causes`](rust-guidelines/select-topics/error-handling.md#rust-fix-root-causes) |
| Rust | Use `debug_assert` for correctness-only checks | [`debug-assert`](rust-guidelines/select-topics/error-handling.md#debug-assert) |
| Rust | Avoid O(n) algorithms on hot paths | [`no-linear-hot-paths`](rust-guidelines/select-topics/performance.md#no-linear-hot-paths) |
| Rust | Minimize unnecessary copies and allocations | [`minimize-copies`](rust-guidelines/select-topics/performance.md#minimize-copies) |
| Rust | No premature optimization without evidence | [`no-premature-optimization`](rust-guidelines/select-topics/performance.md#no-premature-optimization) |
| Rust | Use `log` crate macros exclusively | [`log-crate-only`](rust-guidelines/select-topics/logging.md#log-crate-only) |
| Rust | Choose appropriate log levels | [`log-levels`](rust-guidelines/select-topics/logging.md#log-levels) |
| Rust | Use RAII for all resource acquisition and release | [`raii`](rust-guidelines/select-topics/memory-and-resource-management.md#raii) |
| Rust | Mark memory-safety-affecting functions `unsafe` | [`unsafe-memory-ops`](rust-guidelines/select-topics/memory-and-resource-management.md#unsafe-memory-ops) |
| Rust | Name tests after observable behavior | [`behavior-test-names`](rust-guidelines/select-topics/unit-testing.md#behavior-test-names) |
| Rust | Require regression tests for every bug fix | [`regression-tests`](rust-guidelines/select-topics/unit-testing.md#regression-tests) |
| Rust | Test user-visible behavior, not implementation internals | [`test-visible-behavior`](rust-guidelines/select-topics/unit-testing.md#test-visible-behavior) |
| Git | Write imperative, descriptive subject lines | [`imperative-subject`](git-guidelines.md#imperative-subject) |
| Git | One logical change per commit | [`atomic-commits`](git-guidelines.md#atomic-commits) |
| Git | Separate refactoring from features | [`refactor-then-feature`](git-guidelines.md#refactor-then-feature) |
| Git | Keep pull requests focused | [`focused-prs`](git-guidelines.md#focused-prs) |
| Test | Name tests after observable behavior | [`behavior-names`](testing-guidelines.md#behavior-names) |
| Test | Use assertion macros, not manual inspection | [`use-assertions`](testing-guidelines.md#use-assertions) |
| Test | Clean up resources after every test | [`test-cleanup`](testing-guidelines.md#test-cleanup) |
| Test | Require regression tests for every bug fix | [`require-regression-tests`](testing-guidelines.md#require-regression-tests) |
| Test | Test user-visible behavior, not internals | [`test-public-apis`](testing-guidelines.md#test-public-apis) |
| Assembly | Use the correct section directive | [`section-directives`](asm-guidelines.md#section-directives) |
| Assembly | Place code-width directives after the section definition | [`code-width-directives`](asm-guidelines.md#code-width-directives) |
| Assembly | Place attributes directly before the function | [`function-attributes`](asm-guidelines.md#function-attributes) |
| Assembly | Add `.type` and `.size` for Rust-callable functions | [`type-and-size`](asm-guidelines.md#type-and-size) |
| Assembly | Use unique label prefixes to avoid name clashes | [`label-prefixes`](asm-guidelines.md#label-prefixes) |
| Assembly | Prefer `.balign` over `.align` | [`prefer-balign`](asm-guidelines.md#prefer-balign) |
