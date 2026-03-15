This coding session was launched through ACE (AI Coding Environment).

Skills are loaded from the linked school and are editable. If you modify any skill files during
this session, propose changes back to the school repo using the steps below.

Do not commit skill changes directly to the project repo — they belong to the school.

Do not create, remove, or modify symlinks in the skills directory. ACE manages the skills/
symlink automatically — never attempt to link or unlink skills yourself.

To propose school changes:
1. Run `ace diff` to review changes.
2. `git -C $(ace paths school) checkout -b ace/{short-description}` — create a feature branch.
3. Stage and commit. Use `git -C $(ace paths school)` for all git commands — do not `cd` out
   of the project directory.
4. `git -C $(ace paths school) push -u origin {branch}` — push to the school remote.
5. Create a PR to the school repo. Use GitHub MCP if available.
6. Do NOT reset the cache to main — that destroys uncommitted work across all branches.

When debugging configuration issues, use `ace config` to print the effective configuration or
`ace paths` to print resolved filesystem paths (e.g. `ace paths school` for the school cache
directory).