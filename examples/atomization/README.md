# Live atomization samples

These are snapshots of real skills from the user's installed
[Skillquarium library](https://github.com/stanfish06/skillquarium), copied on
2026-11-14. Both `atoms.json` files are unedited outputs of the CLI using
`gpt-6-luna` and the committed `atomize-v1` prompt. They are reviewable examples,
not golden inventories asserting that every possible atom has been found.

| Skill | Input bytes | Output atoms | Input tokens | Completion tokens |
| --- | ---: | ---: | ---: | ---: |
| `gh-cli` | 1242 | 13 | 1250 | 1842 |
| `gh-address-comments` | 3536 | 25 | 1700 | 3082 |

From the repository root, reproduce the command with your own credential file:

```sh
cargo run -p skill-tokenizer -- atomize examples/atomization/gh-cli/SKILL.md \
  --model gpt-6-luna --base-url https://api.openai.com/v1 \
  --api-key-file /path/to/private-key.txt --output atoms.json
```

Substitute `gh-address-comments/SKILL.md` for the procedural example. A new model
run can produce different atoms; the stored model, prompt hash, and source hash
identify the saved run. The original skill files stay beside their JSON so each
statement can be reviewed without access to the installed library.

## Review observations

- Every saved quotation passed exact-text, source-line, and byte-offset checks.
  Statements were then manually compared with the complete input skill.
- The final `gh-cli` output expresses guidance directly rather than reporting
  that a document contains examples. The concrete command examples were not
  emitted separately; some invocation details therefore have no separate atom.
- The procedural example retains authentication conditions, the selected-fix
  scope, ordering before edits, and the explicit-authorization condition on
  GitHub writes. The original source's instructions were analyzed as text, not
  followed by the extractor.
- `gh-address-comments` atoms 0001 and 0007 overlap: the latter states the known
  repository/PR condition explicitly. Exact-string deduplication does not merge
  semantic paraphrases. Some conditional steps and lists also remain compound
  statements, so independent testability still needs human review.
- The JSON proves provenance, not that every statement is true in today's APIs
  or suitable for the future knowledge evaluation. These samples preserve the
  source's guidance, including environment-specific instructions.

The [verification log](../../tasks/verification.md) records the initial prompt
revision, call budget, and automated checks.
