use std::slice::Iter;
use std::mem;

use jvec::JVec;

use crate::syntax::Syntax;

#[derive(Clone, Debug)]
pub struct Char {
    pub chr: char,
    pub bg: usize,
    pub fg: usize,
    pub font: usize
}

impl Char {
    // for debugging
    pub fn new(chr: char) -> Char {
        Char {
            chr: chr,
            bg: 0,
            fg: 1,
            font: 0
        }
    }
}

// buffer will handle
// syntax update callbacks
// later
pub struct Buffer {
    buffer: JVec<JVec<Char>>,
    syntax: Option<Box<Syntax>>
}

impl Buffer {
    pub fn new(syntax: Option<Box<Syntax>>) -> Buffer {
        Buffer {
            buffer: JVec::new(),
            syntax: syntax
        }
    }

    pub fn line(&self, y: usize) -> &Option<JVec<Char>> {
        &self.buffer[y]
    }

    pub fn line_mut(&mut self, y: usize) -> &mut Option<JVec<Char>> {
        &mut self.buffer[y]
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Char> {
        self.buffer[y].as_ref()?[x].as_ref()
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Char> {
        self.buffer[y].as_mut()?[x].as_mut()
    }

    pub fn insert(&mut self, x: usize, y: usize, chr: Char) {
        let line_maybe = &mut self.buffer[y];
        if line_maybe.is_none() {
            *line_maybe = Some(JVec::new());
        }
        let line = line_maybe.as_mut().unwrap();
        let chr_chr = chr.chr;
        line.insert(x, chr);

        // todo rewrite buffer such that there needs to be no
        // self.function(self)
        let syntax_maybe = mem::replace(&mut self.syntax, None);
        if syntax_maybe.is_some() {
            let mut syntax = syntax_maybe.unwrap();
            syntax.insert(x, y, chr_chr, self);
            self.syntax = Some(syntax);
        }
    }

    // returns char
    pub fn remove(&mut self, x: usize, y: usize) -> Option<Char> {
        let line_maybe = &mut self.buffer[y];
        if line_maybe.is_none() {
            return None;
        }
        let line = line_maybe.as_mut().unwrap();
        let chr_maybe = line.remove(x);

        if chr_maybe.is_some() {
            let chr = chr_maybe.as_ref().unwrap();
            
            // todo rewrite buffer such that there needs to be no
            // self.function(self)
            let syntax_maybe = mem::replace(&mut self.syntax, None);
            if syntax_maybe.is_some() {
                let mut syntax = syntax_maybe.unwrap();
                syntax.remove(x, y, &chr, self);
                self.syntax = Some(syntax);
            }
        }

        chr_maybe
    }

    pub fn insert_line(&mut self, y: usize, line: JVec<Char>) {
        self.buffer.insert(y, line);
    }

    pub fn remove_line(&mut self, y: usize) -> Option<JVec<Char>> {
        self.buffer.remove(y)
    }

    pub fn height(&self) -> usize {
        self.buffer.len()
    }

    // can cause a panic
    pub fn width(&self, y: usize) -> usize {
        self.buffer[y].as_ref().unwrap().len()
    }

    pub fn iter(&self) -> Iter<Option<JVec<Char>>> {
        self.buffer.iter()
    }
}