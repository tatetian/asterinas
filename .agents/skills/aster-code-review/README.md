# aster-code-review

A code-review **skill** for the Asterinas OS kernel.
It reviews code — either a Git change (`diff` mode) or a set of target files (`files` mode),
against the working tree — and writes a single Markdown review file.
It makes both *objective* calls (undeniable bugs) and *subjective* calls (guideline violations),
the latter grounded in Asterinas's comprehensive,
persona-keyed [Coding Guidelines](../../../book/src/to-contribute/coding-guidelines/).

It is **agent-agnostic**: the same package runs under both Claude Code and Codex.
It is **local-first**: no server and no PR are required;
it reads the repository and writes a file.
It is **benchmark-driven**: a suite of review problems built from real kernel defects measures how many it catches (recall),
so the guidelines and harness improve against evidence rather than intuition.

## Quick start

`aster-code-review` is a skill,
not a binary — you trigger it from inside an agent session, not from a shell.
It reviews the **working tree**, so to review a specific commit you check it out first.

It works in one of two **review modes**.
The `diff` mode reviews the working tree against a `base` commit:

```
diff <base> <output> [--overwrite]
```

The `files` mode reviews a set of specified files in the working tree:

```
files <path[:lines] ...> <output> [--overwrite]
```

Both modes write the reviews in a review file at path `<output>`.

This skill supports both Claude Code and Codex.

- **Claude Code:** `/aster-code-review diff main review.md`,
  or just ask: *"Use aster-code-review to review what this branch added over main,
  into review.md."*
- **Codex:** *"Use the aster-code-review skill to review kernel/src/sched/fair.rs into review.md."*

See [the interface spec](spec/interface.md) for the full argument semantics.

## What's in this directory

| Path | What it is |
|---|---|
| [`SKILL.md`](SKILL.md) | The agent-facing entry point: orchestration pipeline, the shared persona-pass contract, and the spawn shim. |
| [`personas/`](personas/) | One pass template per reviewer persona; each points at its guideline page and lists its ordered concerns. |
| [`scripts/`](scripts/) | The deterministic primitives: `resolve-target.sh` (parse args → canonical review input), `build-pass-prompt.sh` (cache-ordered persona pass prompt; uses `pass-contract.md`), and `assemble-review.sh` (fragments → review file). |
| [`benchmark/`](benchmark/) | The recall benchmark — *review quality* — `problems.yaml` (fixtures cited by commit SHA), `agent_profiles/` (pick the agent via `AGENT_PROFILE`), and the `run.sh` harness. Agent-agnostic: Claude and Codex both verified. |
| [`tests/`](tests/) | Integration tests for the deterministic scripts — *machinery*, no model needed. One suite per script. |
| [`spec/`](spec/) | **The design specification** — the authoritative, self-contained design doc. Start at [`spec/README.md`](spec/README.md). |
| [`Makefile`](Makefile) | Task dispatcher: `make test`, `make benchmark`/`make smoke` (both need `AGENT_PROFILE=<name>`), `make help`. |

## Design specification

The full design — motivation,
the persona-keyed guidelines it consumes,
the interface and output format,
the execution model, and the benchmark
— lives under [`spec/`](spec/),
with every significant decision justified inline.
New contributors should read [`spec/README.md`](spec/README.md) first.
