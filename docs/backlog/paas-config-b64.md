# Medium, P1: PaaS-friendly config via env var

### Status

Planned

### Goal

Allow passing the full `config.toml` as base64 so xrat can run on PaaS platforms
that cannot mount custom config files.

### Current behavior

Config loading assumes file-based lookup paths.

### Changes required

- Add `--config-b64` and/or `XRAT_CONFIG_B64` input.
- Decode and feed content into config loading before normal file resolution (or
  bypass filesystem lookup when explicit config content is provided).
- Document usage in deploy docs.

### Possible root cause

Config bootstrap is currently file-path-centric and has no raw-content input
channel.

### Verification

- Unit test decoding and parsing of known base64 config.
- Integration test with no config file on disk and `XRAT_CONFIG_B64` set.
