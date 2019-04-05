use std::ops::Range;

use crate::buffer::{Char, Buffer};

// divine intellect syntax highlighter

pub trait Syntax {
    fn insert(&mut self, x: usize, y: usize, chr: char, buffer: &mut Buffer);
    fn remove(&mut self, x: usize, y: usize, chr: &Char, buffer: &mut Buffer);
}

pub struct Plain;

impl Plain {
    pub fn new() -> Plain {
        Plain
    }
}

impl Syntax for Plain {
    fn insert(&mut self, x: usize, y: usize, chr: char, buffer: &mut Buffer) {
        buffer.get_mut(x, y).unwrap().fg = 1;
    }

    fn remove(&mut self, x: usize, y: usize, chr: &Char, buffer: &mut Buffer) {
        return;
    }
}

// Rust syntax
const TYPE: usize = 4; // type or const
const IDENTIFIER: usize = 1;
const STRING: usize = 3;
const STRING_START: usize = STRING + 8;
const STRING_END: usize = STRING + 16;
const NUMERIC: usize = 5;

const COMMENT: usize = 7;
const COMMENT_0: usize = COMMENT + 8;
const COMMENT_1: usize = COMMENT + 16;

const KEYWORD: usize = 2;
const DEFAULT: usize = 1;

pub struct Rust;

impl Rust {
    pub fn new() -> Rust {
        Rust
    }
}

// range must be valid
fn color_range(range: Range<usize>, y: usize, color: usize, buffer: &mut Buffer) {
    for x in range {
        let chr_maybe = buffer.get(x, y);
        if chr_maybe.is_none() {
            return;
        }
        let chr = buffer.get_mut(x, y).unwrap();
        chr.fg = color;
    }
}

fn search(x: usize, y: usize, buffer: &mut Buffer, pred: fn(char) -> bool) -> (Range<usize>, String) {
    let mut start = x;
    loop {
        let chr_maybe = buffer.get(start, y);
        if chr_maybe.is_none() {
            break;
        }
        let chr = chr_maybe.unwrap().chr;

        if !pred(chr) {
            start += 1;
            break;
        }

        if start == 0 {
            break;
        }

        start -= 1;
    }

    let mut end = x;
    loop {
        let chr_maybe = buffer.get(end, y);
        if chr_maybe.is_none() {
            break;
        }
        let chr = chr_maybe.unwrap().chr;

        if !pred(chr) {
            break;
        }

        end += 1;
    }

    let mut string = String::new();

    let range = start..end;
    for x in range.clone() {
        string.push(buffer.get(x, y).unwrap().chr);
    }

    (range, string)
}

impl Rust {
    fn infer(&mut self, x: usize, y: usize, buffer: &mut Buffer) {
        let chr_maybe = buffer.get(x, y);
        if chr_maybe.is_none() {
            return
        }
        let chr = (*chr_maybe.unwrap()).clone();

        let mut color = DEFAULT;
        if x == 0 {
            if chr.chr == '"' {
                color = STRING_START;
            }

            color_range(0..1, y, color, buffer);
        } else {
            let left_chr_maybe = buffer.get(x - 1, y);
            if left_chr_maybe.is_some() {
                let left_chr = left_chr_maybe.unwrap();

                color = match left_chr.fg {
                    COMMENT | COMMENT_1 => COMMENT,
                    STRING_START | STRING => STRING,
                    _ => color
                };

                if chr.chr == '"' {
                    if left_chr.fg == STRING {
                        color = STRING_END;
                    } else {
                        color = STRING_START;
                    }
                }

                color_range(x..(x + 1), y, color, buffer);
            }
        }
    }

    // returns new x
    fn update(&mut self, x: usize, y: usize, buffer: &mut Buffer) -> usize {
        self.infer(x, y, buffer);

        let chr_maybe = buffer.get(x, y);
        if chr_maybe.is_none() {
            return 0
        }
        let mut color = chr_maybe.unwrap().fg;

        let (range, string) = search(x, y, buffer, |chr| chr.is_alphanumeric());

        if color != COMMENT && color != STRING {
            color = match string.as_str() {
                | "struct" | "enum" | "impl" | "trait"
                | "loop" | "while" | "for" | "in" | "if" | "else" | "unsafe"
                | "return" | "break" | "continue"
                | "pub" | "mod"
                | "fn" | "use" | "extern" | "crate"
                | "i8" | "i16" | "i32" | "i64" | "isize"
                | "u8" | "u16" | "u32" | "u64" | "usize"
                | "char" | "str"
                | "self"
                | "let" | "mut" | "const" => KEYWORD,
                _ => DEFAULT
            };
        }

        color = match string.chars().next() {
            Some(chr) => if chr.is_uppercase() {
                TYPE
            } else if chr.is_numeric() {
                NUMERIC
            } else {
                color
            },
            _ => color
        };

        let end = range.end;
        color_range(range, y, color, buffer);

        end - x
    }
}

impl Syntax for Rust {
    fn insert(&mut self, x: usize, y: usize, chr: char, buffer: &mut Buffer) {
        let width = buffer.width(y);

        // todo dont use search
        let (range, string) = search(x, y, buffer, |chr| chr == '/');
        if string.len() > 1 {
            color_range(range.start..(range.start + 1), y, COMMENT_0, buffer);
            color_range((range.start + 1)..(range.start + 2), y, COMMENT_1, buffer);
            color_range((range.start + 2)..width, y, COMMENT, buffer);

            return;
        }

        Rust::update(self, x, y, buffer);
    }

    fn remove(&mut self, x: usize, y: usize, chr: &Char, buffer: &mut Buffer) {
        let width = buffer.width(y);

        match chr.fg {
            COMMENT_0 => {
                for i in x..width {
                    Rust::update(self, i, y, buffer);
                }
            },
            COMMENT_1 => {
                buffer.get_mut(x - 1, y).unwrap().fg = DEFAULT;

                for i in x..width {
                    Rust::update(self, i, y, buffer);
                }
            }
            _ => {
                if x != 0 {
                    Rust::update(self, x - 1, y, buffer);
                }

                Rust::update(self, x, y, buffer);
            }
        };
    }
}