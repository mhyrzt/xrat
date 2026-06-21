# Plugin System Backlog

## Summary

`xrat` can support a plugin system, but the first version should be narrow and
process-based. Plugins should not become arbitrary Rust code loaded into the
core process. Instead, a plugin should be an optional external capability
provider that the application core starts, configures, observes, and stops.

A practical definition:

> A plugin is an optional external capability provider that can transform
> configs, start sidecar processes, expose local proxy endpoints, run
> probes/scanners, or contribute runtime health/status while the core app owns
> config storage, lifecycle, daemon control, logs, and UI/API presentation.

This model fits sidecar-style tools such as SNI spoofing forwarders and
domain-fronting helpers while preserving the desired architecture: one shared
application core with thin CLI, TUI, Axum, and daemon adapters.

## Reference Use Cases

### SNI spoofing sidecar

Reference project:

- `https://github.com/therealaleph/sni-spoofing-rust`

This is best modeled as a runtime sidecar plugin. The plugin can start a local
forwarder that accepts traffic from Xray or another runtime, injects or spoofs
TLS/SNI behavior for Cloudflare IPs, and relays traffic upstream.

Possible plugin inputs:

- selected config id or normalized runtime config
- target Cloudflare IP
- fake SNI / front domain
- local listen host and port
- timeout and health-check settings

Possible plugin outputs:

- local endpoint to connect through
- process id
- rewritten runtime target
- health status
- logs/events
- privilege or platform requirements

`xrat` should own the persisted config, runtime session, daemon lifecycle, and
event records. The plugin should only provide the sidecar behavior and report
structured status.

### MITM / domain-fronting helper

Reference project:

- `https://github.com/patterniha/MITM-DomainFronting`

This is best modeled as a config/template plus optional sidecar plugin. It may
generate or adapt runtime configuration for domain fronting and may require
certificate/key setup. Because locally trusted certificates or private keys can
be sensitive, this plugin type should be treated as high risk and require
explicit user confirmation.

Possible plugin inputs:

- selected config or raw profile template
- front domain
- origin domain
- certificate/key paths when required
- local listen endpoint

Possible plugin outputs:

- generated runtime config fragment
- setup warnings
- local endpoint
- health status
- required manual steps

## Recommended V1 Architecture

Prefer a process-based plugin interface:

- Plugin ships a binary and a manifest, for example `plugin.toml`.
- `xrat` starts the plugin as a child process or connects to a local plugin
  socket.
- Communication uses JSON over stdio or a local socket.
- The plugin declares capabilities and required permissions up front.
- The plugin never mutates the database directly.
- The plugin returns structured results to the application core.
- The daemon owns long-running plugin process lifecycle.
- CLI, TUI, and HTTP adapters only call shared application plugin services.

Avoid these in v1:

- Rust dynamic library plugins through `dlopen`.
- Plugins that receive raw database handles.
- Plugins that directly write runtime session records.
- Plugins that directly own CLI/TUI/HTTP presentation.
- Unscoped arbitrary shell hooks.

## Manifest Shape

A minimal plugin manifest could contain:

```toml
[plugin]
id = "sni-spoof"
name = "SNI Spoof Sidecar"
version = "0.1.0"
binary = "sni-spoof-plugin"

[capabilities]
runtime_sidecar = true
config_transform = true
scanner = false
health = true

[permissions]
network_listen = true
network_connect = true
spawn_process = true
filesystem_read = []
filesystem_write = []
requires_root = false

[protocol]
transport = "stdio-json"
version = 1
```

The exact schema can evolve, but the important rule is that capabilities and
permissions are explicit and visible before a plugin runs.

## Plugin Capability Types

### Runtime sidecar plugin

Starts and manages a helper process that exposes a local endpoint. Examples: SNI
spoofing forwarder, domain-fronting sidecar, custom tunnel helper.

Core responsibilities:

- select config
- decide runtime session lifecycle
- start/stop plugin via daemon
- record events
- generate final runtime config

Plugin responsibilities:

- start local listener
- connect to upstream target
- expose health/status
- provide logs

### Config transform plugin

Transforms an app-level config into a runtime-ready fragment or patch.

Examples:

- replace upstream target with a local sidecar endpoint
- add domain-fronting metadata
- generate Xray/sing-box fragments from plugin-specific settings

The plugin should return a structured patch, not directly write files.

