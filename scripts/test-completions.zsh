#!/usr/bin/env zsh
# Verify the public CLI's generated Zsh script and its Usage completion results.
emulate -L zsh
setopt errexit pipefail

binary=${1:-./target/debug/skill-tokenizer}
binary=${binary:A}
command -v usage >/dev/null || { print -u2 'Install the Usage CLI first: https://usage.jdx.dev'; exit 1; }

completion_tmp=$(mktemp -d "${TMPDIR:-/tmp}/skill-tokenizer-completions.XXXXXX")
trap 'rm -rf -- "$completion_tmp"' EXIT
export XDG_CACHE_HOME="$completion_tmp/cache"

"$binary" usage > "$completion_tmp/spec.kdl"
"$binary" completions zsh > "$completion_tmp/_skill-tokenizer"
zsh -n "$completion_tmp/_skill-tokenizer"
autoload -Uz compinit
compinit -D -i
source "$completion_tmp/_skill-tokenizer"
[[ ${_comps[skill-tokenizer]-} == _skill_tokenizer ]]

expect_candidate() {
    local expected=$1
    shift
    local output row
    output=$(usage complete-word --shell zsh -f "$completion_tmp/spec.kdl" -- skill-tokenizer "$@")
    for row in "${(@f)output}"; do
        [[ ${row%%$'\t'*} == "$expected" ]] && return 0
    done
    print -u2 -- "Missing completion '$expected' for: skill-tokenizer ${(j: :)@}"
    print -u2 -r -- "$output"
    return 1
}

expect_candidate atomize a
expect_candidate train tr
expect_candidate encode en
expect_candidate --usage --u
expect_candidate --api-key-file atomize --api
expect_candidate --max-output-tokens atomize --max-out
expect_candidate zsh completions z

print -r -- '# Sample source' > "$completion_tmp/sample skill.md"
print -r -- '{}' > "$completion_tmp/model.json"
print -r -- '# Not a directory' > "$completion_tmp/training-file.md"
mkdir "$completion_tmp/training-data"
cd "$completion_tmp"
expect_candidate 'sample skill.md' atomize --model test-model sam
expect_candidate 'sample skill.md' atomize --api-key-file sam
expect_candidate model.json encode mod
expect_candidate training-data/ train tra

directory_results=$(usage complete-word --shell zsh -f "$completion_tmp/spec.kdl" -- skill-tokenizer train tra)
[[ $directory_results != *training-file.md* ]]
print -r -- 'Zsh completions passed: registration, subcommands, flags, shell choices, files with spaces, and directories.'
