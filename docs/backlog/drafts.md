# UI Draft Notes

## Log tab renaming

- `xrat event` → `events`
- `proxy engine` → `engine`
- `stats` → `traffic`

## Stats & Traffic tabs layout

Two rows in the traffic tab:

### Row 1: Textual stats

Two sub-tables:

**1a. Current traffic — columns: Download | Upload**

- current download/upload rate
- cumulative sum of all traffic

**1b. Probing/testing results — columns: Metric | Value**

- one row per active profile from `config.toml` (probing/testing section)
  - current probe/test value (interval-driven)
  - stats expressed as `mean ± std`
- cumulative failure/block stats (packet loss) — optional

### Row 2: Graphs (takes most of the height)

**2a. Traffic graph (redesign)**

- single graph replacing current separate download/upload graphs
- download: downward bars with distinct color
- upload: upward bars with distinct color
- bar height encodes traffic volume
- Y-axis labeled in KB/MB with grid lines for readability
- optional: mark packet loss/failures on X-axis with red `x`

**2b. Historical delay graph (optional)**

- continuous line with markers (`─o─` style, matplotlib-like)
