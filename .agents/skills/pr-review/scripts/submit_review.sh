#!/usr/bin/env bash
# submit_review.sh - Parse reviews.md and submit to GitHub as a pending review.
#
# Usage: bash submit_review.sh <path_to_reviews.md>
#
# This script:
# 1. Parses the YAML frontmatter for pr, repo, head_sha, event
# 2. Extracts the Summary section as the review body
# 3. Parses each ## comment section into GitHub API comment objects
# 4. Checks for staleness (head_sha vs current PR HEAD)
# 5. POSTs a pending review via the GitHub API

set -euo pipefail

if [ $# -ne 1 ]; then
    echo "Usage: $0 <path_to_reviews.md>"
    exit 1
fi

REVIEWS_FILE="$1"

if [ ! -f "$REVIEWS_FILE" ]; then
    echo "Error: $REVIEWS_FILE not found."
    echo "Run '/pr-review new <N>' first to generate the review file."
    exit 1
fi

# --- Parse frontmatter ---
# Extract YAML between first pair of --- lines
FRONTMATTER=$(awk '/^---$/{n++; next} n==1{print} n==2{exit}' "$REVIEWS_FILE")

get_field() {
    echo "$FRONTMATTER" | grep "^${1}:" | sed "s/^${1}:[[:space:]]*//" | sed 's/^"\(.*\)"$/\1/' | sed "s/^'\(.*\)'$/\1/"
}

PR_NUMBER=$(get_field "pr")
REPO=$(get_field "repo")
HEAD_SHA=$(get_field "head_sha")
EVENT=$(get_field "event")

# Validate required fields
MISSING=""
[ -z "$PR_NUMBER" ] && MISSING="$MISSING pr"
[ -z "$REPO" ] && MISSING="$MISSING repo"
[ -z "$HEAD_SHA" ] && MISSING="$MISSING head_sha"
if [ -n "$MISSING" ]; then
    echo "Error: Missing required frontmatter fields:$MISSING"
    exit 1
fi

# Default event to comment
EVENT="${EVENT:-comment}"

# Map event to GitHub API event string
case "$EVENT" in
    comment)         API_EVENT="COMMENT" ;;
    approve)         API_EVENT="APPROVE" ;;
    request_changes) API_EVENT="REQUEST_CHANGES" ;;
    *)
        echo "Error: Invalid event '$EVENT'. Must be: comment, approve, request_changes"
        exit 1
        ;;
esac

echo "PR:       #$PR_NUMBER"
echo "Repo:     $REPO"
echo "HEAD SHA: $HEAD_SHA"
echo "Event:    $EVENT -> $API_EVENT"
echo ""

# --- Check staleness ---
CURRENT_SHA=$(gh pr view "$PR_NUMBER" --repo "$REPO" --json headRefOid -q .headRefOid 2>/dev/null || echo "")

if [ -z "$CURRENT_SHA" ]; then
    echo "Warning: Could not fetch current PR HEAD. Proceeding anyway."
elif [ "$CURRENT_SHA" != "$HEAD_SHA" ]; then
    echo "Error: PR has been updated since this review was generated."
    echo "  Review's head_sha: $HEAD_SHA"
    echo "  Current head_sha:  $CURRENT_SHA"
    echo ""
    echo "Run '/pr-review new $PR_NUMBER' to regenerate the review."
    exit 1
else
    echo "Staleness check: OK (head_sha matches)"
fi

