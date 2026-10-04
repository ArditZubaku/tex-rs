mod editor;

use std::{env, process};
use termbox_rs::{Attribute, Event, Key, Termbox};

use crate::editor::Editor;

fn main() {
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

        e.scroll_text_buffer();
        e.display_text_buffer();
        e.display_status_bar();
        e.sync_cursor();

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
