# Phase 3.5 Local App Configuration

## Goal

Define the local machine configuration layer that sits between Phase 3
connection testing and Phase 4 managed runtime.

By the end of this phase, XRAT should have a clear answer for:

- what belongs in `~/.config/xrat/config.toml`
- what local policy files may be referenced by `config.toml`
- how local config, geo assets, and route lists are created, loaded, and
  validated
- how local disk configuration differs from SQLite-persisted config records
- how Phase 4 runtime generation will consume these files

This phase is about the shape of local app policy and runtime defaults. It is
not yet the full implementation of long-lived runtime control.

## Why This Phase Exists

Phase 3 proved that XRAT can store configs, test them, and use short-lived Xray
processes for real probing.

Phase 4 needs one more layer before it can be implemented cleanly:

- a stable place for machine-local settings
- a stable place for user-managed direct/block rules
- a stable place for managed custom geo assets
- a clear ownership boundary between local files and SQLite state

Without this phase, Phase 4 would likely hardcode ports, routing defaults, and
direct/block policy directly into runtime code, then immediately need
refactoring.

Phase 3.5 exists to prevent that.

## Scope Boundary

Phase 3.5 should cover:

- the canonical config directory layout under `~/.config/xrat/`
- the canonical naming of `config.toml`
- the purpose and shape of routing direct/block lists
- the purpose and shape of managed custom geo assets
- the ownership boundary between:
  - `config.toml`
  - local files referenced by `config.toml`
  - SQLite data in `db.sqlite`
- first-pass parsing and validation rules
- how these files map into Phase 4 runtime config generation

Phase 3.5 should not yet cover:

- full `connect`, `disconnect`, or `status` behavior
- TUI or HTTP API configuration editing
- advanced profile management
- automatic syncing between remote subscriptions and local route rules
- policy UIs beyond file-based editing

Those belong to later phases.

## Desired User Experience

The first usable version should feel like this:

- XRAT creates or expects `~/.config/xrat/config.toml`
- the user edits `config.toml` directly
- XRAT may fetch or read local geo assets referenced by `config.toml`
- XRAT uses them as local-machine policy inputs
- SQLite remains the source of truth for imported remote configs and history

That gives the app a clean split:

- SQLite = persisted imported config records and runtime/test history
- local files = machine-local policy and defaults

## Current Starting Point

The codebase already has path bootstrapping in `src/app/path.rs`.

It currently:

- resolves the app root under `XRAT_PATH` or `~/.config/xrat`
- creates the runtime directory if missing
- creates a default config file if missing
- uses `Config.toml` with a capital `C`

This phase sets the canonical name to `config.toml` in lowercase.

## Local File Model

Phase 3.5 should define three categories of local state:

### 1. SQLite database

Stored in the XRAT runtime directory as `db.sqlite`.

Responsibilities:

- imported configs
- subscriptions
- connection test history
- runtime session history
- selection and active flags

This remains structured application state.

### 2. `config.toml`

Stored in the XRAT runtime directory as the app-level settings file.

Responsibilities should likely include:

- local inbound ports
- runtime defaults for generated Xray configs
- optional DNS behavior
- optional routing toggles
- paths to auxiliary local files
- future app-level preferences that are not tied to one imported node

This should not duplicate node-specific values already persisted in SQLite.

### 3. Geo assets

Managed under the XRAT runtime directory, normally as:

- `geo/<profile>/geosite.dat`
- `geo/<profile>/geoip.dat`

Responsibilities should likely include:

- named geo asset profiles
- local file or URL sources for `geosite.dat` and `geoip.dat`
- optional automatic update behavior
- a default save location under the XRAT config directory

Geo assets are local machine policy inputs. They are not imported remote nodes
and should not be persisted as node records in SQLite.

## Recommended Ownership Boundary

A useful first-pass rule is:

- if the value describes an imported remote node, it belongs in SQLite
- if the value describes local machine behavior, it belongs in local files

Examples:

- remote server address -> SQLite
- remote UUID/password -> SQLite
- selected proxy listening port -> `config.toml`
- direct/block routing lists -> `config.toml`
- managed geo asset sources -> `config.toml`
- downloaded geo asset files -> local files under the XRAT config directory
- test history -> SQLite
- runtime session state -> SQLite

This boundary keeps the app easier to reason about.

