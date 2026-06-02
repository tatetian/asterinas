#!/usr/bin/env python3
"""
validate_review.py — validate a review file's comment anchors against a PR diff.

GitHub silently rejects inline review comments anchored to lines that are not
part of the PR's unified diff. For *modified* files (as opposed to *added*),
only the added/changed lines are in the diff; pre-existing context lines are
not. A reviewer who anchors a comment to a pre-existing line will see the
mutation accepted but the thread silently dropped.

This script catches that failure mode before submission:

  - Parses the review file's `## ...` headings to extract (path, line).
  - Parses the PR diff (from `gh pr diff` or a saved file) to compute the set
    of (path, line) pairs that are valid anchors on the RIGHT side of the diff.
  - Reports any heading whose line is not in the valid set, with the nearest
    valid lines as a hint, and suggests either re-anchoring or converting to
    file-level format.

Usage:

    # Validate against a fresh diff fetched from GitHub:
    validate_review.py --review <path/to/review.md> --pr <N> --repo <owner/repo>

    # Validate against a saved diff file:
    validate_review.py --review <path/to/review.md> --diff <path/to/pr.diff>

    # Just dump the valid-lines JSON for a PR (consumed by sub-agents):
    validate_review.py --pr <N> --repo <owner/repo> --emit-valid-lines

Exit status: 0 if all comments are valid (or only file-level), 1 if any
line-anchored comment is outside the diff.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path


HEADING_RE = re.compile(
    r"^##\s+`([^`]+)`"
    r"(?:\s+lines?\s+(\d+)(?:-(\d+))?)?"
    r"(\s+\(old\))?\s*$",
    re.MULTILINE,
)


def parse_diff(diff_text: str) -> dict[str, set[int]]:
    """
    Parse a unified diff and return {path: set(line_numbers_visible_in_RIGHT_side_hunks)}.

    GitHub accepts inline review comments on any RIGHT-side line that appears
    inside a diff hunk — both `+` (added) lines AND ` ` (context) lines around
    them. Removed (`-`) lines have no RIGHT-side equivalent. So we collect
    both `+` and ` ` (and blank) lines and skip `-` lines.

    `b/path` is normalised to `path` so the result can be compared directly to
    review-file paths.
    """
    valid: dict[str, set[int]] = {}
    current_file: str | None = None
    new_line: int = 0

    for raw in diff_text.splitlines():
        if raw.startswith("+++ "):
            # `+++ b/path/to/file` or `+++ /dev/null`
            target = raw[4:]
            if target == "/dev/null":
                current_file = None
            elif target.startswith("b/"):
                current_file = target[2:]
                valid.setdefault(current_file, set())
            else:
                current_file = target
                valid.setdefault(current_file, set())
            new_line = 0
            continue

        if raw.startswith("--- ") or raw.startswith("diff --git"):
            # Old-side header or git's per-file marker — reset, real state is
            # set on +++ above.
            continue

        if raw.startswith("@@"):
            # Hunk header: @@ -OLD,LEN +NEW,LEN @@ optional context
            m = re.search(r"\+(\d+)(?:,(\d+))?", raw)
            if m:
                new_line = int(m.group(1))
            continue

        if current_file is None:
            continue

        if raw.startswith("+") and not raw.startswith("+++"):
            valid[current_file].add(new_line)
            new_line += 1
        elif raw.startswith("-") and not raw.startswith("---"):
            # Old-side line; doesn't advance new-side counter.
            pass
        elif raw.startswith(" ") or raw == "":
            # Context line — visible on the RIGHT side of the diff, so GitHub
            # accepts inline comments on it. Note: a blank context line may
            # arrive with the leading space stripped by some tools.
            valid[current_file].add(new_line)
            new_line += 1
        # Anything else (e.g. "\\ No newline at end of file") is metadata.

    return valid


def fetch_pr_diff(pr: str, repo: str | None) -> str:
    cmd = ["gh", "pr", "diff", pr]
    if repo:
        cmd += ["--repo", repo]
    proc = subprocess.run(cmd, check=True, capture_output=True, text=True)
    return proc.stdout


def parse_review_headings(review_path: Path) -> list[dict]:
    """
    Parse all `## ...` comment headings from the review body (after the
    frontmatter). Returns a list of dicts:
        {path, line_start (int|None), line_end (int|None), is_old (bool)}
    """
    text = review_path.read_text()

    # Skip frontmatter (between the first two --- lines)
    parts = text.split("\n")
    fm = 0
    body_start = 0
    for i, line in enumerate(parts):
        if line.strip() == "---":
            fm += 1
            if fm == 2:
                body_start = i + 1
                break

    body = "\n".join(parts[body_start:])
    headings = []
    for m in HEADING_RE.finditer(body):
        path = m.group(1)
        line_start = int(m.group(2)) if m.group(2) else None
        line_end = int(m.group(3)) if m.group(3) else None
        is_old = m.group(4) is not None
        headings.append(
            {
                "path": path,
                "line_start": line_start,
                "line_end": line_end,
                "is_old": is_old,
            }
        )
    return headings


def coalesce_ranges(lines: set[int]) -> list[list[int]]:
    """Convert a set of line numbers into sorted inclusive [start, end] pairs."""
    if not lines:
        return []
    sorted_lines = sorted(lines)
    ranges: list[list[int]] = []
    start = prev = sorted_lines[0]
    for n in sorted_lines[1:]:
        if n == prev + 1:
            prev = n
        else:
            ranges.append([start, prev])
            start = prev = n
    ranges.append([start, prev])
    return ranges


def nearest(valid_lines: list[int], target: int, k: int = 3) -> list[int]:
    if not valid_lines:
        return []
    return sorted(valid_lines, key=lambda v: (abs(v - target), v))[:k]


def validate(
    headings: list[dict],
    valid: dict[str, set[int]],
) -> tuple[int, list[str]]:
    """
    Returns (error_count, error_messages). An "error" is a line-anchored
    comment whose line is not in the diff. File-level comments are validated
    only against file presence in the diff.
    """
    errors: list[str] = []

    for h in headings:
        path = h["path"]
        if h["is_old"]:
            # We don't validate (old) anchors here — they refer to the LEFT
            # side, and our parser tracks only the RIGHT side. Skip.
            continue

        if path not in valid:
            anchor = (
                "file-level comment"
                if h["line_start"] is None
                else f"line {h['line_start']}"
            )
            errors.append(
                f"`{path}` {anchor}: file is NOT in the PR diff. "
                "Neither re-anchoring within the file nor converting to "
                "file-level will help (GitHub drops both). If the finding is "
                "in-scope, re-anchor to a touched file that interacts with "
                "the issue and describe the untouched-file problem in the "
                "body; otherwise move it into `# Summary` as a related/"
                "out-of-scope note, or drop it."
            )
            continue

        if h["line_start"] is None:
            # File-level on a file in the diff is always fine.
            continue

        valid_for_file = valid[path]
        # Both endpoints of a multi-line range must be in the diff.
        endpoints = [h["line_start"]]
        if h["line_end"] is not None and h["line_end"] != h["line_start"]:
            endpoints.append(h["line_end"])

        bad = [ep for ep in endpoints if ep not in valid_for_file]
        if bad:
            sorted_valid = sorted(valid_for_file)
            hint_target = bad[0]
            hint = nearest(sorted_valid, hint_target)
            hint_str = ", ".join(str(x) for x in hint) if hint else "(none — file has no added lines on the RIGHT side)"
            errors.append(
                f"`{path}` line {h['line_start']}"
                + (f"-{h['line_end']}" if h['line_end'] else "")
                + f": line(s) {bad} not in the PR diff. "
                + f"Nearest valid line(s): {hint_str}. "
                + "Either re-anchor to a line that IS in the diff, or convert to "
                + f"file-level: `## \\`{path}\\``."
            )

    return (len(errors), errors)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--review", type=Path, help="Path to review markdown file.")
    ap.add_argument("--pr", help="PR number (used with --repo, fetches via gh).")
    ap.add_argument("--repo", help="Repo as owner/name (used with --pr).")
    ap.add_argument("--diff", type=Path, help="Path to a saved unified diff file.")
    ap.add_argument(
        "--emit-valid-lines",
        action="store_true",
        help="Print the {path: [valid_lines]} map as JSON and exit. "
        "Useful for passing to sub-agents during review generation.",
    )
    args = ap.parse_args()

    # Source the diff text.
    if args.diff:
        diff_text = args.diff.read_text()
    elif args.pr:
        diff_text = fetch_pr_diff(args.pr, args.repo)
    else:
        ap.error("provide either --diff <path> or --pr <N> [--repo <owner/name>]")

    valid = parse_diff(diff_text)

    if args.emit_valid_lines:
        # Coalesce each file's set of valid line numbers into a list of
        # inclusive [start, end] ranges. Within a hunk the valid RHS lines
        # are always contiguous (the parser advances `new_line` only on `+`
        # and ` `), so ranges represent the data faithfully and an order of
        # magnitude more compactly than listing every line.
        out = {p: coalesce_ranges(lines) for p, lines in valid.items()}
        print(json.dumps(out, indent=2))
        return 0

    if not args.review:
        ap.error("--review <path> is required unless --emit-valid-lines is set")

    headings = parse_review_headings(args.review)
    n_errors, errors = validate(headings, valid)

    if n_errors == 0:
        # Count totals for a friendly summary.
        total = len(headings)
        line_anchored = sum(1 for h in headings if h["line_start"] is not None and not h["is_old"])
        print(
            f"All {total} comment heading(s) reference valid diff anchors "
            f"({line_anchored} line-anchored, {total - line_anchored} file-level/old)."
        )
        return 0

    print(f"{n_errors} comment heading(s) reference lines NOT in the PR diff:")
    for e in errors:
        print(f"  - {e}")
    print()
    print(
        "GitHub will silently drop these comments on submit. Fix the review "
        "file. If the file IS in the PR diff: re-anchor to a line shown in "
        "the diff, or convert to file-level format `## `path`` (drop the "
        "`line N` suffix). If the file is NOT in the PR diff: re-anchor to "
        "a related touched file, or move the observation into `# Summary`."
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
