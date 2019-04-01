use std::cmp;
use std::env;
use std::fs::File;
use std::io::prelude::*;

extern crate jwin;
extern crate jvec;

use jwin::{Code, Event, Win};
use jvec::JVec;


fn pad(mut string: String, n: usize) -> String {
    if string.len() < n {
        for _ in string.len()..n {
            string.insert(0, ' ');
        }
    }

    string
}

struct JEdit {
    win: Win,

    buffer: JVec<JVec<char>>,
    width: usize, height: usize,
    buffer_width: usize, buffer_height: usize,

    cursor_x: usize,
    cursor_y: usize,

    view_x: usize,
    view_y: usize,

    // line number offset
    offset_x: usize
}

impl JEdit {
    pub fn set_str(&mut self, string: &str) {
        self.buffer = JVec::new();
        self.handle_str(string);
        self.move_cursor(0, 0);
    }

    fn handle_str(&mut self, string: &str) {
        for chr in string.chars() {
            match chr {
                '\n' => {
                    let mut empty = JVec::new();
                    let line = self.buffer[self.cursor_y].as_mut().unwrap_or(&mut empty);
                    let mut new_line = JVec::new();
                    while line.len() != self.cursor_x {
                        new_line.push(line.remove(self.cursor_x));
                    }
                    self.buffer.insert(self.cursor_y + 1, new_line);
                    self.move_cursor(0, self.cursor_y + 1);
                },
                '\t' => {
                    let line_maybe = &mut self.buffer[self.cursor_y];
                    if line_maybe.is_none() {
                        *line_maybe = Some(JVec::new());
                    }
                    let line = line_maybe.as_mut().unwrap();
                    let mut new_x = self.cursor_x;
                    for _ in 0..2 {
                        line.insert(self.cursor_x, ' ');
                        new_x += 1;
                    }
                    self.move_cursor(new_x, self.cursor_y);
                },
                _ => {
                    let line_maybe = &mut self.buffer[self.cursor_y];
                    if line_maybe.is_none() {
                        *line_maybe = Some(JVec::new());
                    }
                    let line = line_maybe.as_mut().unwrap();
                    line.insert(self.cursor_x, chr);
                    self.move_cursor(self.cursor_x + 1, self.cursor_y);
                }
            }
        }
    }

    fn move_cursor(&mut self, x: usize, y: usize) {
        self.cursor_x = x;
        self.cursor_y = y;

        // cursor_x fix non paging
        if self.cursor_x < (self.buffer_width / 4) + self.view_x {
            if self.cursor_x >= self.buffer_width / 4 {
                self.view_x = self.cursor_x - (self.buffer_width / 4);
            } else {
                self.view_x = 0;
            }
        }

        if self.cursor_x >= (3 * self.buffer_width / 4) + self.view_x {
            if self.cursor_x >= (3 * self.buffer_width / 4) {
                self.view_x = self.cursor_x - (3 * self.buffer_width / 4);
            } else {
                if self.buffer_width >= 1 {
                    self.view_x = self.buffer_width - 1;
                }
            }
        }

        // cursor_y fix non paging
        if self.cursor_y < (self.buffer_height / 4) + self.view_y {
            if self.cursor_y >= self.buffer_height / 4 {
                self.view_y = self.cursor_y - (self.buffer_height / 4);
            } else {
                self.view_y = 0;
            }
        }

        if self.cursor_y >= (3 * self.buffer_height / 4) + self.view_y {
            if self.cursor_y >= (3 * self.buffer_height / 4) {
                self.view_y = self.cursor_y - (3 * self.buffer_height / 4);
            } else {
                if self.buffer_height >= 1 {
                    self.view_y = self.buffer_height - 1;
                }
            }
        }

        self.redraw();
    }

    // must call after you clear the line
    fn redraw_line(&mut self, y: usize) {
        let maybe = self.buffer[y + self.view_y].clone();
        if maybe.is_some() {
            let string: String = maybe.unwrap().into_iter().skip(self.view_x).map(|x| x.unwrap_or(' ')).collect();
            self.win.put_str(self.offset_x, y, string.as_str(), 0);
        }
    }

