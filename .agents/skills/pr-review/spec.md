# Specification of PR Review Skill

## Overview

A skill called `pr-review` that enables maintainers to generate,
edit, and submit GitHub PR reviews entirely from the CLI. The skill has two
subcommands: `new` (generate) and `submit` (post to GitHub).

The core workflow is: **generate -> edit -> submit**.

## Motivation

- Claude Code outputs review comments as console text, which cannot be posted to
  GitHub without manual copy-paste.
- Maintainers need to edit AI-generated reviews before posting.
- Reviews should be submitted as a single batch (pending review), not as
  individual comments, so the maintainer can finalize on GitHub.

## Workflow

```
/pr-review new <PR>              User edits <N>.md              /pr-review submit <PR>
       |                         (manually or via Claude)                |
       v                                                                v
 1. Fetch PR metadata                                          1. Parse <N>.md
 2. Create worktree with PR source                             2. Validate format
 3. Fetch PR diff                                              3. POST to GitHub API
 4. Generate <N>.md                                               (pending review)
       |                                                                |
       v                                                                v
 pr_reviews/<N>.md                                             Pending review created
                                                               (user finalizes on GitHub)
```

## Directory Structure

```
pr_reviews/
  .gitignore
  2887/               # The source code of the target PR (git worktree)
  2887.md             # The editable review file
```

Each PR review task has an associated worktree directory,
`pr_reviews/<pr_number>`, which contains the source code of the target PR.
Having the whole source tree cloned gives code review more context.

The review file itself is placed at `pr_reviews/<pr_number>.md` (next to
the worktree directory, not inside it) so it is easy to find and open.

The `pr_reviews` directory has a `.gitignore` that instructs Git to ignore
all files under it. This way, adding directories like `2887` would not
affect the working Git project.

## The Review File Format

The file serves two audiences simultaneously:

1. **Human reviewer** -- readable, editable markdown
2. **Submit parser** -- structured enough to extract file paths, line numbers,
   and comment bodies programmatically

### Full Example

````markdown
---
pr: 2887
repo: asterinas/asterinas
title: "Fix Metadata fields and pseudofs DeviceID"
head_sha: 40327e72abc123
event: comment
---

# PR Description

This PR refactors the `Metadata` struct fields with more descriptive
names and fixes the issue that all pseudo filesystems report a
`container_dev_id` of zero.

---

# Summary

This PR improves `Metadata` field naming and fixes all pseudo filesystems
reporting `container_dev_id` as zero. The field renames improve readability.
However, there is a semantic mismatch in `nr_sectors_allocated` that will
cause `st_blocks` to be under-reported for disk-backed filesystems.

---

## `kernel/src/fs/ext2/inode.rs` line 113

> ```diff
> -            blocks: inner.blocks_count() as _,
> +            nr_sectors_allocated: inner.blocks_count() as _,
> ```

