# Extract Shared Config Query And Listing Use-Cases

## Finding

### [Priority: High] Extract shared config query and listing use-cases

**Files involved:**

- `src/server/routes/configs.rs`
- `src/server/routes/json.rs`
- `src/server/routes/b64.rs`
- `src/app/commands/list.rs`
- `src/tui/data/mod.rs`

**Problem:** Config listing behavior is implemented separately by HTTP routes,
CLI commands, and TUI data loading. `list_configs`, `get_config`, `json`, and
`b64` build `ConfigListFilter` values and call database methods directly. The
CLI list command builds filters, loads subscriptions, formats rows, and prints
output in one module. `TuiData::load` repeats the query path and applies its own
sort and summary counts.

**Why this change is needed:** Every interface has to remember the same filter
defaults, pagination limits, top-by-delay behavior, deleted/enabled semantics,
and row enrichment rules. This increases duplication and makes new config-list
features risky because a change can easily land in the CLI but not in the HTTP
API or TUI.

**How to implement it:** Create a shared application service under
`src/app/use_cases/configs.rs` or `src/app/services/configs.rs`. Add input
structs such as `ConfigListRequest`, `ConfigDetailRequest`, and
`ConfigExportRequest`, plus result structs that carry domain/application rows
without interface formatting. Move filter construction, pagination validation,
top limit validation, subscription-ref enrichment, and config identifier
resolution into this service. Update Axum handlers to translate query params
into request structs, CLI list to translate args into requests and format
results, and `TuiData::load` to consume the same result model.

**Positive effect on the codebase:** Config feature additions become one
application-core change plus small adapter updates. Tests can exercise
filtering, pagination, prefix resolution, and export selection once without
going through every interface.

**Suggested target architecture:** Repositories expose persistence primitives;
`ConfigUseCases` owns config querying and selection rules; CLI, TUI, Axum, and
daemon code only map their inputs and outputs.

**Risk / migration notes:** This is safe to do incrementally. Start by moving
the read-only list/detail paths and keep existing route/CLI tests. Add use-case
tests before removing the duplicated adapter logic.
