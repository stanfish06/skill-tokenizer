
# Usage 5's generated handler assumes that the cursor is on the final word.
# Scope the words it sees to the cursor, leaving later arguments on the line intact.
_skill_tokenizer_at_cursor() {
    local -a words=( "${(@)words[1,CURRENT]}" )
    _skill_tokenizer "$@"
}
compdef _skill_tokenizer_at_cursor skill-tokenizer
