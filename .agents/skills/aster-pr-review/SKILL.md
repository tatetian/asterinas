---
name: aster-pr-review
description: >
  Generate and submit GitHub PR code reviews. Use when the user wants
  to review a pull request, generate review comments, or submit review
  feedback to GitHub.
compatibility: Requires gh (GitHub CLI) and git
argument-hint: <new|submit|redo|delete> <pr_number_or_url>
disable-model-invocation: true
allowed-tools: Bash(gh *), Bash(git *), Read, Write, Glob, Grep, Agent
---

# PR Review Skill

You are a code review assistant. The user provides a subcommand (`new`,
`submit`, `redo`, or `delete`) and a PR number or GitHub URL.

Subcommand: $0
PR: $1

## Subcommand: `new`

Generate a structured code review for the given PR. Follow these steps
exactly:

### Step 1: Resolve PR identity

Extract the PR number from the input. If a full GitHub URL is given
(e.g., `https://github.com/owner/repo/pull/123`), extract the number.
If only a number is given, use the current repo.

### Step 2: Fetch PR metadata

Determine the `owner/repo` for the PR. If the user gave a URL, extract it.
If only a number, use the repo from `gh repo view --json nameWithOwner -q .nameWithOwner`.

```bash
gh pr view <N> --repo <owner/repo> --json number,title,headRefOid,baseRefName,headRefName,url,body
```

Save the `headRefOid` as the head SHA, `title`, and `body` (PR description)
for later use.

### Step 3: Resolve the repo root

All paths in this skill are relative to the **repo root**. Resolve it
once and use it throughout:

```bash
REPO_ROOT="$(git rev-parse --show-toplevel)"
```

### Step 4: Create a git worktree with the PR's source

Fetch the PR ref. Determine which git remote points to the repo that
hosts the PR (compare remote URLs against the repo from Step 2).
Typically this is `upstream` for upstream PRs or `origin` for fork PRs.

```bash
git fetch <remote> +pull/<N>/head:refs/pr/<N>
```

Check if a worktree already exists at `$REPO_ROOT/pr_reviews/<N>`:
```bash
git worktree list | grep "pr_reviews/<N>"
```

If it exists, remove it first:
```bash
git worktree remove "$REPO_ROOT/pr_reviews/<N>" --force
```

Then create the worktree:
```bash
git worktree add "$REPO_ROOT/pr_reviews/<N>" refs/pr/<N>
```

### Step 5: Ensure `pr_reviews/.gitignore` exists

If `$REPO_ROOT/pr_reviews/.gitignore` does not exist, create it with
content `*` so that the entire directory is ignored by git.

### Step 6: Fetch the PR diff and the valid-lines map

```bash
gh pr diff <N> --repo <owner/repo> > /tmp/pr_<N>.diff
```

Also compute the **valid-lines map** — the set of line numbers per file on
which GitHub will accept inline comments. GitHub silently rejects comments
anchored to lines that aren't part of any diff hunk on the RIGHT side
(common pitfall: a *modified* file looks complete in the worktree, but only
its added/changed lines and a few context lines around them are in the
diff). Run:

```bash
python3 ${CLAUDE_SKILL_DIR}/scripts/validate_review.py \
    --diff /tmp/pr_<N>.diff \
    --emit-valid-lines > /tmp/pr_<N>_valid_lines.json
```

The resulting JSON maps each path to a list of inclusive `[start, end]`
ranges of anchorable RIGHT-side lines, e.g.
`{"path/to/file.rs": [[42, 58], [73, 89]], ...}`. A line `N` is anchorable
for a file iff some range `[s, e]` in its list satisfies `s <= N <= e`.
Pass this file path to every sub-agent (Step 7b) and the verifier (Step 7e)
so they can check anchors before assigning them.

### Step 7: Generate the review file

Generate the review using **parallel sub-agent review**. This technique
produces higher-quality reviews than a single pass because each sub-agent
focuses deeply on one aspect without context dilution.

#### Step 7a: Prepare context for sub-agents

