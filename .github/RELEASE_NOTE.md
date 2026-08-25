## xrat v0.18.3

This patch release restores XHTTP links exported by clients and subscription
panels that append the inert legacy parameter `headerType=none`.

### XHTTP link compatibility

- **Accept neutral legacy defaults.** XHTTP links with an absent, empty, or
  `none` header type now generate identical runtime transport settings.
- **Preserve strict validation.** Non-neutral `headerType` values remain
  rejected for XHTTP because the field belongs to other transports. Future
  XHTTP fields must still be carried in the URL-encoded JSON `extra` parameter.
- **Cover the compatibility boundary.** Regression tests verify both accepted
  neutral values and rejected non-neutral values.

### Upgrade notes

- No database migration or manual configuration change is required.
- Users seeing `unsupported link parameter "headerType" for transport "xhttp"`
  should upgrade XRAT; existing imported configurations do not need to be
  re-imported.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.18.2...v0.18.3