### Scanner plugin

Runs discovery or probing for candidate front domains, fake SNI values,
Cloudflare IPs, or other transport-specific targets.

The core should persist accepted scan results. The plugin should return
structured candidates and probe results.

### Health plugin

Reports health for a plugin-owned sidecar or transport.

Examples:

- local listener reachable
- upstream handshake succeeded
- last error
- active connection count

### Import/parser plugin

Supports extra import formats or external profile templates.

The plugin should return normalized nodes or application-level config data, not
database rows.

## Application Boundaries

The core application should own:

- config persistence
- subscription persistence
- runtime session records
- daemon lifecycle
- event/log records
- config selection and lifecycle state
- final runtime config generation
- CLI/TUI/API presentation

Plugins may own:

- sidecar process behavior
- transport-specific probing
- transport-specific config transforms
- plugin-specific health details
- plugin-specific logs

Plugins must not own:

- database mutation
- selected/active config state
- daemon supervision policy
- user-facing command output
- HTTP route definitions
- TUI screens

## Suggested Application Modules

Possible new modules:

- `src/app/plugins/manifest.rs`
- `src/app/plugins/registry.rs`
- `src/app/plugins/protocol.rs`
- `src/app/plugins/service.rs`
- `src/app/plugins/runtime_sidecar.rs`
- `src/app/plugins/config_transform.rs`
- `src/app/plugins/health.rs`
- `src/app/daemon/supervisor/handlers/plugins.rs`

Possible adapter modules:

- `src/cli/plugins.rs`
- `src/app/commands/plugins.rs`
- `src/tui/run/tasks/plugins.rs`
- `src/server/routes/plugins.rs`

The application service should be introduced before adapter commands. Avoid
building a CLI-only plugin system.

## Suggested Commands

Possible CLI surface:

```text
xrat plugin list
xrat plugin show <plugin-id>
xrat plugin enable <plugin-id>
xrat plugin disable <plugin-id>
xrat plugin run <plugin-id> --config <config-id>
xrat plugin health <plugin-id>
xrat plugin scan <plugin-id>
```

For v1, prefer read-only and explicit run/status commands before automatic
runtime integration.

## Observability Requirements

Plugin operations should record structured events through the existing event
system where useful:

- plugin discovered
- plugin enabled or disabled
- plugin sidecar started
- plugin sidecar stopped
- plugin health failed
- plugin transform failed
- plugin scan completed

Tracing should include:

- plugin id
- plugin version
- capability
- config id when applicable
- runtime session id when applicable
- sidecar pid when applicable

Plugin stdout/stderr should be captured or tailed in a bounded way so failed
sidecars are debuggable without flooding logs.

## Security And Safety

Plugin support increases risk because plugins are executable code. V1 should
make trust boundaries explicit:

- require explicit installation or enablement
- show manifest permissions before first run
- deny undeclared capabilities
- avoid arbitrary shell hooks
- avoid database handles in plugin processes
- sandbox later if practical
- warn when a plugin requires root, packet privileges, certificate trust, or
  private keys

MITM/domain-fronting plugins should be marked high risk when they require local
trusted certificates or private key material.

## Testing Strategy

Add tests at three levels:

- Manifest parsing and validation tests.
- Protocol tests using a fake stdio/socket plugin.
- Application service tests with fake plugin runners.

Integration tests should cover:

- plugin discovery
- rejected invalid manifest
- capability mismatch
- sidecar start/stop lifecycle
- config transform result applied to runtime generation
- plugin failure recorded as a structured event

CLI/TUI/HTTP tests should only verify adapter translation and presentation, not
plugin business logic.

## Incremental Rollout Plan

1. Add manifest parsing and plugin registry with no execution.
2. Add process-based plugin runner with JSON stdio protocol.
3. Add health capability and fake-plugin tests.
4. Add runtime sidecar capability behind an explicit command.
5. Add config transform capability for runtime generation.
6. Add daemon supervision for long-running plugin sidecars.
7. Add TUI/API visibility after the application service is stable.

## Open Decisions

- Whether plugin communication should start with stdio JSON or Unix/local TCP
  sockets.
- Whether plugins are installed from local paths only or later from a registry.
- Whether plugin config is stored in app config, database tables, or both.
- Whether automatic sidecar startup is allowed on `xrat connect` in v1.
- Whether plugin sandboxing is required before public release.