First, read the diff and save it. Identify the list of changed files
and the PR description. Each sub-agent will need:
- The PR diff (saved at `/tmp/pr_<N>.diff` in Step 6)
- The PR description and title
- Access to the source tree at `$REPO_ROOT/pr_reviews/<N>/`
- The project's review guidelines (from CLAUDE.md or AGENTS.md)
- The review file format specification (from this skill)
- **The valid-lines map at `/tmp/pr_<N>_valid_lines.json`** (from Step 6) — the set of line numbers per file on which GitHub will accept inline comments. Modified files (vs added) include only their changed lines plus a few context lines around them; pre-existing lines outside any diff hunk are NOT anchorable.

#### Step 7b: Spawn parallel sub-agent reviews

Launch **multiple Agent sub-agents in parallel** (in a single message
with multiple Agent tool calls), each focusing on one of the review
aspects below. Each sub-agent should explore the source tree in
`$REPO_ROOT/pr_reviews/<N>/` for full context beyond the diff.

Each sub-agent should return its findings as a list of review comments
in the review file format (`` ## `path/to/file` line N `` headings with
bodies). The sub-agent does NOT write to a file — it returns its
comments as text in its response.

**Anchoring rule (mandatory)**: before assigning a `line N` to any heading,
the sub-agent must check `/tmp/pr_<N>_valid_lines.json` and confirm that
`N` is in the list for that path. Never anchor to a line outside the map —
GitHub will silently drop the comment. The escape depends on whether the
file appears in the PR's diff at all:

*Case A — file IS in the diff, but the issue is on a pre-existing line
this PR didn't modify.* Choose ONE:
1. **Re-anchor** to a nearby line that IS in the map (e.g., a newly added
   line that introduces or interacts with the issue), and explain in the
   comment body which other (pre-existing) lines also need fixing.
2. **Convert to file-level** by dropping the `line N` suffix:
   `` ## `path/to/file` `` — for issues that are inherently file-wide
   (e.g., "add `#![deny(unsafe_code)]` at the crate root") or for
   commentary on pre-existing code that this PR builds on.

*Case B — file is NOT in the diff at all.* Neither escape above works:
a file-level comment on an untouched file is also silently dropped, and
re-anchoring within the same file is impossible. First ask whether the
finding is genuinely in-scope for this PR. A code review is scoped to
the diff under review; drive-by observations about the rest of the
codebase belong in a separate issue, not in this review. Then:
1. **In-scope** (the PR's correctness depends on the untouched code, or
   the PR newly relies on a broken precondition there): re-anchor the
   comment to a line in a *touched* file that interacts with the issue,
   and describe the untouched-file problem in the body.
2. **Out-of-scope but worth noting**: fold the observation into the
   `# Summary` section as a brief "related / out-of-scope" note, ideally
   suggesting a follow-up issue. Do NOT emit it as an inline comment.
3. **Not actionable for this PR**: drop the finding.

**Review aspects** (one sub-agent per aspect — three sub-agents, each
reviewing the PR at a different zoom level):

1. **Design** (macro — *does the shape of the change make sense?*):
   architecture and high-level design; API surface clarity, type
   safety, ease of correct use, difficulty of misuse; module
   organization and abstraction boundaries; whether new features,
   modules, options, or tests fit the existing structure or demand
   adjustments to it; module- and crate-level documentation
   completeness; conformance with overarching design principles
   such as Single Responsibility, Don't Repeat Yourself, Information
   Hiding, Open/Closed, Least Surprise, Loose Coupling and Strong
   Cohesion, and Consistency. If the PR introduces a Linux-facing
   interface, judge the *shape* of that interface against Linux
   (the *behavior* check belongs in Correctness).

2. **Correctness** (meso — *does the code do the right thing?*):
   logical bugs, off-by-one errors, incorrect state transitions,
   unhandled edge cases, violation of documented invariants;
   error handling gaps and whether error paths clean up state;
   safety justifications for unsafe operations; race conditions,
   lock ordering, deadlock and livelock potential, missing
   synchronization, locks held across I/O or blocking calls,
   atomic ordering (Acquire/Release pairing), and whether
   documented locking protocols match the code; whether locks
   protect the right region (neither too small nor too large) and
   whether the most suitable synchronization primitive is chosen;
   Linux ABI / behavioral conformance when this PR touches a
   syscall, /proc, /sys, or other Linux-compatible surface; and
   *test-coverage adequacy* — whether the changed code is
   adequately tested, including regression tests for bug fixes
   and concurrent stress tests for concurrency-sensitive code.

