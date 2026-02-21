# General Guidelines

This page presents the fundamental philosophy and principles
that guide all software development in the Asterinas project.
These ideas are distilled from classic software engineering literature
and refined through thousands of code reviews.

For concrete, actionable advice, see [Themes](themes.md).
For Rust-specific conventions, see
[Rust Guidelines](../rust-guidelines/README.md).

## Philosophy

### Readability is paramount

Code is read far more often than it is written.
Every coding decision should minimize
the time it takes another person to understand the code.
If a technique makes code shorter
but harder to follow at a glance,
choose clarity over brevity.

### Managing complexity is the primary technical imperative

No one can hold an entire modern program in their head.
The purpose of every technique in software construction —
decomposition, naming, encapsulation, abstraction —
is to break complex problems into simple pieces
so that you can safely focus on one thing at a time.

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

### Humility

The best programmers compensate
for the limits of human short-term memory.
Every good practice — decomposition, reviews,
short routines, clear names — exists
to reduce cognitive load on the programmer.

## Principles

### Single Responsibility

Each module, class, or function
should have one, and only one, reason to change.
If you cannot describe what a unit does
without the words "and," "or," or "but,"
it has too many responsibilities.

### Don't Repeat Yourself (DRY)

Every piece of knowledge
should have a single, unambiguous representation.
Duplication harms readability and maintainability.
When the same pattern appears three or more times,
extract it into a reusable helper.

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

### Iterate

Software design is heuristic.
A first attempt may work but is rarely the best.
Revisit and refine —
each attempt produces insight.
