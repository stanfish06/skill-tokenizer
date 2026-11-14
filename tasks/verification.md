# Atomization verification log — 2026-11-14

## Context and scope

- Fresh clone: `/Users/stan/Git/skill-tokenizer`, initial `master` at `3affbe8`.
- Plan committed and pushed first as `96cfbdf`, before implementation.
- User selected real-skill JSON review plus automated checks and a configurable
  OpenAI-compatible endpoint.
- User approved at most three GPT-6 Luna calls, estimated under $0.05 and 1–3
  minutes. The key was read from a user-specified file only by the CLI, never
  printed or copied into this repository. All three calls have been used; no
  automatic retries or repair calls were made.
- Initialized the pinned Hugging Face tokenizers submodule. The new clone had no
  pre-existing build environments or extra worktrees to preserve or remove.

## Automated checks

1. Added a public CLI integration test and ran
   `cargo test -p skill-tokenizer --test cli -- --nocapture` before implementing
   the command. It failed with `unrecognized subcommand 'atomize'` as expected.
2. Implemented the complete request/validation/output path. The same test passed.
3. Expanded the public CLI suite to cover train/encode compatibility, file/stdin
   input, Unicode and CRLF quotations, multiline code, exact duplicate merging,
   authentication, successful atomic replacement, output preservation on errors,
   fabricated/ambiguous citations, frontmatter, bad schema, truncation, refusals,
   HTTP failures, and local input preflight checks. Seven tests passed.
4. The initial lint command also linted the pinned upstream tokenizer and failed
   on six upstream Clippy findings. Updated the README, plan, and hook command to
   `cargo clippy -p skill-tokenizer --all-targets --no-deps -- -D warnings`.
   This command passed without editing vendored source. Manifest warnings about
   the upstream package's profile and redundant homepage remain informational.
5. Final documented checks and code review: pending before delivery.

The development environment uses Rust nightly, sccache 0.19.0, and lefthook.
`mise trust` is documented because this project's environment configuration
requires it on a fresh checkout. The pre-commit hook checks package formatting
and lints the package without linting dependencies.

## Live source review

1. Initial `gh-cli` run: 13 atoms; 1181 prompt tokens and 1808 completion tokens.
   All citations passed. Three atoms described the document's example contents
   instead of expressing reusable guidance. Revised the prompt to require direct
   statements and to avoid example repetition. This first output was superseded.
2. Final `gh-cli` run: 13 atoms; 1250 prompt tokens and 1842 completion tokens.
3. Final `gh-address-comments` run: 25 atoms; 1700 prompt tokens and 3082 completion
   tokens. Reviewed the statements and evidence against the complete source.

Final outputs and their sources are in `examples/atomization/`. Both final
documents identify the committed prompt by SHA-256. The sample README records
remaining semantic overlap, compound statements, and completeness limitations;
these results are source-grounded draft inventories, not human-approved goldens.

Across all three calls: 4131 prompt tokens and 6732 completion tokens. At the
verified standard [GPT-6 Luna rates](https://developers.openai.com/api/docs/models/gpt-6-luna)
of $0.10/$0.50 per million input/output tokens, the token-based cost estimate is
$0.00378. This is an estimate from reported usage, not an account billing read.