# --- Extract summary ---
# Everything between "# Summary" and the next "---"
SUMMARY=$(awk '
    /^# Summary[[:space:]]*$/ { capture=1; next }
    capture && /^---[[:space:]]*$/ { exit }
    capture { print }
' "$REVIEWS_FILE" | sed -e 's/^[[:space:]]*//' -e '/./,$!d' | sed -e :a -e '/^[[:space:]]*$/{ $d; N; ba; }')

if [ -z "$SUMMARY" ]; then
    echo "Warning: No summary section found. Using empty review body."
    SUMMARY=""
fi

# --- Parse comment sections ---
# We use Python for reliable markdown parsing since the format is non-trivial.
COMMENTS_JSON=$(REVIEWS_FILE="$REVIEWS_FILE" python3 << 'PYEOF'
import re
import json
import os

reviews_file = os.environ["REVIEWS_FILE"]

with open(reviews_file, 'r') as f:
    content = f.read()

# Split into sections by ---
# First, skip the frontmatter (between first two ---)
parts = content.split('\n')
in_frontmatter = False
frontmatter_count = 0
body_start = 0
for i, line in enumerate(parts):
    if line.strip() == '---':
        frontmatter_count += 1
        if frontmatter_count == 2:
            body_start = i + 1
            break

body = '\n'.join(parts[body_start:])

# Split body into sections by --- (horizontal rules)
sections = re.split(r'\n---\s*\n', body)

# Heading pattern: ## `path/to/file` line 42
# or: ## `path/to/file` lines 40-42
# or: ## `path/to/file` line 42 (old)
# or: ## `path/to/file`
heading_re = re.compile(
    r'^##\s+`([^`]+)`'           # path in backticks
    r'(?:\s+lines?\s+'           # optional line/lines keyword
    r'(\d+)(?:-(\d+))?'          # line number, optional range end
    r')?'
    r'(\s+\(old\))?'             # optional (old) suffix
    r'\s*$',
    re.MULTILINE
)

comments = []
skipped = 0

for section in sections:
    section = section.strip()
    if not section:
        continue

    match = heading_re.search(section)
    if not match:
        continue

    path = match.group(1)
    line_start = match.group(2)
    line_end = match.group(3)
    is_old = match.group(4) is not None

    # Extract body: everything after the heading, skipping blockquoted lines
    after_heading = section[match.end():].strip()
    lines = after_heading.split('\n')

    # Skip blockquoted diff blocks (lines starting with >)
    body_lines = []
    in_blockquote = False
    for line in lines:
        stripped = line.strip()
        if stripped.startswith('> '):
            in_blockquote = True
            continue
        if in_blockquote and stripped == '>':
            continue
        if in_blockquote and not stripped.startswith('>'):
            in_blockquote = False
        if not stripped.startswith('>'):
            body_lines.append(line)

    body_text = '\n'.join(body_lines).strip()

    if not body_text:
        skipped += 1
        continue

    comment = {"path": path, "body": body_text}

    if line_start:
        comment["line"] = int(line_end) if line_end else int(line_start)
        comment["side"] = "LEFT" if is_old else "RIGHT"

        if line_end and line_start != line_end:
            comment["start_line"] = int(line_start)
            comment["start_side"] = "LEFT" if is_old else "RIGHT"
    else:
        comment["subject_type"] = "file"

    comments.append(comment)

output = {"comments": comments, "skipped": skipped}
print(json.dumps(output))
PYEOF
)

NUM_COMMENTS=$(echo "$COMMENTS_JSON" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d['comments']))")
NUM_SKIPPED=$(echo "$COMMENTS_JSON" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['skipped'])")
COMMENTS_ARRAY=$(echo "$COMMENTS_JSON" | python3 -c "import sys,json; d=json.load(sys.stdin); print(json.dumps(d['comments']))")

echo ""
echo "Parsed $NUM_COMMENTS comment(s), skipped $NUM_SKIPPED empty comment(s)."

if [ "$NUM_COMMENTS" -eq 0 ] && [ -z "$SUMMARY" ]; then
    echo "Nothing to submit (no summary, no comments)."
    exit 0
fi

# --- Submit review ---
echo ""
echo "Submitting review to GitHub..."

# Build the gh api command
# We use --input to pass JSON via stdin to avoid shell escaping issues.
# Pass all values via environment variables to avoid heredoc quoting issues.
# Uses the newer line-based API format (line/side/start_line/start_side)
# instead of the legacy position-based format.
REVIEW_JSON=$(PR_SUMMARY="$SUMMARY" PR_COMMENTS="$COMMENTS_ARRAY" PR_HEAD_SHA="$HEAD_SHA" python3 << 'PYEOF2'
import json, os

comments = json.loads(os.environ["PR_COMMENTS"])

# Process comments: use line-based format, handle file-level comments
api_comments = []
for c in comments:
    path = c["path"]

    if "subject_type" in c and c["subject_type"] == "file":
        # File-level comments are not supported by the REST create-review
        # endpoint. They will be added via GraphQL after the review is created.
        continue

    line = c.get("line")
    side = c.get("side", "RIGHT")

    if line:
        api_comment = {
            "path": path,
            "line": line,
            "side": side,
            "body": c["body"]
        }

        if "start_line" in c:
            api_comment["start_line"] = c["start_line"]
            api_comment["start_side"] = c.get("start_side", side)

        api_comments.append(api_comment)

payload = {
    "commit_id": os.environ["PR_HEAD_SHA"],
    "body": os.environ["PR_SUMMARY"],
    "comments": api_comments
}

# Note: omitting "event" creates a PENDING review

file_level = [c for c in comments if c.get("subject_type") == "file"]

result = {
    "payload": payload,
    "skipped": [],
    "file_level": file_level
}
print(json.dumps(result))
PYEOF2
)

