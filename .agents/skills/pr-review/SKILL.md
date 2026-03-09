---
name: pr-review
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

### Step 6: Fetch the PR diff

```bash
gh pr diff <N>
```

### Step 7: Generate the review file

Read the diff carefully and explore the source tree in
`$REPO_ROOT/pr_reviews/<N>/` for full context. Use Agent subagents to
parallelize reading of changed files when there are many.

Follow the project's review guidelines if they exist in CLAUDE.md or
AGENTS.md.

Generate a timestamp for the review file name:
```bash
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
```

Write the review to `$REPO_ROOT/pr_reviews/<N>/review-<TIMESTAMP>.md`
using the format documented below in "Review file format".

Then create (or update) a symlink for convenient access:
```bash
ln -sf "<N>/review-<TIMESTAMP>.md" "$REPO_ROOT/pr_reviews/<N>.md"
```

The symlink uses a relative target so it works regardless of where
the repo is cloned.

### Step 8: Report to user

Print the path to the generated file and how many comments were generated.
Tell the user they can edit the file and then run `/pr-review submit <N>`
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
`/pr-review new <N>` first.

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
> Run `/pr-review redo <N>` to generate a follow-up review.

### Step 5: Submit via GitHub API

Run the `scripts/submit_review.sh` script that is bundled with this skill:

```bash
bash ${CLAUDE_SKILL_DIR}/scripts/submit_review.sh "$REPO_ROOT/pr_reviews/<N>.md"
```

This script parses the review file and calls the GitHub API to create a
pending review.

### Step 6: Report results

Print the output from the submit script, which includes:
- Number of comments submitted
- Any comments that were skipped (with reasons)
- A link to the PR

Tell the user the review is in PENDING state and they should go to GitHub
to finalize it.

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

### Step 6: Fetch the new PR diff

```bash
gh pr diff <N>
```

### Step 7: Generate the follow-up review

Read the new diff and explore the updated source tree in
`$REPO_ROOT/pr_reviews/<N>/` for full context.

Using the previous review as context, generate a follow-up review that:

1. **Verifies each previous comment.** For every comment in the previous
   review, check whether the issue has been addressed in the new revision.
   - If **addressed**, do not include it in the new review.
   - If **not addressed** or only **partially addressed**, include it
     again with a note explaining what remains unresolved.
2. **Identifies new issues.** Review the new diff for any issues
   introduced by the latest changes that were not present before.
3. **Writes the summary.** The summary should note how many previous
   comments were addressed, how many remain, and any new issues found.

Generate a timestamp and write the review file:
```bash
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
```

Write to `$REPO_ROOT/pr_reviews/<N>/review-<TIMESTAMP>.md` using the
same format documented below in "Review file format".

Update the symlink:
```bash
ln -sf "<N>/review-<TIMESTAMP>.md" "$REPO_ROOT/pr_reviews/<N>.md"
```

### Step 8: Report to user

Print:
- Path to the new review file
- Number of previous comments addressed vs. still unresolved
- Number of new issues found
- Total comment count in the new review

Tell the user they can edit the file and then run `/pr-review submit <N>`
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

**Comment heading formats:**
- `` ## `path/to/file.rs` line 42 `` -- single-line comment (RIGHT side)
- `` ## `path/to/file.rs` lines 40-42 `` -- multi-line comment
- `` ## `path/to/file.rs` line 42 (old) `` -- comment on deleted code (LEFT side)
- `` ## `path/to/file.rs` `` -- file-level comment (no specific line)

**Rules for comments:**
- The blockquoted diff (`> \`\`\`diff ... \`\`\``) is for human context only
  and will NOT be submitted to GitHub.
- Everything after the blockquoted diff is the comment body that WILL be
  submitted.
- Line numbers refer to the NEW version of the file (RIGHT side) unless
  `(old)` is specified.
- Paths are relative to the repo root.

**Comment body conventions:**

Each comment body must start with a short label. There are two formats:

1. **Guideline violation** -- cite the guideline as a hyperlink:
   ```
   [**`short-name`**](https://asterinas.github.io/book/to-contribute/coding-guidelines/<path>.html#short-name): Explanation.
   ```
   To build the URL, find the guideline's relative path in the Index table
   of `book/src/to-contribute/coding-guidelines/README.md` (e.g.,
   `rust-guidelines/language-items/functions-and-methods.md#small-functions`),
   prepend `https://asterinas.github.io/book/to-contribute/coding-guidelines/`,
   and replace `.md` with `.html`. Do not repeat the guideline text -- the
   hyperlink already provides access.

2. **Non-guideline issue** -- use a bold short description:
   ```
   **Short description**: Explanation.
   ```

**Review quality guidelines:**
- Be constructive and specific.
- Focus on correctness, not style (unless style causes bugs).
- Suggest fixes, not just problems.
- Group related issues rather than commenting on every line.
- Highlight both bugs and design concerns.

---

## Additional Resources

- For the complete review file format specification, parsing algorithm,
  and GitHub API details, see [spec.md](spec.md).
