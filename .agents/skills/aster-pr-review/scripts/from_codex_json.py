#!/usr/bin/env python3
"""
from_codex_json.py — convert Codex-action JSON output to the aster-pr-review
markdown review-file format.

Codex produces a structured JSON document (see .github/codex-output-schema.json
in this repo) with a `findings[]` array and a `summary` string. The skill's
submit pipeline expects a markdown review file with YAML frontmatter, a
`# Summary` section, and `## \`path\` line N` comment headings. This script
bridges the two.

Usage:

    from_codex_json.py \
        --codex-json codex-output.json \
        --pr 123 \
        --repo owner/name \
        --head-sha abc123... \
        --workspace /home/runner/work/asterinas/asterinas \
        [--title "PR title"] \
        [--event comment|approve|request_changes] \
        [--max-findings 50] \
        > review.md

Path normalization: each finding's `absolute_file_path` is converted to a
path relative to `--workspace` (stripping that prefix). Findings whose path
remains absolute (outside the workspace) after this are dropped — they
can't be anchored to a file in the PR. Findings with `./` prefix have the
prefix stripped.

Severity mapping (Codex priority 0..3 → skill severity):
    0 → blocker, 1 → significant, 2-3 → minor.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from validate_review import parse_diff  # noqa: E402


SEVERITY_BY_PRIORITY = {0: "blocker", 1: "significant", 2: "minor", 3: "minor"}


def normalize_path(abs_path: str, workspace: str) -> str | None:
    """Convert a Codex-emitted path to a repo-root-relative path.

    Returns None when the path can't be normalized (absolute path outside
    the workspace, or empty)."""
    if not abs_path:
        return None
    ws = workspace.rstrip("/") + "/"
    if abs_path.startswith(ws):
        return abs_path[len(ws):]
    if abs_path.startswith("./"):
        return abs_path[2:]
    if abs_path.startswith("/"):
        return None
    return abs_path


def heading_for(path: str, start: int, end: int) -> str:
    if start == end:
        return f"## `{path}` line {start}"
    return f"## `{path}` lines {start}-{end}"


def format_comment_body(finding: dict) -> str:
    title = finding.get("title", "").strip() or "Review comment"
    body = finding.get("body", "").strip()
    priority = int(finding.get("priority", 2))
    severity = SEVERITY_BY_PRIORITY.get(priority, "minor")

    return f"**{title}** ({severity}): {body}"


def render(
    codex: dict,
    pr: int,
    repo: str,
    head_sha: str,
    workspace: str,
    title: str | None,
    event: str,
    max_findings: int,
    pr_body: str | None,
    valid_lines: dict[str, set[int]] | None = None,
    attribution: str | None = None,
) -> str:
    summary = (codex.get("summary") or "").strip()
    findings = codex.get("findings") or []
    if max_findings > 0:
        findings = findings[:max_findings]
    if attribution:
        summary = f"{attribution}\n\n{summary}" if summary else attribution

    out: list[str] = []
    out.append("---")
    out.append(f"pr: {pr}")
    out.append(f"repo: {repo}")
    if title is not None:
        safe_title = title.replace('"', '\\"')
        out.append(f'title: "{safe_title}"')
    out.append(f"head_sha: {head_sha}")
    out.append(f"event: {event}")
    out.append("---")
    out.append("")

    if pr_body is not None:
        out.append("# PR Description")
        out.append("")
        out.append(pr_body.rstrip())
        out.append("")
        out.append("---")
        out.append("")

    out.append("# Summary")
    out.append("")
    out.append(summary or "_(no summary provided)_")
    out.append("")

    drop_reasons: list[str] = []
    for f in findings:
        title = f.get("title", "<no title>")
        loc = f.get("code_location") or {}
        abs_path = loc.get("absolute_file_path", "")
        line_range = loc.get("line_range") or {}
        start = line_range.get("start")
        end = line_range.get("end")
        if not isinstance(start, int) or not isinstance(end, int):
            drop_reasons.append(f"{title!r}: missing or invalid line_range")
            continue
        path = normalize_path(abs_path, workspace)
        if path is None:
            drop_reasons.append(f"{title!r}: absolute_file_path is outside the workspace ({abs_path!r})")
            continue
        if valid_lines is not None:
            file_lines = valid_lines.get(path)
            if file_lines is None:
                drop_reasons.append(f"{title!r}: file {path!r} not in the PR diff")
                continue
            if start not in file_lines or end not in file_lines:
                drop_reasons.append(
                    f"{title!r}: line range [{start},{end}] for {path!r} not anchorable in the PR diff"
                )
                continue

        out.append("---")
        out.append("")
        out.append(heading_for(path, start, end))
        out.append("")
        out.append(format_comment_body(f))
        out.append("")

    if drop_reasons:
        # Surface to stderr so the workflow logs show what was dropped and why.
        print(
            f"from_codex_json: dropped {len(drop_reasons)} finding(s):",
            file=sys.stderr,
        )
        for r in drop_reasons:
            print(f"  - {r}", file=sys.stderr)

    return "\n".join(out).rstrip() + "\n"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--codex-json", type=Path, required=True, help="Path to Codex JSON output.")
    ap.add_argument("--pr", type=int, required=True, help="PR number.")
    ap.add_argument("--repo", required=True, help="Repository as owner/name.")
    ap.add_argument("--head-sha", required=True, help="Head commit SHA the review was generated against.")
    ap.add_argument("--workspace", required=True, help="Absolute path to the repo root used by Codex (for stripping the prefix from absolute_file_path).")
    ap.add_argument("--title", help="PR title (optional, for the frontmatter).")
    ap.add_argument(
        "--event",
        default="comment",
        choices=["comment", "approve", "request_changes"],
        help="Review event to record in the frontmatter (used by submit_review.sh --finalize).",
    )
    ap.add_argument("--max-findings", type=int, default=50, help="Truncate the findings list to this many entries (0 to disable).")
    ap.add_argument("--pr-body-file", type=Path, help="Optional path to a file containing the PR description body to embed in the review.")
    ap.add_argument(
        "--diff",
        type=Path,
        help="Optional path to the PR's unified diff. When provided, findings whose line range is not anchorable on the RIGHT side of the diff are dropped (with a reason logged to stderr). Use in CI to avoid emitting a review that fails `validate_review.py`.",
    )
    ap.add_argument(
        "--attribution",
        help="Optional string prepended to the summary section so the rendered review carries a clear bot/source attribution (e.g., '## Reviews by Codex').",
    )
    args = ap.parse_args()

    codex = json.loads(args.codex_json.read_text())
    pr_body = args.pr_body_file.read_text() if args.pr_body_file else None
    valid_lines = parse_diff(args.diff.read_text()) if args.diff else None

    output = render(
        codex=codex,
        pr=args.pr,
        repo=args.repo,
        head_sha=args.head_sha,
        workspace=args.workspace,
        title=args.title,
        event=args.event,
        max_findings=args.max_findings,
        pr_body=pr_body,
        valid_lines=valid_lines,
        attribution=args.attribution,
    )
    sys.stdout.write(output)
    return 0


if __name__ == "__main__":
    sys.exit(main())
