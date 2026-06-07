# 01. Easy, P2: VHS tape demos in READMEs

### Status

Planned

### Goal

Replace the static `media/screenshot.png` preview with animated VHS demos.

### Current assets

- Tape files:
  - `media/tapes/tui.tape`
  - `media/tapes/cli.tape`
- Expected generated files:
  - `media/gif/tui.gif`
  - `media/gif/cli.gif`

Render commands:

```sh
mkdir -p media/gif
vhs media/tapes/tui.tape
vhs media/tapes/cli.tape
```

### Changes required

- Generate and commit `media/gif/tui.gif` and `media/gif/cli.gif`.
- Add a `Justfile` recipe, for example `just gifs`, that creates `media/gif/`
  and renders both VHS tapes.
- Replace the screenshot image block in `README.md`.
- Replace the screenshot image block in `docs/src/README.md`.
- Verify the mdBook asset path. `docs/src/README.md` currently uses a
  `media/...` path prefix, so the gifs may need to be copied under
  `docs/src/media/gif/` or referenced differently.

### Verification

- Confirm both gifs render locally.
- Confirm `just gifs` renders both gifs into `media/gif/`.
- Confirm README image paths resolve.
- Confirm mdBook renders the docs preview correctly.

### Decisions

- Use two separate gifs: one for CLI and one for TUI.
- Remove `media/screenshot.png` after the gifs are added.
- Keep gif regeneration manual through `just gifs`; do not add CI regeneration
  for the first pass.
