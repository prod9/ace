School cache: {{ school_cache }}

When skill edits need to go upstream, guide the user through these steps:
1. `ace diff` — review changes.
2. `git -C {{ school_cache }} checkout -b ace/<short-description>` — feature branch.
3. `git -C {{ school_cache }} add` and `git -C {{ school_cache }} commit` — stage and commit.
4. `git -C {{ school_cache }} push -u origin <branch>` — push.
5. Create a PR to the school repo via GitHub MCP if available.
Do NOT reset the cache to main — that destroys uncommitted work.