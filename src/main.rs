use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
    process,
};
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
    source_file: String,
}

impl State {
    fn new() -> Self {
        Self {
            rows: 0,
            cols: 0,
            offset_x: 0,
            offset_y: 0,
            text_buf: vec![],
            source_file: String::new(),
        }
    }
}

fn run_editor() {
    let mut tb = Termbox::init().unwrap_or_else(|err| {
        eprintln!("Could not init termbox - error: {err}");
        process::exit(1);
    });

    let mut state = State::new();

    state.source_file = match env::args().nth(1) {
        Some(path) => {
            read_file(&mut state, &path);
            path
        }
        None => {
            state.text_buf.push(vec![]);
            String::from("out.txt")
        }
    };

    loop {
        (state.rows, state.cols) = tb.size();
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

fn read_file(s: &mut State, path: &str) {
    // Rust automatically closes File when it goes out of scope,
    // handling file descriptor cleanup without needing an explicit defer block.
    let Ok(file) = File::open(path) else {
        s.source_file = path.to_string();
        s.text_buf.push(vec![]);
        return;
    };

    let reader = BufReader::new(file);
    let mut line_num = 0;

    for line in reader.lines() {
        match line {
            Ok(content) => {
                s.text_buf.push(content.chars().collect());
                line_num += 1;
            }
            Err(err) => {
                eprintln!("Failed to read the file '{path}', err: {err}");
                return;
            }
        }
    }

    if line_num == 0 {
        s.text_buf.push(vec![]);
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
