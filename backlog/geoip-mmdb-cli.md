# Medium, P2: GeoIP/MMDB CLI naming and UX improvements

### Status

Draft

### Scope

CLI naming clarity, backend visibility, and download progress rendering.

### Items

- Consider renaming namespace from `xrat geoip ...` to `xrat mmdb ...` (or
  another clearer term) to reduce confusion with Xray GeoIP semantics.
- Improve `xrat geoip backend` output readability and structure.
- During downloads, show source repository/URL being used.
- Fix `xrat geoip download --all --force` progress rendering where lines flicker
  and swap between databases.

### Possible root causes

- Command naming predates expanded MMDB-focused functionality and now overlaps
  conceptually with Xray-native GeoIP terms.
- Backend output currently optimized for compactness rather than operator
  diagnostics.
- Progress renderer likely reuses a single terminal line for multi-task updates
  without stable row allocation per file.

### Changes required

- Decide final command vocabulary and compatibility strategy (aliases,
  deprecation, docs).
- Redesign backend output fields with consistent labels and clearer units.
- Include source endpoint/provider in download logs/progress header.
- Update progress display to stable multi-line or single-active-task format.

### Verification

- CLI snapshot tests for backend/help output.
- Manual/automated progress test with concurrent multi-file downloads.
- Confirm old command aliases (if retained) behave and warn correctly.
