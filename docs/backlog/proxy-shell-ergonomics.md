# Medium, P3: Revisit proxy shell toggle ergonomics

### Status

Draft

### Problem

`xrat proxy shell toggle` prints shell code that users must evaluate, e.g.
`eval "$(xrat proxy shell toggle)"`. This is necessary because a child process
cannot directly mutate the parent shell environment, but the UX can still be
confusing.

### Questions to answer

- Can the command explain more clearly why it emits shell code?
- Should docs recommend shell functions/aliases that hide the `eval` step?
- Should xrat offer a generated shell integration snippet for bash/zsh/fish?
- Can current proxy variables be read and transformed inside xrat while still
  producing minimal shell-specific output?

### Constraints

- A standalone CLI process cannot set or unset environment variables in the
  already-running parent shell.
- Any enable/disable/toggle flow that changes the caller's shell must use
  `eval`, `source`, shell functions, or shell-specific integration.
- Generated code must remain shell-specific because bash/zsh/fish have different
  syntax.

### Changes required

- Review `src/app/commands/proxy/shell.rs` output and user-facing messages.
- Improve docs in `docs/src/02-cli/proxy.md` with copy-pasteable functions for
  bash, zsh, and fish.
- Consider adding `xrat proxy shell init <shell>` to print a durable shell
  function users can place in their profile.
- Keep the existing script-emitting command for compatibility.

### Verification

- CLI parser tests if a new `init` subcommand is added.
- Unit tests for generated snippets where practical.
- Manual verification in bash, zsh, and fish.
