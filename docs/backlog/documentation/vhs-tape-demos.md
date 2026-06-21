# Easy, P2: VHS tape demos in READMEs

### Status

Planned

### Goal

Replace the static `docs/src/media/screenshot.png` preview with animated VHS
demos.

### Current assets

- Tape files:
  - `docs/src/media/tapes/tui.tape`
  - `docs/src/media/tapes/cli.tape`
- Expected generated files:
  - `docs/src/media/gif/tui.gif`
  - `docs/src/media/gif/cli.gif`

Render commands:

```sh
mkdir -p docs/src/media/gif
vhs docs/src/media/tapes/tui.tape
vhs docs/src/media/tapes/cli.tape
```

### Changes required

- Generate and commit `docs/src/media/gif/tui.gif` and
  `docs/src/media/gif/cli.gif`.
- Add a `Justfile` recipe, for example `just gifs`, that creates
  `docs/src/media/gif/` and renders both VHS tapes.
- Replace the screenshot image block in `README.md`.
- Replace the screenshot image block in `docs/src/README.md`.
- Verify mdBook asset paths resolve from `docs/src/media/...`.

### Verification

- Confirm both gifs render locally.
- Confirm `just gifs` renders both gifs into `docs/src/media/gif/`.
- Confirm README image paths resolve.
- Confirm mdBook renders the docs preview correctly.

### Decisions

- Use two separate gifs: one for CLI and one for TUI.
- Remove `docs/src/media/screenshot.png` after the gifs are added.
- Keep gif regeneration manual through `just gifs`; do not add CI regeneration
  for the first pass.
