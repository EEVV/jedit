use std::cmp;
use std::env;
use std::fs::File;
use std::io::prelude::*;

extern crate jwin;
extern crate jvec;

use jwin::{Code, Event, Win};
use jvec::JVec;

mod buffer;
mod syntax;

use crate::buffer::{Buffer, Char};

// configs
pub const TAB_SIZE: usize = 4;

fn pad(mut string: String, n: usize) -> String {
    if string.len() < n {
        for _ in string.len()..n {
            string.insert(0, ' ');
        }
    }

    string
}

// cursor struct to handle
// cursor specific things
#[derive(Copy, Clone, Debug)]
struct Cursor {
    x: usize, y: usize,
    fake_x: usize
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            x: 0, y: 0,
            fake_x: 0
        }
    }
}

struct JEdit {
    win: Win,

    buffer: Buffer,

    width: usize, height: usize,
    buffer_width: usize, buffer_height: usize,

    cursor: Cursor,

    view_x: usize,
    view_y: usize,

    // line number offset
    offset_x: usize
}

impl JEdit {
    pub fn set_str(&mut self, string: &str) {
        self.buffer = Buffer::new(Some(Box::new(syntax::Rust::new())));
        self.handle_str(string);
        self.move_cursor(0, 0);
    }

    fn move_cursor(&mut self, x: usize, y: usize) {
        self.cursor.x = x;
        self.cursor.y = y;

        // cursor.x fix non paging
        if self.cursor.x < (self.buffer_width / 4) + self.view_x {
            if self.cursor.x >= self.buffer_width / 4 {
                self.view_x = self.cursor.x - (self.buffer_width / 4);
            } else {
                self.view_x = 0;
            }
        }

        if self.cursor.x >= (3 * self.buffer_width / 4) + self.view_x {
            if self.cursor.x >= (3 * self.buffer_width / 4) {
                self.view_x = self.cursor.x - (3 * self.buffer_width / 4);
            } else {
                if self.buffer_width >= 1 {
                    self.view_x = self.buffer_width - 1;
                }
            }
        }

        // cursor.y fix non paging
        if self.cursor.y < (self.buffer_height / 4) + self.view_y {
            if self.cursor.y >= self.buffer_height / 4 {
                self.view_y = self.cursor.y - (self.buffer_height / 4);
            } else {
                self.view_y = 0;
            }
        }

        if self.cursor.y >= (3 * self.buffer_height / 4) + self.view_y {
            if self.cursor.y >= (3 * self.buffer_height / 4) {
                self.view_y = self.cursor.y - (3 * self.buffer_height / 4);
            } else {
                if self.buffer_height >= 1 {
                    self.view_y = self.buffer_height - 1;
                }
            }
        }
    }

    // must call after you clear the line
    fn redraw_line(&mut self, y: usize) {
        let line_maybe = self.buffer.line(y + self.view_y);
        if line_maybe.is_none() {
            for _ in 0..self.buffer_width {
                self.win.put_char(self.offset_x, y, ' ');
            }
            return;
        }

        let line = line_maybe.as_ref().unwrap();
        for x in 0..self.buffer_width {
            let chr_maybe = line[x + self.view_x].as_ref();
            if chr_maybe.is_none() {
                self.win.put_char(x + self.offset_x, y, ' ');
                continue;
            }
                
            let chr = chr_maybe.clone().unwrap();
            self.win.set_fg(chr.fg);
            self.win.set_bg(chr.bg);
            self.win.set_font(chr.font);
            self.win.put_char(x + self.offset_x, y, if chr.chr == '\t' {
                ' '
            } else {
                chr.chr
            });
        }
    }

    fn redraw(&mut self) {
        let end = cmp::min(self.buffer_height + self.view_y, self.buffer.height());
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

        //self.win.clear();
        for y in 0..self.buffer_height {
            if line_range.contains(&(y + self.view_y)) {
                self.win.set_bg(0);
                self.win.set_fg(1);
                self.win.set_font(1);
                self.win.put_str(0, y, pad(format!("{} ", y + self.view_y), self.offset_x).as_str());

            } else {
                self.win.put_str(0, y, " ".repeat(self.offset_x).as_str());
            }

            self.win.set_font(0);
            self.redraw_line(y);
        }
        
        if self.cursor.y >= self.view_y {
            self.win.set_bg(0);
            self.win.set_fg(1);
            self.win.set_font(1);
            self.win.put_str(self.cursor.x + self.offset_x - self.view_x, self.cursor.y - self.view_y, "|");
        }

        self.win.flush();
    }