3. **Craft** (micro — *is the code well-written?*): descriptive
   and accurate names for modules, types, functions, and
   variables; consistency in naming, style, and coding patterns
   across the diff and with surrounding code; consistent ordering
   of items (e.g., methods within an impl, fields within a
   struct); unnecessary copies, clones, heap allocations, and
   atomic operations on hot paths; missed opportunities for
   batched I/O or pre-allocation; conformance to the project's
   coding guidelines (in `book/src/to-contribute/coding-guidelines/`
   and AGENTS.md / CLAUDE.md), including function size, nesting
   depth, error propagation style, visibility modifiers, doc
   comment conventions, and lint suppression scope; and the
   *quality of test code* — readability, naming, structure, and
   whether mock-based testing would add value.

Not all aspects apply to every PR. Skip aspects that are not relevant (e.g., skip Correctness's concurrency sub-questions for a pure documentation change, or skip Design entirely for a one-line bug fix). For very large PRs (many files, wide-ranging changes), prefer running the three sub-agents in two sequential waves of split scope (e.g., Correctness over kernel/, then Correctness over libs/) rather than adding more concurrent agents — context blowup in the merge step is worse than the extra wall-clock.

#### Step 7c: Merge sub-agent results

After all sub-agents complete:

1. **Collect** all comments from all sub-agents.
2. **Deduplicate**: if two sub-agents found the same issue (e.g., Design flags an API as misuse-prone and Correctness flags a caller that misused it; or Craft flags a misleading name and Correctness flags the bug the misleading name enabled), keep the most complete version and discard the other.
3. **Merge shared-root-cause comments**: when several comments describe different symptoms of the same underlying bug or design flaw, collapse them into a single comment that names the root cause once and proposes the single fix that resolves all symptoms. Do not leave one comment per symptom; that forces the author to re-derive the connection. Cross-reference the symptom lines in prose within the merged comment.
4. **Group by subject**: order comments so that those targeting the same file, module, subsystem, or topic sit next to one another. The reader should be able to read the review top-to-bottom and pick up related issues in one pass, without jumping around the file tree.
5. **Order within each group by severity**: blockers first, then significant, then minor.
6. **Write the summary**: synthesize cross-cutting themes into the `# Summary` section. Mention what the PR does well, the top 2-3 issues that need attention, and any structural recommendations (not a bulleted list of every comment).

#### Step 7d: Write the draft review file

Generate a timestamp for the review file name:
```bash
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
```

Write the merged, deduplicated, grouped review to `$REPO_ROOT/pr_reviews/<N>/review-<TIMESTAMP>.md` using the format documented below in "Review file format". Do not create the convenience symlink yet — wait until after verification (Step 7e).

#### Step 7e: Verify the draft with an independent sub-agent

Sub-agents routinely hallucinate specific facts: line numbers drift, file modes are guessed, proposed fixes reference flags that don't exist, suggested code calls functions with the wrong signature. A single verification pass catches most of these before they reach the user or GitHub.

Spawn ONE additional sub-agent with a **clean context** (a fresh Agent invocation, not a continuation of an earlier sub-agent). Brief it with:
- The absolute path to the draft review file
- The absolute path to the PR worktree (`$REPO_ROOT/pr_reviews/<N>/`)
- The absolute path to the valid-lines map (`/tmp/pr_<N>_valid_lines.json`)
- Explicit instruction to verify:
  1. **Every concrete claim** — line numbers, file modes, behavior assertions about shell / Rust / framework semantics, named functions and constants, external tool/flag names.
  2. **Every verdict** — is "blocker" warranted? Is "minor" understated?
  3. **Every fix snippet** — does the suggested code actually compile, parse, or otherwise do what the comment says? Does the suggested command flag exist? Does the suggested API field exist?
  4. **Every comment heading** — cross-check against the valid-lines map. For a line-anchored heading `` ## `path` line N ``, the line must be in the map for that path. For a file-level heading `` ## `path` ``, the path must be a key in the map (i.e., the file appears in the PR's diff). Any heading that fails either check will be silently dropped by GitHub on submit; flag those — and indicate whether the file is in the diff at all, since that decides between re-anchoring within the file, re-anchoring to a related touched file, or moving the observation into `# Summary` (see the anchoring rule's Case A vs Case B).
- Instruction to return a structured report (claims-confirmed / claims-with-errors / claims-unverifiable) and NOT rewrite the review itself.

Apply the reported fixes to the draft. For claims the sub-agent could not verify from local files (e.g., external documentation), either hedge the language ("per nixpkgs convention...") or cite a source; do not leave unhedged assertions the sub-agent flagged as unverifiable.

#### Step 7f: Finalize

After verification fixes are applied, create (or update) the convenience symlink:
```bash
ln -sf "<N>/review-<TIMESTAMP>.md" "$REPO_ROOT/pr_reviews/<N>.md"
```

The symlink uses a relative target so it works regardless of where the repo is cloned.

### Step 8: Validate comment headings

After writing the review file, validate that every `##` heading matches
the submit parser's regex. Run this check:

```bash
python3 -c "
import re, sys
heading_re = re.compile(
    r'^##\s+\x60([^\x60]+)\x60'
    r'(?:\s+lines?\s+(\d+)(?:-(\d+))?)?'
    r'(\s+\(old\))?\s*$',
    re.MULTILINE
)
with open(sys.argv[1]) as f:
    content = f.read()
parts = content.split('\n')
fm = 0
start = 0
for i, line in enumerate(parts):
    if line.strip() == '---':
        fm += 1
        if fm == 2:
            start = i + 1
            break
body = '\n'.join(parts[start:])
sections = re.split(r'\n---\s*\n', body)
fails = []
for section in sections:
    for line in section.strip().split('\n'):
        if line.startswith('## '):
            if not heading_re.match(line):
                fails.append(line)
            break
if fails:
    print(f'WARNING: {len(fails)} comment heading(s) will be SKIPPED during submit:')
    for f in fails:
        print(f'  {f[:90]}')
    sys.exit(1)
else:
    print('All comment headings are valid.')
" "$REPO_ROOT/pr_reviews/<N>/review-<TIMESTAMP>.md"
```

If any headings fail validation, fix them before reporting to the user.
Use the file-level format (`` ## `path/to/file` ``) for comments that
don't target a specific line range.

Also run the **line-in-diff** check, which catches the more insidious
failure mode where a heading is well-formed but anchors to a line GitHub
will silently reject:

```bash
python3 ${CLAUDE_SKILL_DIR}/scripts/validate_review.py \
    --review "$REPO_ROOT/pr_reviews/<N>/review-<TIMESTAMP>.md" \
    --diff /tmp/pr_<N>.diff
```

This script exits non-zero with a per-comment list of problems if any
line-anchored heading targets a line not in the diff. For each, it prints
the nearest valid lines (so you can re-anchor) and reminds you to drop
the `line N` suffix to make the comment file-level. Fix the review file
before proceeding to Step 9 — the same check runs again at submit time
and refuses to submit if anything is invalid.

### Step 9: Report to user

Print the path to the generated file and how many comments were generated.
Tell the user they can edit the file and then run `/aster-pr-review submit <N>`
when ready.

---

## Subcommand: `submit`

Parse and submit the review from `pr_reviews/<N>.md` to GitHub. Follow
these steps exactly:

### Step 1: Resolve the repo root

```bash
REPO_ROOT="$(git rev-parse --show-toplevel)"
```

### Step 2: Read the review file

Read `$REPO_ROOT/pr_reviews/<N>.md` (this is a symlink to the latest
review file). If it does not exist, tell the user to run
`/aster-pr-review new <N>` first.

### Step 3: Parse the file

Extract from the file:
1. **Frontmatter**: YAML between the first pair of `---` lines. Required
   fields: `pr`, `repo`, `head_sha`, `event`.
2. **Summary**: Text between `# Summary` and the next `---`.
3. **Comments**: Each `---`-delimited section with a `##` heading.

For each comment section:
- Parse the heading to extract: file path (in backticks), line number(s),
  and `(old)` suffix if present.
- Strip any blockquoted lines (lines starting with `> `).
- The remaining text is the comment body.
- Skip comments with empty bodies.

### Step 4: Check for staleness

```bash
gh pr view <N> --json headRefOid -q .headRefOid
```

Compare the result against `head_sha` from frontmatter. If they differ,
**refuse to submit**. Tell the user:
> The PR has been updated since this review was generated.
> Run `/aster-pr-review redo <N>` to generate a follow-up review.

### Step 5: Submit via GitHub API

Run the `scripts/submit_review.sh` script that is bundled with this skill:

```bash
bash ${CLAUDE_SKILL_DIR}/scripts/submit_review.sh "$REPO_ROOT/pr_reviews/<N>.md"
```

This script parses the review file and calls the GitHub API to create a
pending review. Before any GraphQL mutation, it runs `validate_review.py`
as a **pre-flight check**: if any line-anchored heading targets a line
that isn't in the PR diff, the script aborts with a per-comment list of
problems (and the nearest valid lines as a hint). Fix the review file —
either re-anchor or convert to file-level — and re-run.

### Step 6: Report results

The submit script performs **post-submit verification automatically**: after creating the review threads, it re-queries the review via GraphQL and checks that every expected (path, line) pair is actually present on GitHub. This catches silent failures where GitHub accepts the mutation but drops the thread (e.g., the line isn't in the diff, the path is wrong, or the mutation was accepted but detached).

Relay the script's full output to the user, including:
- Number of comments submitted and any skipped with reasons
- The verification summary (`Expected on review: N; found on GitHub: M`)
- Any list of missing comments and the hints for common causes

If the verification reports missing comments, do NOT claim the submit succeeded cleanly. Tell the user which comments are missing and offer to (a) edit the review file to fix the offending comments and re-submit, or (b) add them manually on GitHub.

Tell the user the review is in PENDING state and they should go to GitHub to finalize it.

---

## Subcommand: `redo`

Generate a follow-up review after the PR author has pushed updates.
This subcommand re-fetches the PR, reads the previous review for context,
and produces a new review that verifies whether previous comments have
been addressed and checks for newly introduced issues.

### Step 1: Resolve PR identity

Same as `new` Step 1.

### Step 2: Fetch PR metadata

Same as `new` Step 2.

### Step 3: Resolve the repo root

Same as `new` Step 3.

### Step 4: Locate the previous review

Read the current symlink at `$REPO_ROOT/pr_reviews/<N>.md` to find the
most recent review file. Read its full contents -- this is the
**previous review** that provides context for the follow-up.

If no previous review exists, fall back to the `new` subcommand behavior
(i.e., generate a fresh review without prior context).

### Step 5: Update the git worktree

Re-fetch the PR ref to get the latest code:
```bash
git fetch <remote> +pull/<N>/head:refs/pr/<N>
```

Remove and recreate the worktree to pick up the new code:
```bash
git worktree remove "$REPO_ROOT/pr_reviews/<N>" --force
git worktree add "$REPO_ROOT/pr_reviews/<N>" refs/pr/<N>
```

Note: the review files inside the worktree directory are lost when the
worktree is removed. This is fine because the previous review has
already been read in Step 4.

### Step 6: Fetch the new PR diff and the valid-lines map

```bash
gh pr diff <N> --repo <owner/repo> > /tmp/pr_<N>.diff
python3 ${CLAUDE_SKILL_DIR}/scripts/validate_review.py \
    --diff /tmp/pr_<N>.diff \
    --emit-valid-lines > /tmp/pr_<N>_valid_lines.json
```

Same as `new` Step 6 — the valid-lines map is fed into every sub-agent and
the verifier so anchors land inside actual diff hunks (see `new` Step 6
for the rationale).

### Step 7: Generate the follow-up review

Follow the same parallel sub-agent flow as `new` Steps 7a through 7f (review, merge, write-draft, verify, finalize), with these additions:

- **Each parallel sub-agent (7b)** receives the previous review alongside the new diff and source tree. For each previous comment in its aspect, it must check whether the issue was addressed: if addressed, omit it; if not or only partially addressed, include it with a short note on what remains. It also surfaces any newly introduced issues.
- **The merge step (7c)** produces a summary section stating how many previous comments were addressed, how many remain unresolved, and how many new issues were found.
- **The verification sub-agent (7e)** runs exactly as in `new`, on the newly generated draft. This includes verifying the "addressed" verdicts — i.e., for each previous comment marked "addressed", the verifier confirms the fix actually landed in the new tree.
- **The finalize step (7f)** writes and symlinks normally. Timestamp is fresh.

### Step 8: Validate comment headings

Same as `new` Step 8. Run both the heading-format check and the
line-in-diff check (`scripts/validate_review.py`) on the generated review
file and fix any failures before reporting to the user.

### Step 9: Report to user

Print:
- Path to the new review file
- Number of previous comments addressed vs. still unresolved
- Number of new issues found
- Total comment count in the new review

Tell the user they can edit the file and then run `/aster-pr-review submit <N>`
when ready. If the review is clean (no unresolved or new issues), suggest
approving the PR on GitHub.

---

## Subcommand: `delete`

Remove all local artifacts for a given PR review: the git worktree,
all review files, and the convenience symlink.

An optional `--yes` flag skips the confirmation prompt.

### Step 1: Resolve the repo root

```bash
REPO_ROOT="$(git rev-parse --show-toplevel)"
```

### Step 2: Ask for confirmation

Unless `--yes` was passed, list what will be deleted and ask the user
to confirm before proceeding:
- The worktree at `$REPO_ROOT/pr_reviews/<N>/` (if it exists)
- The symlink at `$REPO_ROOT/pr_reviews/<N>.md` (if it exists)
- The git ref `refs/pr/<N>` (if it exists)

If the user declines, abort without deleting anything.

### Step 3: Remove the git worktree

If a worktree exists at `$REPO_ROOT/pr_reviews/<N>`, remove it:
```bash
git worktree remove "$REPO_ROOT/pr_reviews/<N>" --force
```

### Step 4: Remove the symlink

```bash
rm -f "$REPO_ROOT/pr_reviews/<N>.md"
```

### Step 5: Clean up the git ref

```bash
git update-ref -d "refs/pr/<N>"
```

### Step 6: Report to user

Print a confirmation that all artifacts for PR #N have been removed.

---

## Review file format

````markdown
---
pr: <number>
repo: <owner/repo>
title: "<PR title>"
head_sha: <full head SHA>
event: comment
---

# PR Description

<Copy the PR description body verbatim from GitHub here.>

---

# Summary

<Write a constructive summary of the PR. Mention what it does well and
what needs attention. This becomes the top-level review body on GitHub.>

---

## `path/to/file.rs` line <N>

> ```diff
> <relevant diff context>
> ```

<Your review comment. Be constructive, specific, and actionable.>

---

(more comments...)
````

**Comment heading formats (these are the ONLY valid formats):**
- `` ## `path/to/file.rs` line 42 `` -- single-line comment (RIGHT side)
- `` ## `path/to/file.rs` lines 40-42 `` -- multi-line comment
- `` ## `path/to/file.rs` line 42 (old) `` -- comment on deleted code (LEFT side)
- `` ## `path/to/file.rs` `` -- file-level comment (no specific line)

**IMPORTANT: The submit parser uses a strict regex to extract the file path
and line numbers from `##` headings. Any heading that does not match one of
the four formats above will be SILENTLY SKIPPED during submission.** Common
mistakes that cause comments to be dropped:
- Prose before the backtick-wrapped path: `` ## Some topic: `file.rs` `` (WRONG)
- Multiple backtick-wrapped paths: `` ## `a.rs` and `b.rs` `` (WRONG)
- Extra text after the line range: `` ## `file.rs` lines 10-20 and 30-40 `` (WRONG)
- Suffixes other than `(old)`: `` ## `file.rs` (module-level) `` (WRONG)
- No backtick-wrapped path at all: `## Some general topic` (WRONG)

For architectural comments that span multiple files or don't target a
specific line, use the **file-level** format: `` ## `path/to/most-relevant-file.rs` ``.
Pick the single most relevant file and put the cross-file context in the
comment body.

**Line-in-diff constraint (often-missed):** an inline comment is only
accepted by GitHub if its line is shown in some hunk of the PR's unified
diff. For *modified* files, that means added lines plus the few context
lines around them — pre-existing code outside any hunk is NOT anchorable
even though it shows up in the worktree. When commenting on pre-existing
code that this PR builds on, use the file-level format. The
`scripts/validate_review.py` helper checks this automatically, and
`submit_review.sh` runs it as a pre-flight check before any submit.

Group headings like `# Design`, `# Correctness`, or `# Craft` (first-level `#`
headings) are allowed for organizing the review for human readability.
They are ignored by the parser (they don't have `##` headings) and will
not be submitted to GitHub. The Summary section is the right place for
high-level themes.

**Rules for comments:**
- The blockquoted diff (`> \`\`\`diff ... \`\`\``) is for human context only
  and will NOT be submitted to GitHub.
- Everything after the blockquoted diff is the comment body that WILL be
  submitted.
- Line numbers refer to the NEW version of the file (RIGHT side) unless
  `(old)` is specified.
- Paths are relative to the repo root.

**Comment body conventions:**

Each comment body must start with a short label followed by a severity tag in parentheses (`blocker`, `significant`, or `minor`). There are two label formats:

1. **Guideline violation** -- cite the guideline as a hyperlink:
   ```
   [**`short-name`**](https://asterinas.github.io/book/to-contribute/coding-guidelines/<path>.html#short-name) (severity): Explanation.
   ```
   To build the URL, find the guideline's relative path in the Index table of `book/src/to-contribute/coding-guidelines/README.md` (e.g., `rust-guidelines/language-items/functions-and-methods.md#small-functions`), prepend `https://asterinas.github.io/book/to-contribute/coding-guidelines/`, and replace `.md` with `.html`. Do not repeat the guideline text -- the hyperlink already provides access.

2. **Non-guideline issue** -- use a bold short description:
   ```
   **Short description** (severity): Explanation.
   ```

**Writing style (MANDATORY for all comments):**

- **No semantic line breaks.** Write each paragraph as a single unwrapped line. Let the reader's markdown viewer soft-wrap it. Do NOT hard-wrap at 72/80/100 columns inside a paragraph. Hard-wrapping makes diffs noisy and makes every minor edit rewrite unrelated lines.
- **Be concise.** State the problem once, directly. Cut throat-clearing ("I noticed that...", "It might be worth considering..."), hedging ("perhaps could maybe..."), and restatements of the diff the reader is already looking at. Target 1-3 short sentences for the problem statement; the fix is the rest.
- **Every comment ends with a concrete fix when one exists.** A fix is a diff, a code snippet, a one-liner command, or a specific API/flag to use. Put it at the end of the body, typically in a fenced code block. Only omit a fix when the right answer genuinely requires design discussion first — and say so explicitly ("No fix suggested: this needs a design decision on X before we know what to change.").
- **One root cause per comment.** If several symptoms share a single underlying bug or design flaw, do NOT file one comment per symptom. Merge them into a single comment that names the root cause once and gives the single fix that resolves all of them. Cross-reference the symptom lines in prose rather than repeating the analysis.

Example of the expected shape:

```markdown
**Race on `inner.blocks_count()` read** (significant): Readers may observe a partially-updated count when the writer is mid-update on another CPU — the `u64` load is not atomic on 32-bit targets.

Fix:

```rust
-let count = self.inner.blocks_count();
+let count = self.inner.blocks_count.load(Ordering::Acquire);
```
```

**Review quality guidelines:**
- Focus on correctness and design, not formatting (unless formatting causes bugs).
- Bugs and design concerns both belong. Style-only nits should usually be omitted.
- When you need to group related issues, merge them (one-root-cause-per-comment) rather than leaving them as separate adjacent comments.

---

## Additional Resources

- For the complete review file format specification, parsing algorithm,
  and GitHub API details, see [spec.md](spec.md).
