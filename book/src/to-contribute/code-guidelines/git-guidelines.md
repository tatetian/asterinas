# Git Guidelines

## Commit Messages

Write commit messages in imperative mood
with the subject line at or below 72 characters.
Wrap identifiers in backticks.

Common prefixes used in the Asterinas commit log:

- `Fix` — correct a bug
- `Add` — introduce new functionality
- `Remove` — delete code or features
- `Refactor` — restructure without changing behavior
- `Rename` — change names of files, modules, or symbols
- `Implement` — add a new subsystem or feature
- `Enable` — turn on a previously disabled capability
- `Clean up` — minor tidying without functional change
- `Bump` — update a dependency version

Examples:

```
Fix deadlock in `Vmar::protect` when holding the page table lock

Add initial support for the io_uring subsystem

Refactor `TcpSocket` to separate connection state from I/O logic
```

If the commit requires further explanation,
add a blank line after the subject
followed by a body paragraph
describing the _why_ behind the change.

## Commit Organization

Each commit should represent one logical change.
Do not mix unrelated changes in a single commit.

### Amend the Right Commit

When fixing an issue discovered during review,
use `git rebase -i` to amend the commit
that introduced the issue
rather than appending a fixup commit at the end.

### Separate Refactoring from Features

If a feature requires preparatory refactoring,
put the refactoring in its own commit(s)
before the feature commit.
This makes each commit easier to review and bisect.

## Pull Requests

Keep pull requests focused on a single topic.
A PR that mixes a bug fix, a refactoring,
and a new feature is difficult to review.

Ensure that CI passes before requesting review.
If CI fails on an unrelated flake,
note it in the PR description.
