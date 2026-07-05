---
id: TASK-21
title: Split Large Command And Schema Files
status: To Do
assignee: []
created_date: '2026-07-05 14:43'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-2
dependencies: []
priority: medium
ordinal: 27
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/1-foundation/27-split-large-command-files.md`

# Split Large Command And Schema Files

## Finding

### [Priority: Medium] Split oversized command modules once logic is extracted

**Files involved:**

- `src/app/commands/list.rs` (724 lines, ~32 fns)
- `src/app/commands/rotate.rs` (617 lines)
- `src/db/schema.rs` (504 lines)
- `src/app/commands/lifecycle.rs` (480 lines)
- `src/app/commands/validate.rs` (442 lines)
- `src/app/commands/proxy/shell.rs` (438 lines)
- `src/app/commands/logs.rs` (434 lines)

**Problem:** Several command modules have grown past the point where a single
file mixes argument translation, business orchestration, repository access, and
terminal rendering. `list.rs` alone holds ~32 functions covering filter
construction, subscription enrichment, sorting, and multiple output formats.
`rotate.rs`, `lifecycle.rs`, and `validate.rs` similarly fold domain rules into
the command handler. This is the concrete file-size symptom of the layering gaps
called out in `01`, `02`, and `04`.

**Why this change is needed:** The project's own guidelines prefer small modules
split by capability, and large mixed files are harder to read, test, and review.
Once the use-cases in `01`–`05` extract the orchestration and the read-model
mapping in `06` extracts the formatting, these files should shrink — but that
shrink needs to be an explicit, verified outcome, not an accident. Stating a size
target keeps the extractions honest and prevents the "moved logic but left the
file huge" failure mode.

**How to implement it:** Treat this as the verification step for `01`/`02`/`04`/
`06`, not standalone surgery. After a use-case is extracted, split the remaining
adapter file by sub-command or by capability (args→request translation, output
rendering) into sibling modules under the command's directory, targeting roughly
<300 lines per file. For `db/schema.rs` (which is not a command file), split by
table/concern. Do not pre-split before the logic extraction — that would just
move lines without reducing coupling.

**Positive effect on the codebase:** Command files become thin, readable
adapters. Reviews shrink. The size reduction is a measurable confirmation that
`01`–`06` actually moved logic out of the adapters rather than just adding a layer
on top.

**Suggested target architecture:** Each command directory holds a small handler
(arg→request, render result) plus focused submodules; orchestration lives in
use-cases; persistence lives in repositories.

**Risk / migration notes:** Low risk if done as the last step of each use-case
extraction with tests already in place. Do not undertake as an independent
"file-splitting" pass — without the prior logic extraction it produces churn
without reducing coupling. Sequence after `01`, `02`, `04`, `06`.
<!-- SECTION:DESCRIPTION:END -->
