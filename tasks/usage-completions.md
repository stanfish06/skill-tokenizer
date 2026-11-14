# Usage integration — 2026-11-14

## Scope

Add jdx's Usage to the existing Clap CLI for shell completions and a reusable
specification. Preserve `train`, `encode`, and `atomize`. The user selected Zsh
verification of subcommands, flags, and file paths, plus automated CLI checks.
Baseline: `976c8e5`. No paid APIs, credential reads, or tokenizer changes are
needed for this work.

## Implementation

- Depend on released `clap_usage` 5.0.0, resolving `usage-lib` 5.1.0 in Cargo.lock.
  Read the downloaded crate source; the upstream main branch has additional APIs
  that are not part of this release.
- Expose `usage` and its Clap `--usage` flag alias to print a KDL spec, and
  `completions <shell>` to generate scripts for Bash, Fish, Nushell, PowerShell,
  and Zsh. Scripts embed the spec and call the external `usage` binary at runtime.
- Build the Clap command tree before exporting it so inferred value names and
  generated help/version options are materialized.
- The released exporter omits path hints, environment-variable names, and
  flag-triggered subcommands. Supplement those from the same Clap command tree;
  do not maintain a parallel command or flag definition.
- Add explicit file/directory value hints to the existing path arguments.
- Document installation/loading without modifying the user's shell startup
  files or replacing an installed binary.

## Verification log

1. A new public CLI test failed with `unrecognized subcommand 'usage'` before
   implementation, then passed with spec generation. It also checks `--usage`
   and ensures sentinel environment values never enter the spec.
2. Extending the test with file/directory hints first failed because the released
   exporter dropped them. The metadata supplement made that test pass. It also
   checks the root alias and exported environment-variable names.
3. `scripts/test-completions.zsh` verifies generated script syntax and registration,
   then asks the real Usage runtime for commands, flags, shell choices, paths
   containing spaces, model files, and matching directories. It passed with
   Usage CLI 3.5.6 and 7.3.0. Usage retains its own filename fallback when no
   directory matches; the test checks filtering with both a matching directory
   and a file present, rather than assuming that fallback is disabled.
4. In a fresh `zsh -f` pseudo-terminal, initialized compinit and sourced the
   generated script. Actual Tab input expanded `a` to `atomize`, `--api` to
   `--api-key-file`, and the example source prefix to `SKILL.md`. Exited the
   temporary shell afterward.
5. Package verification passed: 12 tests, formatting, Clippy, and build. Both
   review axes found the same Zsh cursor-position defect in Usage 5's generator:
   it completed the last word even when the cursor was earlier in the line.
6. Added a registered-handler regression using `--api README.md` with the cursor
   on `--api`. It first failed by offering `README.md`. Added a Zsh-only adapter
   that scopes the handler's local word array to the cursor, preserving the
   actual line. The check now offers `--api-key-file`. Actual ZLE Tab verification
   also changed the flag and left the trailing `README.md` intact.
7. Both reviewers inspected the adapter, reran the Zsh check, and closed the
   finding. No remaining concrete spec findings, standards breaches, or material
   heuristic smells. The check also passes with Usage 7.3.0.

## Commands

Run from the repository root:

```sh
cargo fmt -p skill-tokenizer -- --check
cargo test -p skill-tokenizer
cargo clippy -p skill-tokenizer --all-targets --no-deps -- -D warnings
cargo build -p skill-tokenizer
zsh scripts/test-completions.zsh
```

The original atomization samples and extraction prompt are unchanged. This is a
completion and metadata integration, not a migration to Usage's Rust parser.

Sources: [Clap integration](https://usage.jdx.dev/spec/integrations/clap),
[released adapter](https://docs.rs/crate/clap_usage/5.0.0/source/src/generate.rs),
[shell completion guide](https://usage.jdx.dev/cli/completions).
