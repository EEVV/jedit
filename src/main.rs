use std::cmp;

extern crate jwin;
extern crate jvec;

use jwin::{Code, Event, Win};
use jvec::JVec;


fn pad_zero(mut string: String, n: usize) -> String {
    if string.len() < n {
        for _ in string.len()..n {
            string.insert(0, '0');
        }
    }

    string
}

struct JEdit {
    win: Win,

    buffer: JVec<JVec<char>>,
    width: usize, height: usize,

    cursor_x: usize,
    cursor_y: usize,

    // line number offset
    offset_x: usize
}

impl JEdit {
    fn buffer_get(&self, x: usize, y: usize) -> Option<char> {
        self.buffer[y].as_ref()?[x]
    }

    fn move_cursor(&mut self, x: usize, y: usize) {
        self.win.put_str(self.cursor_x + self.offset_x, self.cursor_y, self.buffer_get(self.cursor_x, self.cursor_y).unwrap_or(' ').to_string().as_str(), 0);

        self.cursor_x = x;
        self.cursor_y = y;

        self.win.put_str(self.cursor_x + self.offset_x, self.cursor_y, "|", 1);
    }

    fn redraw_line(&mut self, y: usize) {
        self.win.put_str(self.offset_x, y, " ".repeat(self.width).as_str(), 0);

        let maybe = self.buffer[y].clone();
        if maybe.is_some() {
            let string: String = maybe.unwrap().into_iter().map(|x| x.unwrap_or(' ')).collect();
            self.win.put_str(self.offset_x, y, string.as_str(), 0);
        }
    }

    fn redraw(&mut self) {
        for y in 0..self.height {
            self.offset_x = format!("{}", self.height - 1).len(); // good logarithm
            self.win.put_str(0, y, pad_zero(format!("{}", y), self.offset_x).as_str(), 1);
            self.offset_x += 1;

            self.redraw_line(y);
        }

        self.win.put_str(self.cursor_x + self.offset_x, self.cursor_y, "|", 1);
    }

    fn run(&mut self) {
        loop {
            match self.win.poll() {
                // redraw event
                Some(Event::Redraw(w, h)) => {
                    self.width = w;
                    self.height = h;

                    self.redraw();
                },

                // key events
                Some(Event::Key(Code::Showable(string))) => {
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
                                // update the lower lines
                                for y in 0..self.height {
                                    self.redraw_line(y);
                                }
                                self.move_cursor(0, self.cursor_y + 1);
                            },
                            _ => {
                                let line_maybe = &mut self.buffer[self.cursor_y];
                                if line_maybe.is_none() {
                                    *line_maybe = Some(JVec::new());
                                }
                                let line = line_maybe.as_mut().unwrap();
                                let shifted = line.insert(self.cursor_x, chr);
                                if shifted {
                                    self.redraw_line(self.cursor_y);
                                } else {
                                    self.win.put_str(self.cursor_x + self.offset_x, self.cursor_y, string, 0);
                                }
                                self.move_cursor(self.cursor_x + 1, self.cursor_y);
                            }
                        }
                    }
                },

                // special key
                Some(Event::Key(Code::Backspace)) => {
                    if self.cursor_x == 0 {
                        if self.cursor_y != 0 {
                            self.buffer.remove(self.cursor_y);
                            self.move_cursor(self.buffer[self.cursor_y - 1].as_ref().unwrap_or(&JVec::new()).len(), self.cursor_y - 1);
                        }
                    } else {
                        self.buffer[self.cursor_y].as_mut().unwrap().remove(self.cursor_x - 1);
                        self.win.put_str(self.cursor_x + self.offset_x - 1, self.cursor_y, " ", 0);
                        self.move_cursor(self.cursor_x - 1, self.cursor_y);
                    }
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

        cursor_x: 0,
        cursor_y: 0,
        offset_x: 0
    };

    jedit.run();
}
