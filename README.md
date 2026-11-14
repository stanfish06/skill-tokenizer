# skill-tokenizer

Train and use a BPE tokenizer for agent skills, or extract source-grounded facts
and rules from a Markdown skill with an OpenAI-compatible model endpoint.

## Build and check

```sh
git clone --recurse-submodules https://github.com/stanfish06/skill-tokenizer.git
cd skill-tokenizer
mise trust
mise install
cargo build -p skill-tokenizer
lefthook install

cargo fmt -p skill-tokenizer -- --check
cargo test -p skill-tokenizer
cargo clippy -p skill-tokenizer --all-targets --no-deps -- -D warnings
```

For an existing clone, initialize the pinned tokenizer dependency with
`git submodule update --init --recursive`. `mise.toml` supplies Rust, sccache,
and lefthook; use the existing Nix shell if preferred, with those tools available.
Formatting and lint commands target this package, leaving the vendored tokenizer
under upstream control. Automated tests use a local HTTP server and never call a
hosted model or read personal credentials.

## BPE tokenization

The existing commands retain their interface and output:

```sh
cargo run -p skill-tokenizer -- train tests/examples --output tokenizer.json --vocab-size 1000
printf 'TypeScript interface' | cargo run -p skill-tokenizer -- encode tokenizer.json
```

`train` reads Markdown recursively. `encode` reads stdin and prints tokens followed
by token IDs on the next line.

## Atomization

An atom is one independently testable fact or rule explicitly supported by the
skill. Conditional rules retain their conditions, exceptions, versions, and
ordering. Extraction uses a model; BPE tokenization does not discover these units.

Choose the model explicitly and provide a raw API key through `OPENAI_API_KEY` or
a file containing only the key:

```sh
cargo run -p skill-tokenizer -- atomize examples/atomization/gh-cli/SKILL.md \
  --model gpt-6-luna \
  --base-url https://api.openai.com/v1 \
  --api-key-file /path/to/private-key.txt \
  --output atoms.json
```

The model and base URL can also come from `OPENAI_MODEL` and `OPENAI_BASE_URL`.
`--api-key-file` takes precedence over `OPENAI_API_KEY`. Keys are not accepted as
literal command-line arguments and are not written to output or diagnostics.
Local loopback endpoints can run without a key; unset `OPENAI_API_KEY` when a local
server should receive no credentials.

For a local server supporting Chat Completions and JSON mode:

```sh
env -u OPENAI_API_KEY cargo run -p skill-tokenizer -- atomize path/to/SKILL.md \
  --base-url http://127.0.0.1:8000/v1 --model your-local-model --output atoms.json
```

Use `-` as the input path to read stdin. Omit `--output` to write JSON to stdout;
progress and 15-second request heartbeats go to stderr. No files referenced by
the skill are fetched automatically.

### Output contract

The JSON document includes:

- `schema_version`: currently `1`.
- `source`: input path (`-` for stdin), SHA-256 of original bytes, and byte count.
- `prompt`: extraction prompt version and SHA-256.
- `model`: requested model and provider-returned model when available.
- `usage`: provider-reported token counts when available, otherwise `null`.
- `atoms`: locally assigned IDs, statements, `fact`/`rule` kinds, and evidence.

Each evidence entry contains an exact `quote`, inclusive one-based `start_line`
and `end_line`, and a half-open UTF-8 byte range `[start_byte, end_byte)`. Slicing
the original bytes at that range reproduces the quotation exactly. These are byte
offsets, not character or UTF-16 indexes. The original Markdown is preserved,
including CRLF line endings. YAML frontmatter cannot be used as atom evidence.

Exact duplicate statements of the same kind are merged, retaining their distinct
evidence. IDs follow source order and are deterministic for an accepted model
response; they are not persistent identities across different model extractions.
An empty atom list is valid for a document without testable knowledge.

### Limits and failures

`atomize --help` exposes these controls:

| Option | Default | Meaning |
| --- | --- | --- |
| `--max-input-bytes` | 131072 | Maximum original UTF-8 input size; hard upper bound 4 MiB |
| `--max-output-tokens` | 16384 | Completion budget, including reasoning tokens where applicable |
| `--timeout-secs` | 120 | Whole request timeout; maximum 3600 seconds |

The endpoint must implement `/chat/completions`, JSON object response mode,
`max_completion_tokens`, and a text response with `finish_reason: "stop"`.
Malformed JSON, missing evidence, invented quotations, ambiguous quotations,
invalid line ranges, refusals, truncation, and HTTP errors fail with nonzero exit.
Requests are not retried automatically, and HTTP redirects are not followed.
Responses are limited to 8 MiB. The entire result is validated before output is
published; a failed extraction leaves an existing output file intact. Output
cannot overwrite the input skill.

### Reviewing extraction quality

Exact evidence validation proves where the quoted text occurs. It does not prove
that a statement follows from its evidence, that every atom is independently
testable, that facts are current, or that the inventory is complete. Review those
properties against the whole skill before using atoms as evaluation questions.
The model sees the source's instructions as data; it does not execute them.

The extraction prompt is in
[`skill-tokenizer/prompts/atomize-v1.txt`](skill-tokenizer/prompts/atomize-v1.txt).
The [implementation plan](tasks/plan.md) describes the later probe, paired-run,
and grading stages; those stages are not part of this command.
