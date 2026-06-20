# Medium, P2: Redesign the TUI rename-subscription modal

### Status

Planned

### Goal

Make the rename-subscription modal (`render_rename_modal` in
`src/tui/view/modals.rs`) compact, informative, and consistent with the other
modals. Identify the subscription by `ref` + name instead of a raw numeric id,
and size the box to its content.

### Current behavior (the flaws)

From the rendered modal (title `Rename Subscription #7`):

- **Oversized box.** It uses `centered_rect(60, 30, area)` — a fixed 60% × 30%
  rectangle — for three short lines of content, so most of the modal is empty
  space. The Help and QR modals use `centered_rect_fixed` and size to content;
  this modal should too.
- **Identifies by raw id.** The title shows `#{source_id}` (the numeric primary
  key), which is meaningless to the user. It should show the subscription `ref`
  and current `name`, the same identity used elsewhere. This matches the broader
  request to stop surfacing internal ids (configId / source_id) and show `ref` +
  name instead.
- **No current name shown.** The input starts empty and the existing name is
  never displayed, so the user cannot see what they are renaming from. The input
  should prefill or display the current name.
- **Weak input affordance.** The field is a bare string plus a `█` block on a
  muted background with no border, padding, or placeholder — it does not read as
  an editable field.
- **Low-contrast chrome.** The instruction line and the `Enter save  Esc cancel`
  hint are both muted; with the empty input there is no clear focal point.
- **Full-width input line.** The input stretches the full 60% width with no
  max-width clamp, exaggerating the empty space.

### Changes required

- Carry `ref` and current `name` into `RenameModalState`
  (`src/tui/app/types.rs`), populated when the modal opens
  (`src/tui/run/mod.rs`), so the view can render identity and prefill.
- Title/header: `Rename <ref> · <name>` (or a two-line header with ref + name)
  instead of `#{source_id}`.
- Prefill the input with the current name, or show it as `Current: <name>` above
  an empty input.
- Size the modal to content with `centered_rect_fixed`, with a sane min/max
  width, matching the Help/QR modals.
- Give the input a bordered field with horizontal padding and a placeholder when
  empty; keep the block cursor.
- Keep the keymap and save/cancel flow unchanged; this is presentation only.

### Verification

- Manual: open the rename modal on a subscription and confirm the box is
  compact, the header shows `ref` + name, and the current name is
  visible/prefilled.
- Confirm save and cancel still work and the error line still renders on a
  failed rename.
- A view/unit test that the rendered header contains the subscription `ref` and
  name rather than the numeric id, if the modal rendering is testable in
  isolation.

### Open decisions

- Prefill the input with the current name (fast edit) versus showing it as a
  read-only `Current:` line with an empty input (avoids accidental keep).
  Leaning prefill.
- Whether to apply the same `ref` + name identity treatment to the other modals
  and confirmations that currently show `#id` (delete/refresh source), as a
  follow-up consistency pass.
