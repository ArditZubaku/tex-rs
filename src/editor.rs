use std::{
    cmp::min,
    fs::File,
    io::{BufRead, BufReader},
};

use termbox_rs::{Attribute, Event, Key, Termbox};
use unicode_width::UnicodeWidthChar;

pub struct Editor {
    pub tb: Termbox,
    // TODO: Maybe I can do a better job and use something smaller than usize?
    pub rows: usize,
    pub cols: usize,
    pub text_buf: Vec<Vec<char>>,
    pub source_file: String,
    #[allow(dead_code)]
    offset_x: usize,
    #[allow(dead_code)]
    offset_y: usize,
    offset_row: usize,
    offset_col: usize,
    current_row: usize,
    current_col: usize,
    undo_buf: Vec<Vec<char>>,
    copy_buf: Vec<char>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Mode {
    #[default]
    Normal,
    Insert,
    Visual,
}

impl Editor {
    pub fn new(tb: Termbox) -> Self {
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

    pub fn read_file(&mut self, path: &str) {
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

    pub fn display_text_buffer(&mut self) {
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

    pub fn display_status_bar(&mut self) {
        let mut s = EditorStatus::default();

        if self.mode == Mode::Insert || self.mode == Mode::Visual {
            s.mode = " EDIT: ".to_string();
        } else {
            s.mode = " VIEW: ".to_string();
        }

        // truncate the file name
        let filename_len = min(self.source_file.len(), 20);
        let state = if self.modified { "modified" } else { "saved" };

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

    pub fn scroll_text_buffer(&mut self) {
        if self.current_row < self.offset_row {
            self.offset_row = self.current_row
        }

        if self.current_col < self.offset_col {
            self.offset_col = self.current_col
        }

        if self.current_row >= self.offset_row + self.rows {
            self.offset_row = self.current_row - self.rows + 1
        }

        if self.current_col >= self.offset_col + self.cols {
            self.offset_col = self.current_col - self.cols + 1
        }
    }

    pub fn sync_cursor(&self) {
        self.tb.set_cursor(
            (self.current_col - self.offset_col) as i32,
            (self.current_row - self.offset_row) as i32,
        );
    }

    pub fn get_key(&self) -> Key {
        let res = self.tb.poll_event().expect("Failed to poll event");

        if let Event::Key { key, .. } = res {
            key
        } else {
            // TODO: Will think of something better in such a case
            panic!()
        }
    }

    pub fn process_key_press(&mut self, key: Key) {
        if key.ch() != Some('\0') {
            // char is not null
            if let Some(ch) = key.ch() {
                match ch {
                    'h' => self.left(),
                    'j' => self.down(),
                    'k' => self.up(),
                    'l' => self.right(),
                    // idk how many lines VIM goes up but this should be good
                    'u' => self.current_row = self.current_row.saturating_sub(15),
                    'd' => {
                        if self.current_row < self.text_buf.len() - 1 {
                            self.current_row += 15;
                        }
                    }
                    _ => {
                        println!("Unhandled character: {:?}", key);
                    }
                }
            } else {
                eprintln!("No character received?")
            }
        } else {
            match key {
                Key::ArrowUp => {
                    self.up();
                }
                Key::ArrowDown => {
                    self.down();
                }
                Key::ArrowLeft => {
                    self.left();
                }
                Key::ArrowRight => {
                    self.right();
                }
                _ => {
                    println!("You are not handling this key - {:?}", key)
                }
            }
        }
    }

    fn clamp_cursor(&mut self) {
        if let Some(line) = self.text_buf.get(self.current_row) {
            self.current_col = min(self.current_col, line.len());
        }
    }

    fn up(&mut self) {
        if self.current_row != 0 {
            self.current_row -= 1;
            self.clamp_cursor();
        }
    }

    fn down(&mut self) {
        if self.current_row < self.text_buf.len() - 1 {
            self.current_row += 1;
            self.clamp_cursor();
        }
    }

    fn left(&mut self) {
        if self.current_col != 0 {
            self.current_col -= 1;
        } else if self.current_row > 0 {
            // if we are not on the first line
            // move to the end of the previous line
            self.current_row -= 1;
            self.current_col = self.text_buf[self.current_row].len();
        }
    }

    fn right(&mut self) {
        let line_len = self.text_buf[self.current_row].len();
        if self.current_col < line_len {
            self.current_col += 1;
        } else if self.current_row < self.text_buf.len() - 1 {
            // if we are not on the last line
            // move to the first column of a the new line
            self.current_col = 0;
            self.current_row += 1;
        }
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
