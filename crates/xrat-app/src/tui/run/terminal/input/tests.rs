use super::*;
use crossterm::event::KeyCode;

fn code(event: Option<TuiEvent>) -> KeyCode {
    match event {
        Some(TuiEvent::Key(key)) => key.code,
        other => panic!("expected key, got {other:?}"),
    }
}

#[test]
fn delivers_legacy_text_at_drained_input_boundary() {
    let now = Instant::now();
    let mut input = TerminalInput::default();
    input.feed("qسلام".as_bytes());
    input.settle(now);
    for character in "qسلام".chars() {
        assert_eq!(code(input.pop(now)), KeyCode::Char(character));
    }
    assert!(input.pop(now).is_none());
}

#[test]
fn lone_escape_expires_without_another_keystroke() {
    let now = Instant::now();
    let mut input = TerminalInput::default();
    input.feed(b"\x1b");
    input.settle(now);
    assert!(input.pop(now).is_none());
    assert_eq!(
        input.wait_duration(Duration::from_secs(1), now),
        ESCAPE_TIMEOUT
    );
    assert_eq!(code(input.pop(now + ESCAPE_TIMEOUT)), KeyCode::Esc);
    assert!(input.pop(now + ESCAPE_TIMEOUT).is_none());
}

#[test]
fn split_escape_sequence_survives_settling_and_timeout() {
    let now = Instant::now();
    let mut input = TerminalInput::default();
    input.feed(b"\x1b");
    input.settle(now);
    input.feed(b"[");
    input.settle(now);
    assert!(input.pop(now + ESCAPE_TIMEOUT).is_none());
    input.feed(b"A");
    input.settle(now + ESCAPE_TIMEOUT);
    assert_eq!(code(input.pop(now + ESCAPE_TIMEOUT)), KeyCode::Up);
}

#[test]
fn split_utf8_is_not_flushed_prematurely() {
    let now = Instant::now();
    let mut input = TerminalInput::default();
    let bytes = "ش".as_bytes();
    input.feed(&bytes[..1]);
    input.settle(now);
    assert!(input.pop(now + ESCAPE_TIMEOUT).is_none());
    input.feed(&bytes[1..]);
    input.settle(now);
    assert_eq!(code(input.pop(now)), KeyCode::Char('ش'));
}

#[test]
fn bracketed_paste_and_key_order_survive_chunk_boundaries() {
    let now = Instant::now();
    let mut input = TerminalInput::default();
    input.feed(b"q\x1b[200~hello");
    input.settle(now);
    assert_eq!(code(input.pop(now)), KeyCode::Char('q'));
    assert!(input.pop(now).is_none());
    input.feed(b" world\x1b[201~j");
    input.settle(now);
    assert_eq!(input.pop(now), Some(TuiEvent::Paste("hello world".into())));
    assert_eq!(code(input.pop(now)), KeyCode::Char('j'));
}

#[test]
fn kitty_release_events_do_not_trigger_actions() {
    let now = Instant::now();
    let mut input = TerminalInput::default();
    input.feed(b"\x1b[113;1:3u\x1b[106;1u");
    input.settle(now);
    assert_eq!(code(input.pop(now)), KeyCode::Char('j'));
    assert!(input.pop(now).is_none());
}

#[test]
fn decodes_sgr_mouse_click_and_scroll_events() {
    let now = Instant::now();
    let mut input = TerminalInput::default();
    // Click: CSI < 0 ; 10 ; 20 M (left press at col 10, row 20)
    // Scroll: CSI < 64 ; 10 ; 20 M (scroll up at col 10, row 20)
    // Scroll: CSI < 65 ; 10 ; 20 M (scroll down at col 10, row 20)
    input.feed(b"\x1b[<0;10;20M\x1b[<64;10;20M\x1b[<65;10;20M");
    input.settle(now);

    assert_eq!(
        input.pop(now),
        Some(TuiEvent::Mouse(TuiMouseEvent {
            kind: TuiMouseEventKind::Down(TuiMouseButton::Left),
            column: 9,
            row: 19,
            modifiers: crossterm::event::KeyModifiers::empty(),
        }))
    );

    assert_eq!(
        input.pop(now),
        Some(TuiEvent::Mouse(TuiMouseEvent {
            kind: TuiMouseEventKind::ScrollUp,
            column: 9,
            row: 19,
            modifiers: crossterm::event::KeyModifiers::empty(),
        }))
    );

    assert_eq!(
        input.pop(now),
        Some(TuiEvent::Mouse(TuiMouseEvent {
            kind: TuiMouseEventKind::ScrollDown,
            column: 9,
            row: 19,
            modifiers: crossterm::event::KeyModifiers::empty(),
        }))
    );
    assert!(input.pop(now).is_none());
}
