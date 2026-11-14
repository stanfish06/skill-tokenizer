# Skill atomization and knowledge evaluation plan

## Goal

Extend `skill-tokenizer` with semantic atomization of agent skill Markdown. The
result will support a later evaluation of how much knowledge a skill makes
accessible to a particular model, compared with the same model without the skill.
The existing `train` and `encode` commands are already used by Skillquarium and
must retain their arguments and output behavior.

The user requested that this plan be committed and pushed to `master` before
implementation. Today's implementation covers atomization only.

## Evaluation roadmap

1. **Atomization (today):** produce independently testable, source-grounded facts
   and rules from a skill. Review extraction quality before using them as a test
   inventory.
2. **Probe construction (later):** create questions and answer rubrics that hide
   each atom's answer, preserving its conditions, exceptions, and required order.
3. **Paired model runs (later):** ask identical probes in fresh sessions without
   and with the complete skill, using the same model configuration.
4. **Grading and reporting (later):** grade responses as correct, partial,
   incorrect, unanswered, or ungradable. Calculate baseline coverage, coverage
   with the skill, recovered atoms, regressions, and net gain.

FActScore motivates the decomposition approach, but its generated-answer factual
precision score is different from this fixed, skill-derived coverage inventory.
See the [paper](https://aclanthology.org/2023.emnlp-main.741/) and
[extractor](https://github.com/shmsw25/FActScore/blob/main/factscore/atomic_facts.py).

## Today's interface and architecture

- Add `skill-tokenizer atomize <PATH|->`, writing JSON to stdout or `--output`.
- Use a configurable OpenAI-compatible Chat Completions endpoint. Accept a model
  and base URL through flags or `OPENAI_MODEL` / `OPENAI_BASE_URL`. Require an
  explicit model rather than silently selecting a paid model.
- Accept credentials through `OPENAI_API_KEY` or `--api-key-file`; never put key
  contents in arguments, output, diagnostics, repository files, or test fixtures.
  Permit a local endpoint without credentials.
- Keep extraction instructions in a versioned prompt file. Present the full
  Markdown with line numbers as source material, not executable instructions.
  Preserve cross-paragraph context. Do not follow source links or execute code.
- Start with a bounded, single-document request. Reject oversized inputs and
  truncated responses explicitly; do not silently omit sections or issue an
  unbounded number of requests. Expose input/output limits and request timeout.
- Request a JSON object containing atoms. Each atom is a `fact` or `rule` with one
  independently testable statement and one or more exact evidence quotations
  with source line ranges. Conditions, exceptions, versions, ordering, and exact
  code identifiers belong in the statement when they affect its meaning.
- Exclude metadata and navigation-only text. Preserve meaningful knowledge
  expressed in prose, lists, tables, and code examples. Do not invent implications
  or turn illustrative example values into universal requirements.
- Validate types, nonempty statements, evidence, source ranges, and exact quoted
  text locally. Derive byte offsets from the original UTF-8 source and reject
  unsupported or ambiguous citations. Remove exact duplicate atoms, order them
  by provenance, and assign deterministic IDs for the accepted response.
- Emit a versioned JSON document containing source identity and hash, prompt
  version, requested/returned model, token usage when supplied, and accepted
  atoms with their evidence. Validation proves provenance and structure;
  semantic fidelity, atomicity, and completeness still need review.
- Validate the entire response before publishing output. A failed request or
  invalid response must not overwrite an existing output file.

## Ordered implementation tasks

### 1. Establish the baseline and regression checks

Read the Rust workspace, its pinned tokenizer submodule, and dependency source.
Run tests through the public CLI with representative Markdown fixtures.

Acceptance: the existing tokenizer can still train and encode with its current
interface. The new atomization test fails before implementation.

### 2. Implement one complete atomization path

Add CLI options, the versioned extraction prompt, typed request/response models,
the endpoint adapter, evidence validation, and JSON output. Use an in-process
local HTTP test server at the CLI boundary; paid credentials are not needed for
the automated suite.

Acceptance: a caller can atomize a file or stdin; cited text and UTF-8 offsets
match the original input; conditional rules and code survive the JSON round trip.

### 3. Cover failure paths and document operation

Exercise malformed JSON, fabricated evidence, bad line ranges, refusals,
truncation, HTTP failures, input limits, and output preservation. Add documented
build/test/lint commands and pre-commit format/lint hooks.

Acceptance: errors exit nonzero with useful diagnostics and leave existing output
intact. Tests use public CLI behavior, not private implementation details.

### 4. Verify a real skill and review before delivery

The user selected a real `SKILL.md` JSON review plus automated checks. Use a small
public skill, then a procedural skill if the first result warrants another run.
Before any paid request, confirm approximate cost, time, model, and call budget.
Read credentials only inside the requesting process.

Acceptance: inspect the generated atoms against the whole source for invented
claims, missed content, duplicated facts, and lost conditions/order. Keep a
reproducible example command and a concise verification log. Review the code
against this plan and address valid findings.

## Verification commands

Run from the repository root, with the pinned tokenizer submodule initialized:

```sh
git submodule update --init --recursive
cargo fmt --all -- --check
cargo test -p skill-tokenizer
cargo clippy -p skill-tokenizer --all-targets -- -D warnings
cargo build -p skill-tokenizer
```

The README will contain the exact live atomization command, including credential
file, configurable model, and output file. Offline tests must never read personal
credentials or call hosted models. Use sccache for Rust compilation when available.

## Risks and boundaries

- An LLM can produce plausible but incomplete or non-atomic statements. Exact
  evidence matching catches fabricated quotations, not semantic mistakes; review
  real output and document this limitation.
- Questions, judges, paired evaluations, embeddings, automatic skill pruning,
  package publication, and tokenizer algorithm changes are outside today's scope.
- Batch extraction and context-aware chunking can follow the single-file path;
  this version reports limits explicitly instead of claiming complete extraction
  for inputs it cannot process.
- No paid retries or automatic repair calls: a caller decides whether to rerun a
  failed extraction after seeing the error.

## Decisions confirmed with the user

- Deliver atomization as part of the existing package.
- Commit and push the plan to `master` first, then implement today.
- Verify with a real skill's JSON output and automated checks.
- Use a configurable OpenAI-compatible endpoint.
