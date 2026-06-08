# 09.9 Low, P3: BSD clipboard (arboard)

**Difficulty:** Low — negligible change.

`arboard` already handles Linux (X11/Wayland) and macOS (objc2). On
FreeBSD/OpenBSD running X11, `arboard` falls back to X11 clipboard if the
`x11rb` feature is enabled. Likely works out of the box. Test and confirm.
