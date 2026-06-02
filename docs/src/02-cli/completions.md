# completions

Generate shell completion scripts for xrat.

```bash
xrat completions <shell>
```

This is a hidden command (not shown in `--help`) intended for shell setup and
release packaging.

## Arguments

| Argument  | Description                                  | Values                              |
| --------- | -------------------------------------------- | ----------------------------------- |
| `<shell>` | Shell to generate completions for (required) | `bash`, `zsh`, `fish`, `powershell` |

## Per-shell installation

### Bash

```bash
mkdir -p ~/.local/share/bash-completion/completions
xrat completions bash > ~/.local/share/bash-completion/completions/xrat
```

Reload: open a new shell or `source ~/.bashrc`.

### Zsh

```bash
mkdir -p ~/.zfunc
xrat completions zsh > ~/.zfunc/_xrat
```

Add to `~/.zshrc` if not already present:

```zsh
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit
```

Reload: `exec zsh`.

### Fish

```bash
xrat completions fish > ~/.config/fish/completions/xrat.fish
```

Reload: open a new Fish shell.

### PowerShell

```powershell
xrat completions powershell > xrat.ps1
. ./xrat.ps1
```

Add the dot-source line to your `$PROFILE` for persistence.

## Release packaging

Pre-generated completion scripts are included in release archives under
`completions/`:

| File                    | Shell |
| ----------------------- | ----- |
| `completions/xrat.bash` | Bash  |
| `completions/_xrat`     | Zsh   |
| `completions/xrat.fish` | Fish  |

CI generates these during the release workflow using `xrat completions <shell>`.

## Related

- [Quickstart](../01-getting-started/quickstart.md)
- [`manpage`](manpage.md)
