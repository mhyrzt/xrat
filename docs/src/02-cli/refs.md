# Stable Refs

xrat stores numeric database IDs internally, but user-facing commands also
accept stable short refs for configs and subscriptions.

Refs are random lowercase hex strings generated on insert. Human output shows
the first 8 characters by default:

```text
REF       STATUS          PROTO  ADDRESS              NAME
a1b2c3d4  enabled,active  vless  example.com:443      Main
```

You can use any unique prefix:

```bash
xrat connect a1b2
xrat show config a1b2c3d4
xrat test a1b2
xrat delete subscription f00d
```

Numeric IDs still work for compatibility:

```bash
xrat connect 42
```

If a prefix matches more than one row, xrat asks for more characters. If a
numeric string matches an existing numeric ID, the numeric ID wins; otherwise
xrat tries it as a ref prefix.

Commands that accept config refs include `connect`, `show config`, `enable`,
`disable`, `restore`, `delete config`, `test <id>`, and
`rotate now --config-id`.

Commands that accept subscription refs include `show subscription`,
`delete subscription`, `list configs --subscription`, and `test --subscription`.
