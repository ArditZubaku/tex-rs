use std::{
    cmp::min,
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

fn run_editor() {
    let tb = Termbox::init().unwrap_or_else(|err| {
        eprintln!("Could not init termbox - error: {err}");
        process::exit(1);
    });

    let mut e = Editor::new(tb);

    e.source_file = match env::args().nth(1) {
        Some(path) => {
            e.read_file(&path);
            path
        }
        None => {
            e.text_buf.push(vec![]);
            String::from("out.txt")
        }
    };

    e.rows -= 1;

    loop {
        if e.cols < 80 {
            e.cols = 80;
        }

        e.tb.clear(Attribute::DEFAULT, Attribute::DEFAULT);

        e.display_text_buffer();
        e.display_status_bar();

        e.tb.flush();

        match e.tb.poll_event().expect("Failed to poll event") {
            Event::Key {
                key: Key::Esc | Key::Char('q'),
                ..
            } => break,
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Mode {
    #[default]
    Normal,
    Insert,
    Visual,
}

struct Editor {
    tb: Termbox,
    // TODO: Maybe I can do a better job and use something smaller than usize?
    rows: usize,
    cols: usize,
    #[allow(dead_code)]
    offset_x: usize,
    #[allow(dead_code)]
    offset_y: usize,
    offset_row: usize,
    offset_col: usize,
    current_row: usize,
    current_col: usize,
    text_buf: Vec<Vec<char>>,
    undo_buf: Vec<Vec<char>>,
    copy_buf: Vec<char>,
    source_file: String,
    mode: Mode,
    modified: bool,
}

#[derive(Default)]
struct EditorStatus {
    mode: String,
    copy: String,
    undo: String,
    file: String,
    cursor: String,
}

impl Editor {
    fn new(tb: Termbox) -> Self {
        let (width, height) = tb.size();
        Self {
            tb: tb,
            cols: width,
            rows: height,
            offset_x: 0,
            offset_y: 0,
            offset_row: 0,
            offset_col: 0,
            current_row: 0,
            current_col: 0,
            text_buf: vec![],
            undo_buf: vec![],
            copy_buf: vec![],
            source_file: String::new(),
            mode: Mode::default(),
            modified: false,
        }
    }

    fn read_file(&mut self, path: &str) {
        // Rust automatically closes File when it goes out of scope,
        // handling file descriptor cleanup without needing an explicit defer block.
        let Ok(file) = File::open(path) else {
            self.source_file = path.to_string();
            self.text_buf.push(vec![]);
            return;
        };

        let reader = BufReader::new(file);
        let mut line_num = 0;

        for line in reader.lines() {
            match line {
                Ok(content) => {
                    self.text_buf.push(content.chars().collect());
                    line_num += 1;
                }
                Err(err) => {
                    eprintln!("Failed to read the file '{path}', err: {err}");
                    return;
                }
            }
        }

        if line_num == 0 {
            self.text_buf.push(vec![]);
        }
    }

    fn display_text_buffer(&mut self) {
        let buf_len = self.text_buf.len();

        for row in 0..self.rows {
            let text_buf_row = row + self.offset_row;

            // past end of buffer -> draw line indicator once per row
            if text_buf_row >= buf_len {
                self.tb
                    .set_cell(0, row, '*', Attribute::BLUE, Attribute::DEFAULT);
                continue;
            }

            let line = &self.text_buf[text_buf_row];
            let line_len = line.len();

            // render visible characters in the current row
            for col in 0..self.cols {
                let text_buf_col = col + self.offset_col;
                if text_buf_col >= line_len {
                    continue;
                }

                let ch = line[text_buf_col];
                if ch == '\t' {
                    self.tb
                        .set_cell(col, row, ' ', Attribute::DEFAULT, Attribute::GREEN);
                } else {
                    self.tb.set_char(col, row, ch);
                }
            }
        }
    }

    fn display_status_bar(&mut self) {
        let mut s = EditorStatus::default();

        if self.mode == Mode::Insert || self.mode == Mode::Visual {
            s.mode = " EDIT: ".to_string();
        } else {
            s.mode = " VIEW: ".to_string();
        }

        // truncate the file name
        let filename_len = min(self.source_file.len(), 20);
        let mut state = String::from("saved");
        if self.modified {
            state = String::from("modified");
        }

        s.file = format!(
            "{} - {} lines {}",
            &self.source_file[..filename_len],
            self.text_buf.len(),
            state
        );

        s.cursor = format!("Row {}, Col {}", self.current_row + 1, self.current_col + 1);

        if self.copy_buf.len() > 0 {
            s.copy = " [Copy]".to_string();
        }

        if self.undo_buf.len() > 0 {
            s.undo = " [Undo]".to_string();
        }

        let used_space = s.mode.len() + s.file.len() + s.copy.len() + s.undo.len();
        let spaces = " ".repeat(self.cols - used_space);
        let txt = format!(
            "{}{}{}{}{}{}",
            s.mode, s.file, s.copy, s.undo, spaces, s.cursor
        );

        self.print_message(0, self.rows, Attribute::BLACK, Attribute::WHITE, &txt);
    }

    fn print_message(
        &mut self,
        mut col: usize,
        row: usize,
        fg: Attribute,
        bg: Attribute,
        msg: &str,
    ) {
        for ch in msg.chars() {
            self.tb.set_cell(col, row, ch, fg, bg);
            col += ch.width().unwrap_or(0);
        }
    }
}
