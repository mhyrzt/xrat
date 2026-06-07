# update

Refresh stored subscriptions.

```bash
xrat update [SUBS_REF...]
```

If no refs are provided, xrat refreshes all subscriptions with stored source
values. If refs are provided, xrat refreshes only matching subscriptions
(numeric IDs and stable ref prefixes are both accepted).

## Examples

Refresh all subscriptions:

```bash
xrat update
```

Refresh selected subscriptions:

```bash
xrat update 7 feedbeef
```
