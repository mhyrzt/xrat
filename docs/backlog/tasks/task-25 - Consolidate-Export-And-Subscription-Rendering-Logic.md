---
id: TASK-25
title: Consolidate Export And Subscription Rendering Logic
status: To Do
assignee: []
created_date: '2026-07-05 14:43'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-3
dependencies: []
priority: medium
ordinal: 10
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/2-use-cases/10-export-subscription-rendering.md`

# Consolidate Export And Subscription Rendering Logic

## Finding

### [Priority: Medium] Consolidate export and subscription rendering logic

**Files involved:**

- `src/server/routes/json.rs`
- `src/server/routes/b64.rs`
- `src/server/routes/pac.rs`
- `src/app/commands/list.rs`
- `src/app/commands/proxy/pac.rs`

**Problem:** HTTP export routes and CLI formatting paths each assemble output
data directly. The `json` route and `b64` route duplicate filter construction
and top-limit validation. PAC endpoint extraction is embedded in the Axum route
module, while PAC rendering helpers and proxy command code live separately.

**Why this change is needed:** Export behavior is a product feature, not an
Axum-only concern. Duplicated filter and rendering decisions can cause `/json`,
`/b64`, CLI listing, and proxy/PAC support to drift.

**How to implement it:** Create export use-cases such as `ExportConfigsUseCase`
and `PacFileUseCase`. Move top-limit validation, raw-config selection, JSON
summary selection, active proxy endpoint extraction, and PAC rule assembly into
those use-cases. Keep HTTP handlers responsible for auth, query extraction,
headers, and body encoding. Keep CLI commands responsible for terminal
formatting.

**Positive effect on the codebase:** Exports become reusable by CLI, HTTP, TUI
sharing actions, and tests. Adding a new export filter or output route requires
less duplicated work.

**Suggested target architecture:** Application export services return strings or
read models; adapters handle transport details such as headers, base64 encoding,
and terminal output.

**Risk / migration notes:** Low to medium risk. Migrate `/json` and `/b64` first
because they already share query semantics, then extract PAC endpoint selection
separately.
<!-- SECTION:DESCRIPTION:END -->
