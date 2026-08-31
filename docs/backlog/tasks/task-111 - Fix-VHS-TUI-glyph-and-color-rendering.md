---
id: TASK-111
title: Fix VHS TUI glyph and color rendering
status: Done
assignee:
  - '@codex'
created_date: '2026-08-31 20:23'
updated_date: '2026-08-31 21:03'
labels:
  - documentation
  - bug
dependencies: []
references:
  - docs/src/media/tapes/tui.tape
  - docs/src/media/tapes/base.tape
modified_files:
  - docs/src/media/tapes/base.tape
  - docs/src/media/gif/tui.gif
priority: medium
ordinal: 85000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Correct the VHS configuration used by docs/src/media/tapes/tui.tape so the recorded TUI renders terminal glyphs and colors faithfully.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The TUI tape renders its line-drawing and symbol glyphs without missing or substituted characters
- [x] #2 The generated TUI recording preserves the intended terminal color palette
- [x] #3 The tape renders successfully with the documented Justfile workflow
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Make VHS color behavior independent of the invoking shell. 2. Regenerate the TUI GIF with the original chart markers. 3. Validate the tape and visually verify the palette.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Root cause: VHS inherited NO_COLOR=1 from the developer shell, which disabled Ratatui RGB styles. The shared tape also declared xterm-256color/FORCE_COLOR=1 despite the TUI using truecolor. Updated the environment to xterm-direct, cleared NO_COLOR, and forced color level 3. Validation: vhs validate passed; the documented Justfile tape recipe completed successfully; a coalesced 1600x900 GIF frame was visually checked for box drawing, arrows, checks, Braille charts, and orange/green/red/blue/magenta styles.

Per user direction, discarded the chart-marker/font experiment and retained the original Braille probe chart. Regenerated the GIF with only the truecolor/NO_COLOR correction active.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Corrected VHS color rendering by clearing inherited NO_COLOR and declaring direct 24-bit color support. Preserved the original TUI glyphs and chart markers, then regenerated and validated the tracked GIF.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
