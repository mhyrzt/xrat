# db

Inspect and maintain the XRAT database.

```bash
xrat db <action>
```

## Actions

| Action    | Description                                        |
| --------- | -------------------------------------------------- |
| `migrate` | Apply any pending database migrations and report   |

---

## db migrate

```bash
xrat db migrate
```

Applies any pending schema migrations and confirms the database is up to date.

Migrations normally run automatically on the first command after an upgrade and
during `xrat upgrade` itself. This command makes that step explicit, which is
useful for:

- Verifying the database is current after a manual binary swap.
- Surfacing a migration error on demand with actionable context.

### Output

```
OK Database migrations are up to date.
```

### Migration errors

If a migration fails, the error names the migration version, the likely cause,
and recovery guidance. Common cases:

- **Checksum mismatch** — a previously shipped migration file was edited after
  release. Restore the original migration (reinstall the matching release) or
  reset the database from a backup.
- **Dirty / partially applied** — inspect the `_sqlx_migrations` table, finish or
  revert the offending migration by hand, and remove its row before retrying.
- **Missing migration** — the database records a migration this build does not
  contain, usually after a downgrade. Upgrade back to a build that includes it.

> **Contributor policy:** never edit a migration that has already shipped in a
> release. sqlx stores a per-migration checksum in `_sqlx_migrations`; editing a
> released migration changes its embedded checksum and breaks upgrades for
> existing databases. Always add a new ordered migration instead.

## Related

- [upgrade](upgrade.md) — runs migrations as part of self-upgrade
- [init](init.md) — create the database before first use
