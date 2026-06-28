#!/usr/bin/env bash
#
# run.sh — benchmark harness for aster-code-review.
# Reads `benchmark/problems.yaml`.
#
# The agent that BOTH reviews and grades is chosen by AGENT_PROFILE (required)
# — a JSON manifest under agent_profiles/ describing how to launch a headless agent.
# The harness itself names no agent (see spec/benchmark.md, "Agent profiles").
#
# For each problem it reconstructs the snapshot in a scratch worktree,
# runs the skill, and grades recall.
# Two review modes (problems.yaml `review_mode`):
#   diff  — fetch the commit by full SHA from its `remote` if absent,
#           worktree at it (detached);
#           review `diff <commit>^`.
#   files — worktree at base_commit;
#           review `files <targets>`.
# Each recall problem is reviewed CHEAP first (--per-persona-context=no);
# only on a miss does it escalate to the fan-out (=yes).
#
# INTEGRITY — the review agent must never see the answers:
#   * `defects`/`source` and the descriptive problem_id are ground truth for the
#     GRADER only; they never reach the reviewer.
#   * the scratch worktree path is OPAQUE (wt<N>), never the slug.
#   * overlay-skill.sh overlays the current skill but EXCLUDES benchmark/.
#
# Knobs (env vars):
#   AGENT_PROFILE    REQUIRED. a profile NAME -> agent_profiles/<name>/, or a dir path.
#   PROFILE_VARIANT  `smoke` merges the `.smoke` overlay over the base profile; unset = base.
#   MIN_RECALL       recall% gate, 0..100 (default 100).
#                    MIN_RECALL=0 is a SMOKE run:
#                    reviews only — NO grading, escalation,
#                    or precision — so it is fast and answers just "does the skill run here?".
#                    A smoke passes iff every selected problem's reviewer wrote a non-empty review;
#                    a run that errors or writes no review fails.
#                    MIN_RECALL>0 grades and gates on recall.
#   PROBLEMS         space-separated selectors;
#                    a token matches by id prefix (e.g. "0002" -> 0002-fair-weight-race).
#                    Empty/unset = all.
#   KEEP_REVIEWS     keep each problem's produced review + expected defects for inspection:
#                    a <dir>, or `1` for a temp dir (path printed).
#                    Copied post-review, so no answer key ever leaks into a review.
#   WORK             scratch dir (default: a mktemp dir, removed on exit)
#   REVIEW_CMD / GRADE_CMD / NEG_GRADE_CMD   override the agent calls (CI / mocking)
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(git -C "$HERE" rev-parse --show-toplevel)"

# --- agent profile (required) -----------------------------------------------
# AGENT_PROFILE names a directory under agent_profiles/ (e.g. AGENT_PROFILE=codex -> agent_profiles/codex/).
# It holds profile.json (command/env/inherit) and,
# by convention, an optional config.toml seeded into a private {workdir}.
# When PROFILE_VARIANT=smoke,
# the `.smoke` overlay (profile.smoke.json / config.smoke.toml) is shallow-merged over the base
# — a smoke key wins.
# All agent specifics live in the profile; run_agent below is generic.
profiles_dir="$HERE/agent_profiles"
list_profiles() { find "$profiles_dir" -mindepth 2 -maxdepth 2 -name profile.json -printf '%h\n' 2>/dev/null | xargs -r -n1 basename | sort | tr '\n' ' '; }
[[ -n "${AGENT_PROFILE:-}" ]] || {
    echo "run.sh: AGENT_PROFILE is required (e.g. AGENT_PROFILE=codex). Available: $(list_profiles)" >&2; exit 2; }
