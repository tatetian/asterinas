# Review Guidance

This document is the canonical source for the review philosophy used by
the `aster-pr-review` skill. Human reviewers (via the slash command) and
CI-driven reviewers (via the Codex workflow) follow these conventions so
reviews look consistent regardless of origin. Update this file when the
philosophy changes — do not embed policy in the workflow YAML or in
ad-hoc prompts.

You are reviewing a pull request for the Asterinas kernel.

## Three zoom levels

Cover issues at all three levels when they exist (no need to label them
in the output — just make sure problems at each level are surfaced):

* **Design** (macro): architecture, API surface clarity, type safety,
  module boundaries, whether new pieces fit the existing structure, and
  overarching design principles (single responsibility, DRY, information
  hiding, least surprise, loose coupling, consistency).

* **Correctness** (meso): logical bugs, off-by-one errors, error-handling
  gaps, race conditions, lock ordering, atomic-ordering pairing, Linux
  ABI / behavioral conformance when the PR touches a syscall, /proc,
  /sys, or other Linux-facing surface, and test-coverage adequacy.

* **Craft** (micro): naming, consistency in style and patterns across
  the diff and with surrounding code, unnecessary clones / heap
  allocations / atomic ops on hot paths, and conformance to the coding
  guidelines in `book/src/to-contribute/coding-guidelines/`. For each
  guideline citation, build a hyperlink: take the guideline's relative
  path from the Index in `README.md` (e.g.,
  `rust-guidelines/language-items/functions-and-methods.md#small-functions`),
  prepend `https://asterinas.github.io/book/to-contribute/coding-guidelines/`,
  and replace the trailing `.md` with `.html`.

## Severity

Each comment carries one of three severities:

* **blocker** — must fix before merge (a real bug, a missing critical
  guard, a Linux ABI violation, …).
* **significant** — should fix before merge (a clear issue, not a
  release-blocker).
* **minor** — style or low-impact polish.

## One root cause per comment

If several symptoms share one underlying bug or design flaw, file ONE
comment that names the root cause and gives the single fix that resolves
all of them. Cross-reference the other symptom lines in the body rather
than filing per-symptom comments.

## End with a concrete fix

Each comment's body should end with a concrete fix: a diff, a code
snippet, a one-liner command, or a specific API / flag to use. Only omit
a fix when the issue requires design discussion first — and say so
explicitly ("No fix suggested: this needs a design decision on X.").

## Anchoring

A comment's line range must be on lines visible on the RIGHT side of the
PR's unified diff (added lines or the context lines around them).
Pre-existing lines outside any diff hunk are NOT anchorable — GitHub
silently drops comments that target them. If you need to call out
pre-existing code, re-anchor to a related diff line and describe the
pre-existing problem in the comment body.

## Paths

Use repository-relative paths (e.g., `kernel/src/foo.rs`), NOT absolute
filesystem paths (e.g., `/home/runner/work/...`).
