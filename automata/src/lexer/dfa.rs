
use std::marker::PhantomData;
use std::collections::BTreeMap;

pub use crate::TokenOrEof;
use super::types::{Character, is_behaved};

pub trait IndexedInput<'a> {
    type Loc: std::fmt::Debug + Copy;
    type Span: std::fmt::Display + Copy + 'a;

    fn first_loc(&self) -> Self::Loc;
    fn span(&self, start: &Self::Loc, end: &Self::Loc) -> Self::Span;
    fn slice(&self, span: &Self::Span) -> &'a str;
}

pub struct DFA<'a, 'b, I, S, U, E>
    where I: Iterator<Item = (char, S::Loc)>,
          S: IndexedInput<'a>,
          E: std::error::Error + From<(S::Span, String)>,
{
    phantom_error: PhantomData<E>,
    /* 
     * States are usize's, each state is associated
     * with its transition table and a flag which indicates
     * wether it is an accepting state.
     */
    pub(super) states: Vec<(BTreeMap<Character, usize>, Option<usize>)>,
    chars: std::iter::Peekable<I>,
    input: &'b S,
    next_start: S::Loc,
    last_span: S::Span,
    producers: &'a [&'a dyn Fn(S::Span, &'a str) -> Result<U, String>],
}

impl<'a, 'b, I, S, U, E> DFA<'a, 'b, I, S, U, E>
    where I: Iterator<Item = (char, S::Loc)>,
          S: IndexedInput<'a>,
          E: std::error::Error + From<(S::Span, String)>,
{
    pub fn new(
        states: Vec<(BTreeMap<Character, usize>, Option<usize>)>,
        producers: &'a [&'a dyn Fn(S::Span, &'a str) -> Result<U, String>],
        chars: I,
        input: &'b S,
    )
        -> Self
    {
        DFA {
            phantom_error: std::marker::PhantomData,
            states,
            next_start: input.first_loc(),
            last_span: input.span(&input.first_loc(), &input.first_loc()),
            chars: chars.peekable(),
            producers,
            input,
        }
    }
}

impl<'a, 'b, I, S, U, E> Iterator for DFA<'a, 'b, I, S, U, E>
    where I: Iterator<Item = (char, S::Loc)>,
          S: IndexedInput<'a>,
          E: std::error::Error + From<(S::Span, String)>,
{
    type Item = Result<(S::Span, TokenOrEof<U>), E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.chars.peek().is_none() {
            return Some(Ok((self.last_span, TokenOrEof::Eof)))
        }

        let start = self.next_start;
        let mut curr = self.next_start;
        let mut state = Some((0, self.states[0].1));

        fn trans(map: &BTreeMap<Character, usize>, c: char) -> Option<&usize> {
            // We made sure, when we built the table, that this will be correct
            // (the priority order will follow the declaration order).
            map.get(&Character::Char(c)).or_else(|| if c.is_ascii_alphabetic() {
                    map.get(&Character::Alpha)
                } else {None}.or_else(|| if c.is_ascii_digit() {
                    map.get(&Character::Num)
                } else {None}).or_else(|| if is_behaved(c) {
                    map.get(&Character::Behaved)
                } else {None})
                .or_else(|| map.get(&Character::Any))
            )
        }

        while let (Some((q, _)), Some((c, loc))) = (state, self.chars.peek()) {
            if let Some(next) = trans(&self.states[q].0, *c) {
                curr = *loc;
                
                self.chars.next();
                state = Some((*next, self.states[*next].1))
            } else {
                self.next_start = *loc;
                break
            }
        }
        
        let span = self.input.span(&start, &curr);

        if let Some((_, Some(id))) = state {
            self.last_span = span;
            let res = self.producers[id](span, self.input.slice(&span));
            Some(match res {
                Ok(res) => Ok((span, TokenOrEof::Token(res))),
                Err(msg) => Err((span, msg).into()),
            })
        } else {
            Some(Err((span, "Unrecognized token at {}.".to_string()).into()))
        }
    }
}

