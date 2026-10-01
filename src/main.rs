use std::process;
use termbox_rs::{Attribute, Event, Key, Termbox};
use unicode_width::UnicodeWidthChar;

fn main() {
    run_editor();
}

struct State {
    rows: usize,
    cols: usize,
    offset_x: usize,
    offset_y: usize,
    text_buf: Vec<Vec<char>>,
}

impl State {
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            offset_x: 0,
            offset_y: 0,
            text_buf: vec![],
        }
    }
}

fn run_editor() {
    let mut tb = Termbox::init().unwrap_or_else(|err| {
        eprintln!("Could not init termbox - error: {err}");
        process::exit(1);
    });

    loop {
        let (width, height) = tb.size();

        let mut state = State::new(height, width);
        state.text_buf.push(vec!['h', 'e', 'l', 'l', 'o']);
        state.text_buf.push(vec!['w', 'o', 'r', 'l', 'd']);
        state.rows -= 1;

        if state.cols < 80 {
            state.cols = 80;
        }

        tb.clear(Attribute::DEFAULT, Attribute::DEFAULT);

        display_text_buffer(&mut tb, &state);

        tb.flush();

        match tb.poll_event().expect("Failed to poll event") {
            Event::Key {
                key: Key::Esc | Key::Char('q'),
                ..
            } => break,
            _ => {}
        }
    }
}

#[allow(dead_code)]
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

fn display_text_buffer(tb: &mut Termbox, s: &State) {
    let buf_len = s.text_buf.len();

    for row in 0..s.rows {
        let text_buf_row = row + s.offset_y;

        // past end of buffer -> draw line indicator once per row
        if text_buf_row >= buf_len {
            tb.set_cell(0, row, '*', Attribute::BLUE, Attribute::DEFAULT);
            continue;
        }

        let line = &s.text_buf[text_buf_row];
        let line_len = line.len();

        // render visible characters in the current row
        for col in 0..s.cols {
            let text_buf_col = col + s.offset_x;
            if text_buf_col >= line_len {
                continue;
            }

            let ch = line[text_buf_col];
            if ch == '\t' {
                tb.set_cell(col, row, ' ', Attribute::DEFAULT, Attribute::GREEN);
            } else {
                tb.set_char(col, row, ch);
            }
        }
    }
}