if [[ "$AGENT_PROFILE" == */* ]]; then PROFILE_DIR="$AGENT_PROFILE"; else PROFILE_DIR="$profiles_dir/$AGENT_PROFILE"; fi
[[ -f "$PROFILE_DIR/profile.json" ]] || { echo "run.sh: profile not found: $PROFILE_DIR/profile.json (available: $(list_profiles))" >&2; exit 2; }
PROFILE_DIR="$(cd "$PROFILE_DIR" && pwd)"
PROFILE_SMOKE=0; [[ "${PROFILE_VARIANT:-}" == smoke ]] && PROFILE_SMOKE=1
PROFILE_WORKDIR="$(mktemp -d)"          # the {workdir}: config.toml + auth land here (e.g. CODEX_HOME)
declare -a PROFILE_CMD=() PROFILE_ENV=() INH_SRC=() INH_DEST=()
# Parse the (smoke-merged) profile.json into C<TAB>token | E<TAB>KEY=VAL | I<TAB>src<TAB>dest,
# and seed the (smoke-merged) config.toml into {workdir}. {workdir}/{home} are resolved now (static);
# {prompt} is left for run_agent.
profile_parsed="$(python3 - "$PROFILE_DIR" "$PROFILE_WORKDIR" "$HOME" "$PROFILE_SMOKE" <<'PY'
import json, os, sys
pdir, workdir, home, smoke = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4] == "1"
def load_json(p):
    if not os.path.exists(p): return {}
    try: return json.load(open(p))
    except Exception as e: sys.stderr.write(f"invalid JSON {p}: {e}\n"); sys.exit(3)
prof = load_json(os.path.join(pdir, "profile.json"))
if smoke: prof.update(load_json(os.path.join(pdir, "profile.smoke.json")))   # shallow: a smoke key wins
cmd = prof.get("command")
if not isinstance(cmd, list) or not cmd or not all(isinstance(x, str) for x in cmd):
    sys.stderr.write("profile 'command' must be a non-empty array of strings\n"); sys.exit(3)
def sub(s): return str(s).replace("{workdir}", workdir).replace("{home}", home)
for t in cmd:                                          print("C\t" + sub(t))
for k, v in (prof.get("env") or {}).items():           print("E\t" + f"{k}={sub(v)}")
for src, dest in (prof.get("inherit") or {}).items():  print("I\t" + sub(str(src)) + "\t" + sub(dest))
# config convention: seed config.toml into {workdir} if present,
# shallow-merging the smoke overlay (flat top-level `key = value` — our config has no tables; a smoke key wins).
def toml_flat(p):
    d = {}
    if os.path.exists(p):
        for ln in open(p):
            s = ln.strip()
            if s and not s.startswith(("#", "[")) and "=" in s:
                k, v = s.split("=", 1); d[k.strip()] = v.strip()
    return d
cfg = toml_flat(os.path.join(pdir, "config.toml"))
if cfg:
    if smoke: cfg.update(toml_flat(os.path.join(pdir, "config.smoke.toml")))
    with open(os.path.join(workdir, "config.toml"), "w") as f:
        for k, v in cfg.items(): f.write(f"{k} = {v}\n")
PY
)" || { echo "run.sh: invalid AGENT_PROFILE: $PROFILE_DIR" >&2; exit 2; }
while IFS=$'\t' read -r tag a b; do
    case "$tag" in
        C) PROFILE_CMD+=("$a") ;;
        E) PROFILE_ENV+=("$a") ;;
        I) INH_SRC+=("$a"); INH_DEST+=("$b") ;;
    esac
done <<<"$profile_parsed"
for i in "${!INH_SRC[@]}"; do                     # inherit outside files (e.g. the agent's real auth)
    src="${INH_SRC[$i]}"; dest="$PROFILE_WORKDIR/${INH_DEST[$i]}"
    [[ -f "$src" ]] || { echo "run.sh: profile 'inherit' source not found: $src (is the agent logged in?)" >&2; exit 2; }
    mkdir -p "$(dirname "$dest")"; cp "$src" "$dest"
done

# run_agent <prompt> — launch the profile's agent with {prompt} substituted,
# with NO shell (so a prompt full of backticks/quotes/newlines is safe).
# Runs in the current cwd and inherits the current env PLUS the profile env.
run_agent() {
    local prompt="$1" tok; local -a argv=()
    for tok in "${PROFILE_CMD[@]}"; do argv+=("${tok//\{prompt\}/$prompt}"); done
    # stdin from /dev/null: the prompt is an argv token,
    # and a headless agent that also reads stdin (e.g. `codex exec` appends piped stdin as a <stdin> block) must NOT swallow the caller's stdin
    # — which carries the problem list (below).
    if [[ ${#PROFILE_ENV[@]} -gt 0 ]]; then env "${PROFILE_ENV[@]}" "${argv[@]}" </dev/null; else "${argv[@]}" </dev/null; fi
}

# --- scratch dirs ------------------------------------------------------------
WORK_IS_TEMP=0
if [[ -z "${WORK:-}" ]]; then WORK="$(mktemp -d)"; WORK_IS_TEMP=1; fi
mkdir -p "$WORK"
# Ground truth and the guideline tree live OUTSIDE the worktree parent,
# so a review can never reach them by walking up from its own worktree.
SPEC="$(mktemp -d)"           # per-problem expected defects (grader only)
GROOT="$(mktemp -d)"          # guidelines-only tree: current pages, no answer key
mkdir -p "$GROOT/book/src/to-contribute"
cp -r "$REPO/book/src/to-contribute/coding-guidelines" "$GROOT/book/src/to-contribute/" 2>/dev/null || true

# Optional inspection: keep each problem's produced review alongside its expected defects,
# labelled by problem_id, so a user can eyeball how the skill did instead of trusting the score.
# KEEP_REVIEWS=<dir> collects them there; KEEP_REVIEWS=1 uses a temp dir and prints it.
# Copies happen POST-review/grade,
# so nothing leaks the answer key into a review;
# the copies live outside $WORK and survive cleanup.
KEEP_DIR=""
if [[ -n "${KEEP_REVIEWS:-}" ]]; then
    if [[ "$KEEP_REVIEWS" == 1 ]]; then KEEP_DIR="$(mktemp -d)"; else KEEP_DIR="$KEEP_REVIEWS"; mkdir -p "$KEEP_DIR"; fi
fi
keep_reviews() { # <id> <wt> — copy any produced review + expected defects into KEEP_DIR/<id>/
    [[ -n "$KEEP_DIR" ]] || return 0
    local id="$1" wt="$2" dst="$KEEP_DIR/$id"; mkdir -p "$dst"
    [[ -s "$wt.off.md" ]]    && cp "$wt.off.md"    "$dst/review.md"          # combined / smoke review
    [[ -s "$wt.on.md" ]]     && cp "$wt.on.md"     "$dst/review-fanout.md"   # escalated fan-out review
    [[ -s "$wt.review.md" ]] && cp "$wt.review.md" "$dst/review.md"          # precision-problem review
    [[ -f "$SPEC/$id.defects.txt" ]]   && cp "$SPEC/$id.defects.txt"   "$dst/expected-defects.txt"
    [[ -f "$SPEC/$id.negatives.txt" ]] && cp "$SPEC/$id.negatives.txt" "$dst/expected-negatives.txt"
}

cleanup() {
    while IFS= read -r wt; do
        [[ "$wt" == "$WORK"/* ]] && git -C "$REPO" worktree remove --force "$wt" 2>/dev/null || true
    done < <(git -C "$REPO" worktree list --porcelain | awk '/^worktree /{print $2}')
    git -C "$REPO" worktree prune 2>/dev/null || true
    rm -rf "$SPEC" "$GROOT" "$PROFILE_WORKDIR"
    [[ "$WORK_IS_TEMP" -eq 1 ]] && rm -rf "$WORK"
}
trap cleanup EXIT

# 0.
# Schema-validate; fail closed (never run on a malformed suite).
"$HERE/validate-problem-yaml.sh" >&2 || { echo "run.sh: problems.yaml failed validation; aborting" >&2; exit 2; }

# Emit the per-problem index AND write each problem's ground truth (defects / negatives) to $SPEC.
# Those files feed the grader only — never the reviewer.
# Index line (TAB-separated): <id> <mode> <checkout> <remote> <arg> <n_real> <n_negative>
#   checkout = commit SHA (diff) | base_commit (files);  remote = fetch URL (diff) | "-"
#   arg      = "-" (diff)        | space-joined file targets (files)
emit() {
  python3 - "$HERE/problems.yaml" "$SPEC" <<'PY'
import sys, os, yaml
docs = yaml.safe_load(open(sys.argv[1])); spec = sys.argv[2]
DEFAULT_REMOTE = "https://github.com/asterinas/asterinas"
def block(d, n):
    t = d["target"]; loc = t.get("path") or ("<" + t["kind"] + ">")
    if t.get("lines"): loc += " lines " + str(t["lines"])
    desc   = " ".join(str(d["desc"]).split())
    expect = " ".join(str(d["expectation"]).split())
    # 'defect:' is context; 'MATCH IF:' is the criterion the grader keys on.
    return (f"{n}. location: {loc} (persona: {d['persona']}, grounding: {d['grounding']}, severity: {d['severity']})\n"
            f"   defect: {desc}\n"
            f"   MATCH IF: {expect}")
index = []
for p in docs:
    pid, rm = p["problem_id"], p["review_mode"]
    reals = [d for d in p["defects"] if not d.get("is_negative")]
    negs  = [d for d in p["defects"] if d.get("is_negative")]
    with open(os.path.join(spec, pid + ".defects.txt"), "w") as f:
        f.write("# Expected defects\n\n" + "\n\n".join(block(d, i+1) for i, d in enumerate(reals)) + "\n")
    if negs:
        with open(os.path.join(spec, pid + ".negatives.txt"), "w") as f:
            f.write("# Must NOT be flagged (false-positive traps)\n\n" + "\n\n".join(block(d, i+1) for i, d in enumerate(negs)) + "\n")
    if "diff" in rm:
        mode, co, remote, arg = "diff", rm["diff"]["commit"], rm["diff"].get("remote", DEFAULT_REMOTE), "-"
    else:
        mode, co, remote, arg = "files", p["base_commit"], "-", " ".join(rm["files"])
    index.append("\t".join([pid, mode, co, remote, arg, str(len(reals)), str(len(negs))]))
print("\n".join(index))   # all ground-truth files written before any line is emitted
PY
}

default_review() { # <worktree> <out> <skill-args>
    "$HERE/overlay-skill.sh" "$1"        # current skill into the worktree; excludes benchmark/
    # Guidelines come from GROOT (not the worktree), so the historical diff stays clean.
    ( cd "$1" && ACR_GUIDELINE_ROOT="$GROOT" run_agent \
        "Use the aster-code-review skill with these arguments: $3 $2 --overwrite. Review this working tree." )
}
default_grade() { # <defects-file> <review>
    run_agent "You are grading a code review. The expected defects are in $1; the \
produced review is $2. Each expected defect gives a 'defect:' description for context \
and a 'MATCH IF:' criterion. For each expected defect, decide whether ANY comment in \
the review satisfies its MATCH IF criterion at the stated code location (wording may \
differ). Respond with ONLY two space-separated integers, caught then total, and \
nothing else (for example: 1 2)."
}
default_neg_grade() { # <negatives-file> <review>
    run_agent "The items in $1 are false-positive traps that a correct review must \
NOT raise as real defects. Read the review $2. Output ONLY PASS (none raised) or \
FAIL (at least one raised)."
}
REVIEW_CMD="${REVIEW_CMD:-default_review}"
GRADE_CMD="${GRADE_CMD:-default_grade}"
NEG_GRADE_CMD="${NEG_GRADE_CMD:-default_neg_grade}"

# A selector token matches a problem if its id equals the token or begins with it,
# so "0002" selects 0002-fair-weight-race (numbers are stable; slugs get reworded).
selected() { # <id>
    [[ -z "${PROBLEMS:-}" ]] && return 0
    local id="$1" tok
    for tok in $PROBLEMS; do [[ "$id" == "$tok" || "$id" == "$tok"* ]] && return 0; done
    return 1
}

# Prints "PROD" (smoke: a review was written),
# "OFF c t"/"ON c t" (recall),
# or "NEG PASS|FAIL" (pure-precision);
# non-zero on a setup/review failure.
# Opaque <wt>.
run_one() { # <wt> <id> <mode> <co> <remote> <arg> <n_real> <n_neg>
    local wt="$1" id="$2" mode="$3" co="$4" remote="$5" arg="$6" nreal="$7" nneg="$8" skillargs off on c t out base
    rm -rf "$wt"
    if [[ "$mode" == diff ]]; then
        # The change under review IS the commit.
        # Fetch it by full SHA if absent (PR-derived commits are already on upstream main; synthetic ones are dangling on the fork),
        # check it out detached, and review `diff commit^`.
        git -C "$REPO" cat-file -e "${co}^{commit}" 2>/dev/null \
            || git -C "$REPO" fetch --no-tags "$remote" "$co" >/dev/null 2>&1 || return 1
        git -C "$REPO" worktree add -f --detach "$wt" "$co" >/dev/null 2>&1 || return 1
        base="$(git -C "$wt" rev-parse "${co}^" 2>/dev/null)" || return 1
        skillargs="diff $base"
    else
        git -C "$REPO" worktree add -f --detach "$wt" "$co" >/dev/null 2>&1 || return 1
        skillargs="files $arg"
    fi

    # SMOKE (MIN_RECALL=0): the only question is "did the reviewer run and write a review?"
    # — one combined pass, then NO grading,
    # escalation, or precision check (all of which are about quality/recall, which a smoke does not judge).
    # This is what keeps a smoke fast:
    # half the agent calls, and no flaky low-effort grader.
    if [[ "$MIN_RECALL" -eq 0 ]]; then
        out="$wt.off.md"; rm -f "$out"
        "$REVIEW_CMD" "$wt" "$out" "$skillargs --per-persona-context=no" >&2 || return 1
        [[ -s "$out" ]] || return 1
        printf 'PROD\n'; return 0
    fi

    if [[ "$nreal" -eq 0 && "$nneg" -gt 0 ]]; then     # pure-precision problem
        out="$wt.review.md"; rm -f "$out"
        "$REVIEW_CMD" "$wt" "$out" "$skillargs --per-persona-context=yes" >&2 || return 1
        [[ -s "$out" ]] || return 1
        printf 'NEG %s\n' "$("$NEG_GRADE_CMD" "$SPEC/$id.negatives.txt" "$out")"
        return 0
    fi

    # Recall: cheap combined mode first, escalate to fan-out on a miss.
    # (A problem mixing real and negative defects is recall-graded only;
    #  pure-precision problems take the NEG branch above.
    #  No current problem mixes the two.)
    local df="$SPEC/$id.defects.txt"
    off="$wt.off.md"; rm -f "$off"
    "$REVIEW_CMD" "$wt" "$off" "$skillargs --per-persona-context=no" >&2 || return 1
    [[ -s "$off" ]] || return 1
    read -r c t <<<"$("$GRADE_CMD" "$df" "$off")"
    if [[ "${c:-}" =~ ^[0-9]+$ && "${t:-}" =~ ^[0-9]+$ && "$c" == "$t" && "$t" -gt 0 ]]; then
        printf 'OFF %s %s\n' "$c" "$t"; return 0
    fi
    on="$wt.on.md"; rm -f "$on"
    "$REVIEW_CMD" "$wt" "$on" "$skillargs --per-persona-context=yes" >&2 || return 1
    [[ -s "$on" ]] || return 1
    printf 'ON %s\n' "$("$GRADE_CMD" "$df" "$on")"
}

MIN_RECALL="${MIN_RECALL:-100}"       # recall gate; MIN_RECALL=0 (smoke) reviews only — no grading (run_one)
total_caught=0 total_defects=0 problems=0 off_ok=0 escalated=0 neg_pass=0 neg_total=0 n=0 harness_errors=0 produced=0
while IFS=$'\t' read -r id mode co remote arg nreal nneg <&3; do
    selected "$id" || continue
    n=$((n + 1)); wt="$WORK/wt$n"          # OPAQUE worktree path — never the slug
    if ! result="$(run_one "$wt" "$id" "$mode" "$co" "$remote" "$arg" "$nreal" "$nneg")"; then
        keep_reviews "$id" "$wt"
        printf '%-34s  ?  (harness error — setup/review failed)\n' "$id"; harness_errors=$((harness_errors + 1)); continue
    fi
    keep_reviews "$id" "$wt"
    case "$result" in
        PROD)                                    # smoke: reviewer ran + wrote a review
            produced=$((produced + 1)); printf '%-34s produced ✓\n' "$id" ;;
        NEG\ *)
            verdict="${result#NEG }"; neg_total=$((neg_total + 1))
            [[ "$verdict" == *PASS* ]] && neg_pass=$((neg_pass + 1))
            printf '%-34s precision %s\n' "$id" "$verdict" ;;
        OFF\ *|ON\ *)
            tier="${result%% *}"; read -r caught defects <<<"${result#* }"
            if ! [[ "${caught:-}" =~ ^[0-9]+$ && "${defects:-}" =~ ^[0-9]+$ ]]; then
                printf '%-34s recall  ?/?  (unparseable grader output: %q)\n' "$id" "$result"; harness_errors=$((harness_errors + 1)); continue
            fi
            problems=$((problems + 1)); total_caught=$((total_caught + caught)); total_defects=$((total_defects + defects))
            if [[ "$tier" == OFF ]]; then off_ok=$((off_ok + 1)); label=combined; else escalated=$((escalated + 1)); label=fan-out; fi
            printf '%-34s recall %s/%s [%s]\n' "$id" "$caught" "$defects" "$label" ;;
        *)  printf '%-34s recall  ?/?  (unexpected: %q)\n' "$id" "$result"; harness_errors=$((harness_errors + 1)) ;;
    esac
done 3< <(emit)

echo "----"
[[ -n "$KEEP_DIR" ]] && echo "reviews kept for inspection in: $KEEP_DIR  (per problem: review.md + expected-defects.txt)"
if [[ "$MIN_RECALL" -eq 0 ]]; then     # smoke: reviews only, no grading
    printf 'smoke: %s/%s reviews produced; harness errors: %s\n' "$produced" "$n" "$harness_errors"
    # Pass iff every attempted problem produced a review (no harness errors) and at least one ran
    # — the smoke's whole question,
    # "does the skill run on this agent?".
    [[ "$harness_errors" -eq 0 && "$produced" -gt 0 ]]
else
    recall_pct=0
    [[ "$total_defects" -gt 0 ]] && recall_pct=$(( 100 * total_caught / total_defects ))
    printf 'recall: %s/%s (%s%%, gate >=%s%%) across %s problems; per-persona-context: %s combined, %s fan-out; precision: %s/%s clean; harness errors: %s\n' \
        "$total_caught" "$total_defects" "$recall_pct" "$MIN_RECALL" "$problems" "$off_ok" "$escalated" "$neg_pass" "$neg_total" "$harness_errors"
    # Gate: recall% >= MIN_RECALL,
    # every negative clean, no harness error, >=1 defect measured.
    pass=1
    [[ "$harness_errors" -gt 0 ]] && pass=0
    [[ "$neg_pass" != "$neg_total" ]] && pass=0
    [[ "$recall_pct" -lt "$MIN_RECALL" ]] && pass=0
    [[ "$total_defects" -eq 0 ]] && pass=0
    [[ "$pass" -eq 1 ]]
fi