    fn handle_str(&mut self, string: &str) {
        for chr in string.chars() {
            match chr {
                '\n' => {
                    let mut new_line = JVec::new();
                    let line_maybe = self.buffer.line_mut(self.cursor.y);
                    let mut new_x = 0;
                    if line_maybe.is_some() {
                        let line = line_maybe.as_mut().unwrap();
                        // makes the indentation level the same
                        // for newline
                        // todo use search function?
                        loop {
                            let chr_maybe = &line[new_x];
                            if chr_maybe.is_none() {
                                break;
                            }

                            let chr = chr_maybe.clone().unwrap();
                            if chr.chr != '\t' {
                                break;
                            }

                            new_line.push(Some(chr));

                            new_x += 1;
                        }
                        while line.len() != self.cursor.x {
                            new_line.push(line.remove(self.cursor.x));
                        }
                    }
                    self.buffer.insert_line(self.cursor.y + 1, new_line);
                    self.move_cursor(new_x, self.cursor.y + 1);
                },
                '\t' => {
                    // might change this
                    let tab = TAB_SIZE;
                    for _ in 0..tab {
                        self.buffer.insert(0, self.cursor.y, Char::new('\t'));
                    }
                    self.move_cursor(self.cursor.x + tab, self.cursor.y);
                },
                _ => {
                    self.buffer.insert(self.cursor.x, self.cursor.y, Char::new(chr));
                    self.move_cursor(self.cursor.x + 1, self.cursor.y);
                }
            }
        }
    }

    fn run(&mut self) {
        self.win.set_bg(0);
        self.win.set_fg(1);

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
                    if self.cursor.x == 0 {
                        if self.cursor.y != 0 {
                            let len = self.buffer.width(self.cursor.y - 1);
                            let new_x = len;
                            let new_y = self.cursor.y - 1;

                            let curr_line_maybe = self.buffer.remove_line(self.cursor.y);
                            let new_line_maybe = self.buffer.line_mut(new_y);

                            if new_line_maybe.is_none() {
                                *new_line_maybe = Some(JVec::new());
                            }

                            let new_line = new_line_maybe.as_mut().unwrap();

                            if curr_line_maybe.is_some() {
                                let curr_line = curr_line_maybe.unwrap();
                                for chr_maybe in curr_line {
                                    if chr_maybe.is_none() {
                                        new_line.push(chr_maybe);
                                    } else {
                                        let chr = chr_maybe.as_ref().unwrap();
                                        if chr.chr != '\t' {
                                            new_line.push(chr_maybe);
                                        }
                                    }
                                }
                            }

                            self.move_cursor(new_x, new_y);
                        }
                    } else {
                        let chr_maybe = self.buffer.remove(self.cursor.x - 1, self.cursor.y);
                        if chr_maybe.is_none() {
                            return;
                        }

                        let mut new_x = self.cursor.x - 1;
                        let chr = chr_maybe.as_ref().unwrap().chr;
                        if chr == '\t' {
                            for _ in 1..TAB_SIZE {
                                self.buffer.remove(0, self.cursor.y);

                                if new_x != 0 {
                                    new_x -= 1;
                                }
                            }

                        }

                        self.move_cursor(new_x, self.cursor.y);
                    }

                    self.redraw();
                },

                // moving the cursor with arrow keys
                Some(Event::Key(Code::Left)) => {
                    if self.cursor.x != 0 {
                        self.move_cursor(self.cursor.x - 1, self.cursor.y);

                        self.redraw();
                    }
                    
                    self.cursor.fake_x = self.cursor.x;
                },
                Some(Event::Key(Code::Right)) => {
                    let len = self.buffer.width(self.cursor.y);
                    self.move_cursor(cmp::min(self.cursor.x + 1, len), self.cursor.y);

                    self.cursor.fake_x = self.cursor.x;

                    self.redraw();
                },
                Some(Event::Key(Code::Up)) => {
                    if self.cursor.y != 0 {
                        let len = self.buffer.width(self.cursor.y - 1);
                        self.move_cursor(cmp::min(self.cursor.fake_x, len), self.cursor.y - 1);

                        self.redraw();
                    }
                },
                Some(Event::Key(Code::Down)) => {
                    if self.buffer.height() > self.cursor.y + 1 {
                        let len = self.buffer.width(self.cursor.y + 1);
                        self.move_cursor(cmp::min(self.cursor.fake_x, len), cmp::min(self.cursor.y + 1, self.buffer.height()));

                        self.redraw();
                    }
                }

                Some(Event::Close) => break,
                _ => ()
            }
        }

        for line in self.buffer.iter() {
            match line {
                Some(line) => {
                    let string: String = line.iter().map(|x| x.as_ref().unwrap().chr).collect();
                    let str_string: &str = string.as_str();
                    let string = str_string.replace("\t".repeat(TAB_SIZE).as_str(), "\t");
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

        buffer: Buffer::new(Some(Box::new(syntax::Rust::new()))),
        width: 0, height: 0,
        buffer_width: 0, buffer_height: 0,

        cursor: Cursor::new(),

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
