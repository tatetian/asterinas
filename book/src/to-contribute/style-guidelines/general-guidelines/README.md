# General Guidelines

This page presents the fundamental philosophy and principles
that guide all software development in the Asterinas project.
These ideas are distilled from classic software engineering literature
and refined through thousands of code reviews.

For concrete, actionable advice, see [Themes](themes.md).
For Rust-specific conventions, see
[Rust Guidelines](../rust-guidelines/README.md).

## Philosophy

### Minimize time to understand

Code should be written to minimize the time
it would take someone else to fully understand it.
This is the fundamental theorem of readability
and the single most important measure
of code quality in this project.
"Someone else" includes your future self.

Code is read far more often than it is written.
If a technique makes code shorter
but harder to follow at a glance,
choose clarity over brevity.

### Managing complexity is the primary technical imperative

No one can hold an entire modern program in their head.
The purpose of every technique in software construction —
decomposition, naming, encapsulation, abstraction —
is to break complex problems into simple pieces
so that you can safely focus on one thing at a time.

### Self-documenting code first

The best comment is the one you found a way not to write
by making the code clear enough.
If a comment is needed to explain _what_ code does,
first try to rewrite the code.
Comments should explain _why_ (intent, design decisions, tradeoffs),
not _what_ or _how_.
Do not comment tricky code — rewrite it to be straightforward.

### Craftsmanship and care

Clean code looks like it was written by someone who cares.
Professionalism means never knowingly leaving a mess.
The only way to go fast is to keep the code clean at all times.

### Continuous improvement

Leave code cleaner than you found it.
Small, steady improvements —
renaming a variable, extracting a function,
eliminating duplication —
prevent code from rotting over time.

## Principles

### Single Responsibility

Each module, type, or function
should have one, and only one, reason to change.
If you cannot describe what a unit does
without the words "and," "or," or "but,"
it has too many responsibilities.

### Don't Repeat Yourself (DRY)

Every piece of knowledge
should have a single, unambiguous representation.
Duplication harms readability and maintainability.
When the same pattern appears three or more times,
eliminate the duplication (e.g., adding a helper function).

### Information Hiding

Hide design decisions behind well-defined interfaces.
A module's public surface should contain
only what its consumers need.
Internal data structures, helper types,
and bookkeeping fields should remain private.

### Principle of Least Surprise

Functions, types, and APIs should behave
as their names and signatures suggest.
When an obvious behavior is not implemented,
readers lose trust in the codebase
and must fall back on reading implementation details.

### Loose Coupling, Strong Cohesion

Connections between modules should be
small, visible, and flexible.
Within a module, every part should contribute
to a single, well-defined purpose.

### Consistency

Do similar things the same way throughout the codebase.
Consistency reduces surprise and cognitive load
even when neither approach is objectively superior.
When a convention already exists, follow it;
do not introduce a competing convention
without compelling justification.

### Iterate

Software design is heuristic.
A first attempt may work but is rarely the best.
Revisit and refine —
each attempt produces insight.
