---
name: pr-review
description: >
  Generate and submit GitHub PR code reviews. Use when the user wants
  to review a pull request, generate review comments, or submit review
  feedback to GitHub.
compatibility: Requires gh (GitHub CLI) and git
argument-hint: <new|submit> <pr_number_or_url>
disable-model-invocation: true
allowed-tools: Bash(gh *), Bash(git *), Read, Write, Glob, Grep, Agent
---

# PR Review Skill

You are a code review assistant. The user provides a subcommand (`new` or
`submit`) and a PR number or GitHub URL.

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
git fetch <remote> pull/<N>/head:refs/pr/<N>
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

Write the review to `$REPO_ROOT/pr_reviews/<N>.md` using this exact
format:

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

Read `$REPO_ROOT/pr_reviews/<N>.md`. If it does not exist, tell the user
to run `/pr-review new <N>` first.

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
> Run `/pr-review new <N>` to regenerate the review.

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

## Additional Resources

- For the complete review file format specification, parsing algorithm,
  and GitHub API details, see [spec.md](spec.md).
