# Centralize DTO And View-Model Mapping

## Finding

### [Priority: Medium] Centralize DTO and view-model mapping

**Files involved:**

- `src/server/response.rs`
- `src/tui/data/configs.rs`
- `src/tui/data/sources.rs`
- `src/app/commands/list.rs`
- `src/app/commands/lifecycle.rs`

**Problem:** Record-to-output mapping is repeated across server DTOs, TUI rows,
and CLI JSON or table output. For example, `ConfigWithLatestTest` is converted
into `ApiConfigDetail`, `ApiConfigSummary`, and `TuiConfigRow` in separate
modules, while CLI commands build separate JSON values and status labels.

**Why this change is needed:** Duplicated mappings make fields drift across
interfaces. Adding or renaming a config field requires finding every
adapter-specific transformation. Mixed formatting and mapping also makes tests
brittle because domain behavior is tied to presentation details.

**How to implement it:** Create application read models such as `ConfigSummary`,
`ConfigDetail`, `LatestTestSummary`, and `SubscriptionSummary`. Convert database
records into these read models once in the config query service. Then convert
read models into Axum DTOs, TUI rows, or CLI output rows at the adapter edge.
Move shared labels such as config status flags and endpoint labels into small
pure helpers.

**Positive effect on the codebase:** Output adapters become thinner, field
changes are easier to propagate, and read-model tests can verify shared behavior
without HTTP or terminal rendering.

**Suggested target architecture:** Database records stay persistence-oriented;
application read models represent interface-neutral facts; adapter DTOs and
table rows represent presentation.

**Risk / migration notes:** Low to medium risk. Introduce read models without
deleting existing DTOs, then migrate one adapter at a time.