# Extract payload, skipped info, and file-level merge count from the result
PAYLOAD=$(echo "$REVIEW_JSON" | python3 -c "import sys,json; print(json.dumps(json.load(sys.stdin)['payload']))")
NUM_API_COMMENTS=$(echo "$PAYLOAD" | python3 -c "import sys,json; print(len(json.load(sys.stdin).get('comments', [])))")
DIFF_SKIPPED=$(echo "$REVIEW_JSON" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d.get('skipped', [])))")
FILE_LEVEL_JSON=$(echo "$REVIEW_JSON" | python3 -c "import sys,json; print(json.dumps(json.load(sys.stdin).get('file_level', [])))")
NUM_FILE_LEVEL=$(echo "$FILE_LEVEL_JSON" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))")

echo "  Inline comments:    $NUM_API_COMMENTS"
[ "$NUM_FILE_LEVEL" -gt 0 ] && echo "  File-level comments: $NUM_FILE_LEVEL (added via GraphQL after review creation)"
[ "$NUM_SKIPPED" -gt 0 ] && echo "  Empty (skipped):     $NUM_SKIPPED"
[ "$DIFF_SKIPPED" -gt 0 ] && echo "  Not in diff (skipped): $DIFF_SKIPPED"

# Report any skipped comments
if [ "$DIFF_SKIPPED" -gt 0 ]; then
    echo ""
    echo "Skipped comments (line not found in diff):"
    echo "$REVIEW_JSON" | python3 -c "
import sys, json
d = json.load(sys.stdin)
for s in d.get('skipped', []):
    line = s.get('line', '?')
    print(f\"  - {s['path']} line {line}: {s['reason']}\")
"
fi

# --- Check for existing pending review ---
EXISTING_REVIEW_ID=$(gh api "repos/$REPO/pulls/$PR_NUMBER/reviews" \
    --jq '.[] | select(.state == "PENDING") | .id' 2>/dev/null | head -n1 || echo "")

if [ -n "$EXISTING_REVIEW_ID" ]; then
    echo ""
    echo "Found existing pending review (ID: $EXISTING_REVIEW_ID). Appending to it."

    # Get the GraphQL node_id for the existing review (required by GraphQL mutations)
    REVIEW_NODE_ID=$(gh api "repos/$REPO/pulls/$PR_NUMBER/reviews/$EXISTING_REVIEW_ID" \
        --jq '.node_id' 2>/dev/null || echo "")

    if [ -z "$REVIEW_NODE_ID" ]; then
        echo "Error: Could not fetch node_id for existing review."
        exit 1
    fi

    # Update the review body (summary) via GraphQL if we have a non-empty summary
    if [ -n "$SUMMARY" ]; then
        ESCAPED_SUMMARY=$(python3 -c "import json,sys; print(json.dumps(sys.argv[1]))" "$SUMMARY")
        UPDATE_RESP=$(gh api graphql -f query="mutation {
            updatePullRequestReview(input: {
                pullRequestReviewId: \"$REVIEW_NODE_ID\",
                body: $ESCAPED_SUMMARY
            }) { pullRequestReview { id } }
        }" 2>&1) || {
            echo "  Note: Could not update review body (GitHub does not allow updating"
            echo "  a review that was created without a body). You can add the summary"
            echo "  manually when finalizing the review on GitHub."
        }
    fi

    # Add each comment to the existing review via GraphQL addPullRequestReviewThread.
    # Use the pre-position-mapping comments (COMMENTS_ARRAY from the parser) which
    # have line/side fields instead of diff positions.
    COMMENTS_TMPFILE=$(mktemp)
    echo "$COMMENTS_ARRAY" | python3 -c "
import sys, json
comments = json.load(sys.stdin)
for c in comments:
    print(json.dumps(c))
