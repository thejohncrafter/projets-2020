
use std::collections::{BTreeMap, BTreeSet};

use super::types::*;
use super::sets::*;
use super::dfa::*;

/*
 * Computes the state that is obtained when coming from s
 * by reading c.
 */
fn next_state(exp: &IRegexp, s: &CSet, c: &Character) -> CSet {
    fn is_in(c: &Character, c1: &Character) -> bool {
        match (c, c1) {
            (Character::Char(a), Character::Char(b)) => a == b,
            (Character::Char(a), Character::Alpha) => a.is_ascii_alphabetic(),
            (Character::Char(a), Character::Num) => a.is_ascii_digit(),
            (Character::Char(a), Character::Behaved) => is_behaved(*a),
            (Character::Alpha, Character::Alpha) => true,
            (Character::Num, Character::Num) => true,
            (Character::Alpha, Character::Behaved) => true,
            (Character::Num, Character::Behaved) => true,
            (Character::Behaved, Character::Behaved) => true,
            (_, Character::Any) => true,
            _ => false
        }
    }

    s.iter().filter(|ci| {
        match ci {
            IChar::Char(c1, _) => is_in(&c, c1),
            _ => false
        }
    }).flat_map(|ci| {
        follow(ci, exp).into_iter()
    }).collect()
}

fn build_states(exp: &IRegexp) -> Vec<(BTreeMap<Character, usize>, Option<usize>)> {
    struct Ctx<'a> {
        exp: &'a IRegexp,
        states_trans: BTreeMap<State, (usize, Option<usize>, TransMap)>,
        next_i: usize
    }

    impl Ctx<'_> {
        fn visit(&mut self, s: &State) {
            if self.states_trans.contains_key(s) {
                return
            }

            let id = self.next_i;
            self.states_trans.insert(s.clone(), (id, None, TransMap::new()));
            self.next_i += 1;
            let mut trans_map = TransMap::new();
            let mut accept = None;

            s.iter().map(|ic| {
                match ic {
                   IChar::Char(c, _) => CharOrEof::Char(c.clone()),
                   IChar::Hash(i) => CharOrEof::Eof(i.clone()),
                }
            }).collect::<BTreeSet<CharOrEof>>().into_iter().for_each(|c| {
                match c {
                    CharOrEof::Char(c) => {
                        let t = next_state(self.exp, s, &c);
                        self.visit(&t);
                        trans_map.insert(c, t);
                    },
                    CharOrEof::Eof(i) => {
                        match accept {
                            None => accept = Some(i),
                            // The priority order follows
                            // the delcaration order.
                            Some(_) => (),
                        }
                    },
                } 
            });

            self.states_trans.insert(s.clone(), (id, accept, trans_map));
        }
    }

    let mut ctx = Ctx {
        exp,
        states_trans: BTreeMap::new(),
        next_i: 0,
    };
    let q0 = first(exp);
    ctx.visit(&q0);

    // Fill this vector with a dummy element.
    let mut states = vec![(BTreeMap::<Character, usize>::new(), None); ctx.states_trans.len()];
    ctx.states_trans.values().for_each(|(id, accept, trans)| {
        let trans = trans.iter()
            .map(|(c, s)| (c.clone(), ctx.states_trans.get(s).unwrap().0))
            .collect();
        states[*id] = (trans, *accept);
    });

    states
}

/*
 * Builds a new regexp where each character has
 * a unique identifier, and appends a hash
 * (represented by '#').
 */
fn build_iregexp(exps: &[Regexp]) -> IRegexp {
    struct Ctx {
        i: usize
    }

    impl Ctx {
        fn visit(&mut self, exp: &Regexp) -> IRegexp {
            match exp {
                Regexp::Epsilon => IRegexp::Epsilon,
                Regexp::Character(c) => {
                    let e = IRegexp::Character(IChar::Char(c.clone(), self.i));
                    self.i += 1;
                    e
                },
                Regexp::Union(l, r) => IRegexp::Union(
                    Box::new(self.visit(l)),
                    Box::new(self.visit(r)),
                ),
                Regexp::Concat(l, r) => IRegexp::Concat(
                    Box::new(self.visit(l)),
                    Box::new(self.visit(r)),
                ),
                Regexp::Star(e) => IRegexp::Star(Box::new(self.visit(e))),
            }
        }

        fn transform(&mut self, exp: &Regexp, id: usize) -> IRegexp {
            let e = self.visit(exp);
            IRegexp::Concat(Box::new(e), Box::new(IRegexp::Character(IChar::Hash(id))))
        }
    }

    let mut ctx = Ctx {i: 0};

    if exps.len() > 0 {
        exps.iter().enumerate().skip(1).fold(
            ctx.transform(&exps[0], 0),
            |acc, (id, exp)| IRegexp::Union(Box::new(acc), Box::new(ctx.transform(exp, id)))
        )
    } else {
        panic!("Expected at least one regexp") 
    }
}

/*
 * Builds an automaton able to tokenize according to
 * the given regexp's.
 */
pub fn build_automaton<'a, 'b, T, S, U, E>(
    exps: &[Regexp],
    prods: &'a [&dyn Fn(S::Span, &str) -> Result<U, String>],
    chars: T,
    input: &'b S,
)
        -> DFA<'a, 'b, T, S, U, E>
    where T: Iterator<Item = (char, S::Loc)>,
          S: IndexedInput<'a>,
          E: std::error::Error + From<(S::Span, String)>,
{
    assert!(exps.len() == prods.len(), "Expected exactly as much expressions as producers.");
    let iexp = build_iregexp(exps); 
    let states = build_states(&iexp);
    DFA::new(states, prods, chars, input)
}