    fn redraw(&mut self) {
        let end = cmp::min(self.buffer_height + self.view_y, self.buffer.len());
        let line_range = self.view_y..end;

        self.offset_x = if end == 0 {
            0
        } else {
            format!("{}", end - 1).len() // good logarithm
        };
        self.offset_x += 1;
        if self.width >= self.offset_x {
            self.buffer_width = self.width - self.offset_x;
        } else {
            self.buffer_width = 0;
        }

        for y in 0..self.buffer_height {
            self.win.put_str(0, y, " ".repeat(self.width).as_str(), 0);

            if line_range.contains(&(y + self.view_y)) {
                self.win.put_str(0, y, pad(format!("{}", y + self.view_y), self.offset_x - 1).as_str(), 1);

                self.redraw_line(y);
            }
        }
        
        if self.cursor_y >= self.view_y {
            self.win.put_str(self.cursor_x + self.offset_x - self.view_x, self.cursor_y - self.view_y, "|", 1);
        }
    }

    fn run(&mut self) {
        loop {
            match self.win.poll() {
                // redraw event
                Some(Event::Redraw(w, h)) => {
                    self.width = w;
                    self.height = h;
                    self.buffer_height = h;

                    self.redraw();
                },
                // key events
                Some(Event::Key(Code::Showable(string))) => {
                    self.handle_str(&string);

                    self.redraw();
                },

                // special key
                Some(Event::Key(Code::Backspace)) => {
                    if self.cursor_x == 0 {
                        if self.cursor_y != 0 {
                            self.move_cursor(self.buffer[self.cursor_y - 1].as_ref().unwrap_or(&JVec::new()).len(), self.cursor_y - 1);

                            let line_maybe = self.buffer.remove(self.cursor_y + 1);
                            let new_line_maybe = &mut self.buffer[self.cursor_y];

                            if new_line_maybe.is_none() || line_maybe.is_none() {
                                self.redraw();
                                continue;
                            }
                            let new_line = new_line_maybe.as_mut().unwrap();
                            let line = line_maybe.unwrap();
                            for chr_maybe in line {
                                if chr_maybe.is_some() {
                                    let chr = chr_maybe.unwrap();
                                    new_line.push(Some(chr));
                                }
                            }
                        }
                    } else {
                        self.buffer[self.cursor_y].as_mut().unwrap().remove(self.cursor_x - 1);
                        self.move_cursor(self.cursor_x - 1, self.cursor_y);
                    }

                    self.redraw();
                },

                // moving the cursor with arrow keys
                Some(Event::Key(Code::Left)) => {
                    if self.cursor_x != 0 {
                        self.move_cursor(self.cursor_x - 1, self.cursor_y);
                    }
                },
                Some(Event::Key(Code::Right)) => {
                    let len = self.buffer[self.cursor_y].as_ref().unwrap_or(&JVec::new()).len();
                    self.move_cursor(cmp::min(self.cursor_x + 1, len), self.cursor_y);
                },
                Some(Event::Key(Code::Up)) => {
                    if self.cursor_y != 0 {
                        let len = self.buffer[self.cursor_y - 1].as_ref().unwrap_or(&JVec::new()).len();
                        self.move_cursor(cmp::min(self.cursor_x, len), self.cursor_y - 1);
                    }
                },
                Some(Event::Key(Code::Down)) => {
                    if self.buffer.len() > self.cursor_y + 1 {
                        let len = self.buffer[self.cursor_y + 1].as_ref().unwrap_or(&JVec::new()).len();
                        self.move_cursor(cmp::min(self.cursor_x, len), cmp::min(self.cursor_y + 1, self.buffer.len()));
                    }
                }

                Some(Event::Close) => break,
                _ => ()
            }
        }

        for line in self.buffer.iter() {
            match line {
                Some(line) => {
                    let string: String = line.iter().map(|x| x.unwrap_or(' ')).collect();
                    println!("{}", string);
                },
                None => {
                    println!();
                }
            }
        }
    }
}

fn main() {
    let mut jedit = JEdit {
        win: Win::new(String::from("jedit")).unwrap(),

        buffer: JVec::new(),
        width: 0, height: 0,
        buffer_width: 0, buffer_height: 0,

        cursor_x: 0,
        cursor_y: 0,

        view_x: 0,
        view_y: 0,

        offset_x: 0
    };

    let path_maybe = env::args().nth(1);
    if path_maybe.is_none() {
        jedit.run();

        return;
    }
    let path = path_maybe.unwrap();

    let file_maybe = File::open(path);
    assert!(file_maybe.is_ok(), "invalid file");
    let mut file = file_maybe.ok().unwrap();
    let mut string = String::new();
    let res = file.read_to_string(&mut string);
    assert!(res.is_ok(), "couldn't read from file");

    jedit.set_str(string.as_str());

    jedit.run();
}
