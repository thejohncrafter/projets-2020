
use std::fmt;

use super::lexer::*;

#[derive(Debug, Clone, Copy)]
pub struct Span<'a> {
    pub file: &'a str,
    pub start: (usize, usize, usize),
    pub end: (usize, usize, usize),
}

impl fmt::Display for Span<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.start.1 == self.end.1 {
            if self.start.2 == self.end.2 {
                write!(
                    f,
                    "File {}, line {}, character {}:",
                    self.file,
                    self.start.1,
                    self.start.2,
                )
            } else {
                write!(
                    f,
                    "File {}, line {}, characters {}-{}:",
                    self.file,
                    self.start.1,
                    self.start.2,
                    self.end.2,
                )
            }
        } else {
            write!(
                f,
                "File {}, line {}, character {} to line {}, character {}:",
                self.file,
                self.start.1, self.start.2,
                self.end.1, self.end.2,
            )
        }
    }
}

struct CountingIter<'a> {
    chars: std::str::CharIndices<'a>,
    line: usize,
    column: usize,
}

impl<'a> CountingIter<'a> {
    fn new(s: &'a str) -> CountingIter<'a> {
        CountingIter {
            chars: s.char_indices(),
            line: 1,
            column: 1,
        }
    }
}

impl Iterator for CountingIter<'_> {
    type Item = (char, (usize, usize, usize));

    fn next(&mut self) -> Option<Self::Item> {
       if let Some((i, c)) = self.chars.next() {
            let res = (c, (i, self.line, self.column));
            
            // Prepare the position of the next character.
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            
            Some(res)
       } else {
           None
       }
    }
}

pub struct LineIter<'a> {
    chars: std::iter::Peekable<std::iter::Enumerate<CountingIter<'a>>>,
    i: usize,
}

impl LineIter<'_> {
    pub fn new<'a> (s: &'a str) -> LineIter<'a> {
        LineIter {
            chars: CountingIter::new(s).enumerate().peekable(),
            i: 0,
        }
    }
}

impl Iterator for LineIter<'_> {
    type Item = (char, (usize, usize, usize));

    fn next(&mut self) -> Option<Self::Item> {
        self.chars.next().map(|(i, x)| {
            self.i = i;
            x
        })
    }
}

pub struct IndexedString<'a> {
    file_name: &'a str,
    s: &'a str,
}

impl<'a> IndexedString<'a> {
    pub fn new(file_name: &'a str, s: &'a str) -> IndexedString<'a> {
        IndexedString {file_name, s}
    }
}

impl<'a> IndexedInput<'a> for IndexedString<'a> {
    type Loc = (usize, usize, usize);
    type Span = Span<'a>;

    fn first_loc(&self) -> Self::Loc {
        (0, 1, 1)
    }

    fn span(&self, start: &Self::Loc, end: &Self::Loc) -> Self::Span {
        Span {file: self.file_name, start: *start, end: *end}
    }

    fn slice(&self, span: &Self::Span) -> &'a str {
        &self.s[span.start.0 ..= span.end.0]
    }
}

