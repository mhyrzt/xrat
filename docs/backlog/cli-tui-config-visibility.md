# Medium, P2: Config metrics and location visibility in CLI/TUI

### Status

Draft

### Scope

Config list/cards presentation in CLI and TUI.

### Items

- Show intervals in human-readable form in CLI/TUI outputs unless raw format is
  explicitly requested.
- In config table/cards, show enabled probe metrics beyond real delay (ICMP,
  TCP, download, upload) when available.
- Add MMDB-derived location fields in config cards/table (ASN, country, city).

### Possible root causes

- Existing rendering paths likely hardcode `real_delay` as primary metric and do
  not dynamically map enabled probe dimensions.
- Duration formatting may expose raw seconds/milliseconds from storage without a
  shared display formatter.
- MMDB enrichment data may exist in model/repository layers but is not threaded
  into list/card view adapters.

### Changes required

- Reuse/extend shared output formatter for durations across CLI and TUI layers.
- Make metric columns/cards conditional on configured test types and data
  availability.
- Thread MMDB metadata into list/query adapters and surface in output schema.

### Verification

- CLI parser/output tests for formatting and dynamic columns.
- TUI rendering checks for cards/tables with mixed metric availability.
- Snapshot or integration tests confirming ASN/country/city visibility.
