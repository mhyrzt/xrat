Output docs/src/media/gif/tui.gif

Source docs/src/media/tapes/base.tape

Hide
Type "xratui"
Enter
Sleep 3s
Show

# Configs tab: browse rows and cycle panels.
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 500ms
Up
Sleep 1s

Type "2"
Sleep 1.5s

# Log panel sub-tabs: XratEvents (default) -> ProxyEngine -> Stats.
Type "]"
Sleep 1.5s
Type "]"
Sleep 2s

Type "3"
Sleep 1.5s
Type "4"
Sleep 1.5s
Type "1"
Sleep 1s

# Subscriptions tab: select the third row (first real subscription).
Type "]"
Sleep 1s
Down
Sleep 500ms
Down
Sleep 1.5s

# Back to Configs, filtered to that subscription: test the current view.
Type "["
Sleep 1s
Type "tv"
Sleep 4s

# API address QR.
Type "aq"
Sleep 3s
Escape
Sleep 500ms

# Help overlay.
Type "?"
Sleep 3s
Escape
Sleep 500ms

Type "q"
Sleep 1s
