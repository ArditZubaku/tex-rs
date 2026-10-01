use std::process;
use termbox_rs::{Attribute, Event, Key, Termbox};
use unicode_width::UnicodeWidthChar;

fn main() {
    run_editor();
}

fn run_editor() {
    let mut tb = Termbox::init().unwrap_or_else(|err| {
        eprintln!("Could not init termbox - error: {err}");
        process::exit(1);
    });

    let msg = "TEX - a bare bones text editor";
    let (width, height) = tb.size();

    let x = (width - msg.len()) / 2;
    let y = height / 2;

    loop {
        print_message(&mut tb, x, y, Attribute::DEFAULT, Attribute::DEFAULT, msg);
        tb.flush();

        match tb.poll_event().expect("Failed to poll event") {
            Event::Key { key: Key::Esc, .. } => break,
            _ => {}
        }
    }
}

fn print_message(
    tb: &mut Termbox,
    mut col: usize,
    row: usize,
    fg: Attribute,
    bg: Attribute,
    msg: &str,
) {
    for ch in msg.chars() {
        tb.set_cell(col, row, ch, fg, bg);
        col += ch.width().unwrap_or(0);
    }
}