[**`accurate-names`**](https://asterinas.github.io/book/to-contribute/coding-guidelines/general-guidelines/index.html#accurate-names): `nr_sectors_allocated` is documented as the number of 512-byte sectors, but
`blocks_count()` returns filesystem block counts (typically 4K blocks). This
will under-report `st_blocks` by a factor of `blk_size / 512` (8x for 4K).

Suggested fix:
```rust
nr_sectors_allocated: (inner.blocks_count() as usize) * (BLOCK_SIZE / 512),
```

---

## `kernel/src/syscall/statx.rs` lines 164-165

> ```diff
> -            stx_mtime: StatxTimestamp::from(info.ctime),
> +            stx_mtime: StatxTimestamp::from(info.last_meta_change_at),
> ```

**Pre-existing bug exposed by rename**: `stx_mtime` should use `info.last_modify_at`. The old code used `ctime`, but the new descriptive names make it obvious.

---

## `kernel/src/fs/sysfs/fs.rs`

**Missing `Drop` impl**: This file allocates a device ID via `DeviceIdAllocator` but, unlike every
other pseudo FS in this PR (`ProcFs`, `DevPts`, `RamFs`, `OverlayFs`,
`PseudoFs`), it has no `Drop` impl to call `release_container_dev_id()`.
````

### Format Specification

#### YAML Frontmatter

Delimited by `---`. Required fields:

| Field      | Type   | Description                                            |
|------------|--------|--------------------------------------------------------|
| `pr`       | int    | PR number                                              |
| `repo`     | string | `owner/repo` slug                                     |
| `title`    | string | PR title (informational, not submitted)                |
| `head_sha` | string | HEAD commit SHA of the PR at generation time           |
| `event`    | string | One of: `comment`, `approve`, `request_changes`        |

The `event` field defaults to `comment`. The user can change it before
submitting. This maps to the GitHub Review API's `event` parameter.

#### PR Description Section

The `# PR Description` heading contains the PR author's description, copied
verbatim from GitHub. It is **not** submitted (the description already
exists on the PR). It serves two purposes:

1. Giving Claude context when generating the review.
2. Letting the human reviewer see what the PR author intended without
   leaving the terminal.

Because it is a regular markdown section (not YAML), it safely contains
arbitrary markdown content (code blocks, horizontal rules, tables, etc.)
without parsing issues.

Everything between `# PR Description` and the next `---` separator is the
description body.

#### Summary Section

The `# Summary` heading contains the top-level review body. This is posted
as the main review comment on GitHub (the text that appears above all inline
comments). It should be constructive and concise.

Everything between `# Summary` and the first `---` separator is the summary
body.

#### Comment Sections

Each comment is a `##` heading followed by a body, separated from the next
comment by `---`.

**Heading formats** (the parsing contract):

| Heading Format                         | Meaning                        | GitHub API Fields                            |
|----------------------------------------|--------------------------------|----------------------------------------------|
| `` ## `path/to/file.rs` line 42 ``    | Single-line comment            | `path`, `line: 42`, `side: RIGHT`            |
| `` ## `path/to/file.rs` lines 40-42 ``| Multi-line comment             | `path`, `start_line: 40`, `line: 42`         |
| `` ## `path/to/file.rs` line 42 (old)``| Comment on deleted code       | `path`, `line: 42`, `side: LEFT`             |
| `` ## `path/to/file.rs` ``            | File-level comment (no line)   | `path`, `subject_type: file`                 |

Rules:
- Each heading must contain exactly **one** backtick-wrapped path. Headings
  with multiple paths (e.g., `` ## `Makefile` / `foo.sh` ``) will not be
  parsed and the comment will be silently skipped.
- The backtick-wrapped path is always relative to the repo root.
- `line` refers to line numbers in the **file** (not diff-relative positions).
  For `RIGHT` side, this is the line number in the new version of the file.
  For `LEFT` side, this is the line number in the old version.
- The `(old)` suffix indicates the comment targets the left side of the diff
  (deleted or modified lines). Without it, the default is `RIGHT` (new code).

**Body structure:**

```
> ```diff
> - old line
> + new line
> ```

Comment text here. This is what gets posted to GitHub.
```

- **Blockquoted diff** (`> \`\`\`diff ... \`\`\``): Optional. Provides context
  for the human reviewer. This is **not** submitted to GitHub -- it is stripped
  during parsing.
- **Comment body**: Everything after the blockquoted diff (or after the heading
  if there is no diff block). This **is** submitted as the GitHub comment body.
  Supports full GitHub-flavored markdown.

**Comment body conventions:**

Each comment body must start with a short label that categorizes the issue.
There are two formats depending on whether the issue corresponds to a
coding guideline:

1. **Guideline violation** -- cite the guideline as a hyperlink:

   ```
   [**`short-name`**](https://url/to/guideline.html#short-name): Explanation of the problem.
   ```

   Example:

   ```
   [**`module-docs`**](https://asterinas.github.io/book/to-contribute/coding-guidelines/rust-guidelines/language-items/comments-and-documentation.html#module-docs): This crate was completely rewritten but the crate-level doc is still a single sentence.
   ```

   To build the URL, take the relative path from the project's Guideline
   Index table (e.g., `rust-guidelines/language-items/functions-and-methods.md#small-functions`)
   and prepend `https://asterinas.github.io/book/to-contribute/coding-guidelines/`.
   Replace `.md` with `.html` in the final URL.

   Multiple guidelines can be cited in the same comment, each on its own
   paragraph. Do not repeat the guideline text -- the hyperlink already
   provides access.

2. **Non-guideline issue** -- use a bold short description:

   ```
   **Short description**: Explanation of the problem.
   ```

   Example:

   ```
   **Hyphen normalization pitfall**: The `name` field stores the registered name verbatim, but `dispatch_params` normalizes hyphens...
   ```

#### Delimiters

Each comment section is separated by a `---` (horizontal rule) on its own line.
The parser splits on these boundaries.

### Parsing Algorithm (for Submit)

```
1. Extract YAML frontmatter (between first pair of `---` lines)
2. Extract summary: text between `# Summary` and first `---` after it
3. For each `---`-delimited section with a `##` heading:
   a. Parse heading: extract path (in backticks), line/lines, (old) suffix
   b. Skip any blockquoted content (lines starting with `> `)
   c. Remaining text = comment body
   d. Skip sections where body is empty after stripping
4. Build API payload and POST
```

## Subcommand: `/pr-review new`

### Input

```
/pr-review new <pr_number_or_url>
```

Accepts:
- A PR number: `2887`
- A full GitHub URL: `https://github.com/asterinas/asterinas/pull/2887`

### Steps

1. **Resolve PR identity.**
   Extract the PR number (and optionally repo) from the input. If only a
   number is given, infer the repo from the current git remote.

2. **Fetch PR metadata.**
   ```bash
   gh pr view <N> --json number,title,headRefOid,baseRefName,headRefName,url,body
   ```

3. **Resolve the repo root.**
   All paths are relative to the repo root. Resolve it once:
   ```bash
   REPO_ROOT="$(git rev-parse --show-toplevel)"
   ```

4. **Create a git worktree with the PR's source.**
   Determine which git remote points to the repo that hosts the PR
   (compare remote URLs against the `owner/repo` from step 1). Typically
   this is `upstream` for upstream PRs or `origin` for fork PRs.
   ```bash
   git fetch <remote> pull/<N>/head:refs/pr/<N>
   git worktree add "$REPO_ROOT/pr_reviews/<N>" refs/pr/<N>
   ```
   This gives Claude the full source tree of the PR at `pr_reviews/<N>/`
   without disturbing the user's working tree. The worktree reuses the
   existing repo's object store, so it is fast and costs no extra disk
   for git objects (only the checked-out files).

5. **Ensure `pr_reviews/.gitignore` exists.**
   Create `$REPO_ROOT/pr_reviews/.gitignore` containing `*` so that the
   entire directory is ignored by the main repo's git. This file is
   created once and persists across PR reviews.

6. **Fetch the PR diff.**
   ```bash
   gh pr diff <N>
   ```

7. **Generate the review file.**
   Claude analyzes the diff and the full source tree in `pr_reviews/<N>/`,
   then writes the structured review file to `$REPO_ROOT/pr_reviews/<N>.md`.

   The review should follow the project's review guidelines (if any exist
   in CLAUDE.md or AGENTS.md). Comments should be:
   - Constructive and specific
   - Focused on correctness, not style (unless style causes bugs)
   - Actionable (suggest fixes, not just point out problems)

8. **Report to user.**
   Print the path to the generated file and a short summary of how many
   comments were generated.

### Idempotency

Running `new` again for the same PR removes the old worktree and recreates
it:
```bash
git worktree remove "$REPO_ROOT/pr_reviews/<N>" --force
git worktree add "$REPO_ROOT/pr_reviews/<N>" refs/pr/<N>
```
The old `pr_reviews/<N>.md` is overwritten. If the user wants to preserve
it, they should copy the file before re-running.

### Cleanup

When a review is no longer needed, the worktree can be removed:
```bash
git worktree remove pr_reviews/<N>
```
This is not done automatically. The user decides when to clean up.

## Subcommand: `/pr-review submit`

### Input

```
/pr-review submit <pr_number>
```

### Steps

1. **Resolve the repo root.**
   ```bash
   REPO_ROOT="$(git rev-parse --show-toplevel)"
   ```

2. **Locate and read** `$REPO_ROOT/pr_reviews/<N>.md`.
   If the file does not exist, report an error.

3. **Parse** the file according to the format specification above.
   Validate:
   - Frontmatter has required fields (`pr`, `repo`, `head_sha`, `event`)
   - At least the summary section exists
   - Each comment section has a valid heading

4. **Check for staleness.**
   Compare `head_sha` from frontmatter against the PR's current HEAD:
   ```bash
   gh pr view <N> --json headRefOid -q .headRefOid
   ```
   - If they match, proceed.
   - If they differ, **refuse to submit**. Print a message explaining that
     the PR has been updated since the review was generated, and instruct
     the user to run `/pr-review new <N>` to regenerate.

   Rationale: posting comments on outdated line numbers is worse than
   re-reviewing. Auto-remapping line numbers across commits is fragile
   (files may be renamed, deleted, or rewritten) and would give false
   confidence.

5. **Map line numbers to diff positions.**
   The REST API for creating reviews requires `position` (a line number
   within the diff hunk), not absolute file line numbers. The submit step
   fetches the diff:
   ```bash
   gh pr diff <N>
   ```
   Then maps each `(file, line, side)` tuple to the corresponding diff
   position. If a line number doesn't appear in the diff, warn the user
   and skip that comment.

6. **Submit via GitHub API.**
   The `scripts/submit_review.sh` script handles submission. It uses a
   two-step approach:

   **Step 6a: Create the review with inline comments (REST API).**
   ```
   POST /repos/{owner}/{repo}/pulls/{pull_number}/reviews
   ```
   ```json
   {
     "commit_id": "sha",
     "body": "Review summary",
     "comments": [
       {
         "path": "file.rs",
         "position": 5,
         "body": "Comment text"
       }
     ]
   }
   ```
   Inline comments use `position` (diff-relative). The review is always
   created as **PENDING** first, regardless of the `event` field in
   frontmatter. The user must explicitly finalize the review on GitHub.

   **Step 6b: Add file-level comments (GraphQL API).**
   The REST create-review endpoint does not support `subject_type: "file"`
   in its comments array. File-level comments are added after the review
   is created, using the GraphQL `addPullRequestReviewThread` mutation:
   ```graphql
   mutation {
     addPullRequestReviewThread(input: {
       pullRequestReviewId: "<node_id>",
       body: "Comment text",
       path: "file.rs",
       subjectType: FILE
     }) { thread { id } }
   }
   ```

7. **Report results.**
   Print:
   - Number of comments submitted
   - Any comments that were skipped (with reasons)
   - A link to the PR for the user to finalize the review

### Error Handling

| Error                                | Behavior                                    |
|--------------------------------------|---------------------------------------------|
| Review file not found               | Print error, suggest running `new` first    |
| Frontmatter missing required fields | Print error, list missing fields            |
| Comment line not found in diff      | Warn, skip that comment, continue           |
| `head_sha` doesn't match current PR | Refuse to submit, instruct user to re-run `new` |
| GitHub API error                     | Print error body, do not retry              |

## Skill Registration

The skill follows the [Agent Skills](https://agentskills.io) open standard.
The standard defines a portable format (`SKILL.md` in a named directory)
that is supported by Claude Code, Codex, Cursor, Gemini CLI, and many
other tools.

### Cross-Tool Compatibility

The Agent Skills standard defines these frontmatter fields:

| Field           | Required | Standard |
|-----------------|----------|----------|
| `name`          | Yes      | Yes      |
| `description`   | Yes      | Yes      |
| `license`       | No       | Yes      |
| `compatibility` | No       | Yes      |
| `metadata`      | No       | Yes      |
| `allowed-tools` | No       | Yes (experimental) |

Tool-specific extensions (ignored by other tools per the standard):

| Field                      | Tool        | Purpose                              |
|----------------------------|-------------|--------------------------------------|
| `argument-hint`            | Claude Code | Autocomplete hint for arguments      |
| `disable-model-invocation` | Claude Code | Prevent auto-invocation              |
| `user-invocable`           | Claude Code | Hide from `/` menu                   |
| `context`                  | Claude Code | Run in forked subagent               |

Codex uses a separate `agents/openai.yaml` file for its extensions
(e.g., `policy.allow_implicit_invocation`).

The standard requires that **unknown frontmatter fields are ignored**,
so including Claude Code-specific fields is safe for Codex (and vice
versa).

**Argument passing** differs between tools:
- Claude Code: `$ARGUMENTS`, `$0`, `$1` substitution in skill content
- Codex: arguments are appended to the prompt by the runtime

The `SKILL.md` body should be written so that it works regardless of how
arguments are injected. The instructions should say "the user provides
a subcommand (`new` or `submit`) and a PR number or URL" rather than
relying solely on `$0`/`$1` substitution. For Claude Code, we can
additionally use `$0`/`$1` for convenience.

### Directory Layout

The canonical location is `.agents/skills/pr-review/`. Both Claude Code
and Codex discover skills from `.agents/skills/` at the repo root.

Claude Code additionally scans `.claude/skills/`. To keep a single source
of truth, `.claude/skills` is a symlink:

```
.agents/
  skills/
    pr-review/
      SKILL.md              # Skill prompt (the source of truth)
      spec.md               # This specification (supporting file)
      scripts/
        submit_review.sh    # Review submission script

.claude/skills -> ../.agents/skills    # Symlink for Claude Code
```

The `name` field in frontmatter must match the directory name (`pr-review`),
as required by the standard.

### SKILL.md Frontmatter

```yaml
---
name: pr-review
description: >
  Generate and submit GitHub PR code reviews. Use when the user wants
  to review a pull request, generate review comments, or submit review
  feedback to GitHub.
# Standard fields
compatibility: Requires gh (GitHub CLI) and git
# Claude Code extensions (ignored by other tools)
argument-hint: <new|submit> <pr_number_or_url>
disable-model-invocation: true
allowed-tools: Bash(gh *), Bash(git *), Read, Write, Glob, Grep, Agent
---
```

Key settings:
- **`description`** (standard): Describes what the skill does and when to
  use it. Must be under 1024 characters.
- **`compatibility`** (standard): Documents that `gh` and `git` are required.
- **`disable-model-invocation`** (Claude Code): Only the user can trigger
  this skill via `/pr-review`. The agent should never auto-invoke a review.
- **`allowed-tools`** (Claude Code): Pre-approves the tools needed so the
  user is not prompted repeatedly during review generation or submission.
- **`argument-hint`** (Claude Code): Shows `<new|submit> <pr_number_or_url>`
  in the autocomplete menu.

### Argument Handling

The skill accepts two arguments: a subcommand and a PR identifier.

In the `SKILL.md` body, instructions are written tool-agnostically:

```
The user provides:
- A subcommand: `new` or `submit`
- A PR number (e.g., 2887) or GitHub URL
```

For Claude Code, the skill content additionally uses `$0` and `$1`:

```
Subcommand: $0
PR: $1
```

Examples of user invocation:
```
/pr-review new 2887
/pr-review new https://github.com/asterinas/asterinas/pull/2887
/pr-review submit 2887
```

### Supporting Files

The `SKILL.md` references this specification (`spec.md`) as a supporting
file for the review file format specification, parsing rules, and API
interaction details. The agent loads it when needed via relative path
reference, as recommended by the standard.

## Appendix: GitHub API Reference

### Create a Pull Request Review (REST)

```
POST /repos/{owner}/{repo}/pulls/{pull_number}/reviews
```

Request body (inline comments only; file-level comments are not supported):
```json
{
  "commit_id": "sha",
  "body": "Review summary",
  "comments": [
    {
      "path": "file.rs",
      "position": 5,
      "body": "Comment text"
    },
    {
      "path": "file.rs",
      "position": 12,
      "body": "Multi-line comment"
    }
  ]
}
```

Note: omitting `event` creates a **PENDING** review.

### Add a File-Level Comment (GraphQL)

The REST create-review endpoint does not support `subject_type: "file"` in
its `DraftPullRequestReviewComment` type. File-level comments must be added
to an existing review via GraphQL:

```graphql
mutation {
  addPullRequestReviewThread(input: {
    pullRequestReviewId: "<review_node_id>",
    body: "File-level comment",
    path: "file.rs",
    subjectType: FILE
  }) { thread { id } }
}
```

### Submit a Pending Review (REST)

```
POST /repos/{owner}/{repo}/pulls/{pull_number}/reviews/{review_id}/events
```
```json
{
  "event": "COMMENT"
}
```

Reference: https://docs.github.com/en/rest/pulls/reviews#create-a-review-for-a-pull-request