" > "$COMMENTS_TMPFILE"

    ADDED=0
    FAILED=0
    while IFS= read -r COMMENT_JSON; do
        # Build the GraphQL mutation for this comment
        GQL_RESULT=$(echo "$COMMENT_JSON" | REVIEW_NODE_ID="$REVIEW_NODE_ID" python3 -c "
import sys, json, os

c = json.load(sys.stdin)
review_id = os.environ['REVIEW_NODE_ID']
path = c['path']
body = json.dumps(c['body'])

# File-level comments: use subject_type FILE (no line)
if c.get('subject_type') == 'file':
    mutation = '''mutation {
        addPullRequestReviewThread(input: {
            pullRequestReviewId: \"%s\",
            body: %s,
            path: \"%s\",
            subjectType: FILE
        }) { thread { id } }
    }''' % (review_id, body, path)
else:
    line = c.get('line', 1)
    side = c.get('side', 'RIGHT')
    start_line = c.get('start_line')
    start_side = c.get('start_side')

    if start_line and start_line != line:
        mutation = '''mutation {
            addPullRequestReviewThread(input: {
                pullRequestReviewId: \"%s\",
                body: %s,
                path: \"%s\",
                line: %d,
                side: %s,
                startLine: %d,
                startSide: %s
            }) { thread { id } }
        }''' % (review_id, body, path, line, side, start_line, start_side)
    else:
        mutation = '''mutation {
            addPullRequestReviewThread(input: {
                pullRequestReviewId: \"%s\",
                body: %s,
                path: \"%s\",
                line: %d,
                side: %s
            }) { thread { id } }
        }''' % (review_id, body, path, line, side)

print(mutation)
" 2>&1) || {
            FAILED=$((FAILED + 1))
            COMMENT_PATH=$(echo "$COMMENT_JSON" | python3 -c "import sys,json; print(json.load(sys.stdin).get('path','?'))")
            echo "  Warning: Failed to build mutation for $COMMENT_PATH"
            continue
        }

        RESP=$(gh api graphql -f query="$GQL_RESULT" 2>&1) && {
            ADDED=$((ADDED + 1))
        } || {
            FAILED=$((FAILED + 1))
            COMMENT_PATH=$(echo "$COMMENT_JSON" | python3 -c "import sys,json; print(json.load(sys.stdin).get('path','?'))")
            echo "  Warning: Failed to add comment on $COMMENT_PATH: $RESP"
        }
    done < "$COMMENTS_TMPFILE"
    rm -f "$COMMENTS_TMPFILE"

    echo "  Added $ADDED comment(s) to existing review."
    [ "$FAILED" -gt 0 ] && echo "  Failed to add $FAILED comment(s)."

    REVIEW_ID="$EXISTING_REVIEW_ID"
else
    # No existing pending review -- create a new one via REST API
    RESPONSE=$(echo "$PAYLOAD" | gh api \
        "repos/$REPO/pulls/$PR_NUMBER/reviews" \
        --method POST \
        --input - 2>&1) || {
        echo ""
        echo "Error: GitHub API call failed."
        echo "$RESPONSE"
        exit 1
    }

    REVIEW_ID=$(echo "$RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin).get('id', 'unknown'))" 2>/dev/null || echo "unknown")
fi

# --- Add file-level comments via GraphQL ---
# The REST create-review endpoint does not support subject_type: "file",
# so file-level comments are added after the review exists.
if [ "$NUM_FILE_LEVEL" -gt 0 ]; then
    # Get the GraphQL node_id for the review
    REVIEW_NODE_ID=$(gh api "repos/$REPO/pulls/$PR_NUMBER/reviews/$REVIEW_ID" \
        --jq '.node_id' 2>/dev/null || echo "")

    if [ -z "$REVIEW_NODE_ID" ]; then
        echo "Warning: Could not fetch node_id for review. File-level comments not added."
    else
        FL_ADDED=0
        FL_FAILED=0
        echo "$FILE_LEVEL_JSON" | python3 -c "
import sys, json
for c in json.load(sys.stdin):
    print(json.dumps(c))
" | while IFS= read -r FL_COMMENT; do
            GQL_MUTATION=$(echo "$FL_COMMENT" | REVIEW_NODE_ID="$REVIEW_NODE_ID" python3 -c "
import sys, json, os
c = json.load(sys.stdin)
review_id = os.environ['REVIEW_NODE_ID']
body = json.dumps(c['body'])
path = c['path']
print('mutation { addPullRequestReviewThread(input: { pullRequestReviewId: \"%s\", body: %s, path: \"%s\", subjectType: FILE }) { thread { id } } }' % (review_id, body, path))
")
            gh api graphql -f query="$GQL_MUTATION" > /dev/null 2>&1 && {
                echo "  Added file-level comment on ${FL_COMMENT}" | python3 -c "import sys,json; c=json.load(sys.stdin.readline().split('on ',1)[1] if False else sys.stdin); print()" 2>/dev/null || true
            } || {
                COMMENT_PATH=$(echo "$FL_COMMENT" | python3 -c "import sys,json; print(json.load(sys.stdin).get('path','?'))")
                echo "  Warning: Failed to add file-level comment on $COMMENT_PATH"
            }
        done
    fi
fi

echo ""
echo "Review submitted successfully (review ID: $REVIEW_ID)"
echo "  State: PENDING"
echo ""
echo "Go to GitHub to finalize the review:"
echo "  https://github.com/$REPO/pull/$PR_NUMBER"
echo ""
echo "The review is in PENDING state. On GitHub you can:"
echo "  - Add more comments"
echo "  - Edit existing comments"
echo "  - Submit as: Comment, Approve, or Request Changes"