## `config.toml` Discussion Areas

The first-pass schema should likely answer:

- what local SOCKS port to use
- what local HTTP port to use
- whether HTTP is enabled at all
- whether local SOCKS UDP and auth are enabled
- whether runtime sniffing is enabled
- what DNS or routing defaults should be applied
- how custom geo assets are sourced and stored
- whether test defaults such as probe target URL ever belong here

Possible sections:

- `[runtime]`
- `[routing]`
- `[geo]`
- `[dns]`
- `[testing]`
- `[paths]`

This phase does not need to lock every future key, but it should define a clean
starting structure.

## Routing And Geo Discussion Areas

The first-pass `config.toml` format should answer:

- which destinations go direct
- which destinations are blocked
- how domain, IP, geosite, and geoip entries are represented
- how custom geo profiles are named
- whether geo profile sources are local files or URLs
- where fetched geo assets are saved
- how invalid route or geo entries are handled

A practical first-pass format keeps the route lists explicit:

- `domain = []`
- `ip = []`
- `geosite = []`
- `geoip = []`

Custom geo profiles should live in a standalone `[geo]` section and use
`[[geo.profiles]]` entries with explicit names. If `save_dir` is omitted, XRAT
stores fetched files under:

- `CONFIG_PATH/geo/PROFILE_NAME/geosite.dat`
- `CONFIG_PATH/geo/PROFILE_NAME/geoip.dat`

## Path and Naming Decisions

Phase 3.5 sets the canonical names under the XRAT config directory.

Recommended target layout:

- `~/.config/xrat/db.sqlite`
- `~/.config/xrat/config.toml`
- `~/.config/xrat/geo/<profile>/geosite.dat`
- `~/.config/xrat/geo/<profile>/geoip.dat`

Decision:

- XRAT should use `config.toml` as the canonical app config file name
- the existing `Config.toml` path in code should be migrated to `config.toml`

Also keep:

- `XRAT_PATH` as a whole-directory override for development and tests

That is simpler than adding separate environment overrides per file.

## Validation and Error-Handling Questions

This phase should define the behavior for missing or invalid files.

Questions to settle:

- should missing `config.toml` be auto-created with defaults?
- should invalid TOML fail startup hard?
- should invalid route list entries fail startup hard?
- should missing geo assets be fetched, warned about, or ignored until used?

A likely first-pass choice:

- missing files -> create with safe defaults
- invalid `config.toml` -> fail with a clear error
- invalid route or geo entries -> either warn and skip or fail fast, but choose
  one policy consistently

## Relationship To Phase 4

Phase 4 should consume the outcomes of this phase directly.

Expected handoff:

- runtime config generation reads local ports and routing defaults from
  `config.toml`
- runtime routing generation reads direct/block rules from `config.toml`
- geo asset management reads local paths and URLs from `[geo]`
- selected imported node still comes from SQLite
- runtime session state still persists to SQLite

That means Phase 4 can focus on process lifecycle rather than policy design.

## Recommended Module Impact

Once implementation starts, likely touchpoints are:

- `src/app/path.rs`
- `src/app/runtime.rs`
- `src/xray/config.rs`
- a new app config loader module such as:
  - `src/app/config.rs`
  - or `src/support/config.rs`

There may also be a dedicated route/geo parser module if validation becomes more
than simple config deserialization.

## Suggested Delivery Order

1. Decide canonical naming and path layout
2. Define first-pass `config.toml` schema
3. Define first-pass routing and geo asset formats
4. Decide validation and file-creation rules
5. Wire those decisions into the Phase 4 runtime plan

## Success Criteria

Phase 3.5 is complete when:

- the local XRAT config directory layout is explicitly defined
- `config.toml` is the canonical app config file name
- the first-pass purpose of routing and geo configuration is clear
- the ownership boundary between local files and SQLite is documented
- Phase 4 can proceed without re-litigating the local configuration surface

## Open Questions

- should `config.toml` own testing defaults such as the real-delay probe URL?
- should planned testing fields such as `[testing.tcp].timeout` remain in the
  schema before config loading is wired into `xrat test`?
- should invalid route or geo entries fail startup or be skipped with warnings?
- should geo profile files be fetched only on demand or proactively when
  `auto_update` is enabled?
- should missing local geo files fail startup, fail only when referenced, or
  emit warnings?
