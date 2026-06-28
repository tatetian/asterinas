# Benchmark tasks for aster-code-review.
#
# This Makefile owns benchmark detail;
# the package root only delegates to it (`make -C benchmark <target>`).
# The agent is chosen with AGENT_PROFILE (REQUIRED)
# — a profile NAME under agent_profiles/ (e.g. AGENT_PROFILE=codex -> agent_profiles/codex/);
# run.sh itself names no agent (see run.sh header and ../spec/benchmark.md).
# The scripts have shebangs and are executable,
# so they run directly (./run.sh), not via bash.
#
# Targets (default is `check`):
#   make check                              validate problems.yaml (model-free, no agent)
#   make benchmark AGENT_PROFILE=claude     full suite, gate at 100% recall
#   make smoke     AGENT_PROFILE=codex      2-problem smoke: low-effort .smoke profile, no recall gate
#
# Knobs:
#   AGENT_PROFILE=<name>   REQUIRED for benchmark/smoke; the agent_profiles/ subdir to run
#   PROBLEMS="0002 0006"   run only the id-prefix-selected problems (default: all)
#   SMOKE_PROBLEMS="..."   the smoke subset (default below)
#   KEEP=<dir> | KEEP=1    keep produced reviews + expected defects to inspect (1 = temp dir, printed)

# Smoke covers both review modes cheaply: diff-code (0002) + files (0006).
SMOKE_PROBLEMS ?= 0002 0006

.PHONY: check benchmark smoke

# Static check: is problems.yaml well-formed?
# Model-free, needs no agent.
check:
	@./validate-problem-yaml.sh

# Full recall benchmark: base profile, gate at 100%.
# PROBLEMS narrows the set; KEEP=<dir> (or KEEP=1) keeps the produced reviews for inspection.
benchmark:
	@AGENT_PROFILE="$(AGENT_PROFILE)" MIN_RECALL=100 PROBLEMS="$(PROBLEMS)" KEEP_REVIEWS="$(KEEP)" ./run.sh

# Smoke test: does the skill run on this agent and emit gradeable reviews?
# Uses the `.smoke` profile overlay (low effort) and skips escalation;
# no recall gate (MIN_RECALL=0)
# — but a run that errors or yields no review still fails.
smoke:
	@AGENT_PROFILE="$(AGENT_PROFILE)" PROFILE_VARIANT=smoke MIN_RECALL=0 \
	  PROBLEMS="$(or $(PROBLEMS),$(SMOKE_PROBLEMS))" KEEP_REVIEWS="$(KEEP)" ./run.sh
