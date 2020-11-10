
use std::collections::BTreeSet;

use super::types::*;

fn first(rules: &[Production], sym: &Symbol) -> Vec<usize> {
    struct Visitor<'a> {
        rules: &'a [Production],
        found: BTreeSet<usize>,
        visited_nterms: BTreeSet<usize>,
    }

    impl<'a> Visitor<'a> {
        fn visit(&mut self, sym: &Symbol) {
            match sym {
                Symbol::T(k) => {
                    self.found.insert(*k);
                },
                Symbol::N(k) if !self.visited_nterms.contains(k) => {
                    self.visited_nterms.insert(*k);
                    self.rules.iter()
                        .filter(|r| r.symbol == *k && r.expand.len() != 0)
                        .for_each(|r| self.visit(&r.expand.first().unwrap()));
                },
                _ => ()
            }
        }
    }

    let mut visitor = Visitor {
        rules,
        found: BTreeSet::new(),
        visited_nterms: BTreeSet::new(),
    };
    visitor.visit(sym);
    visitor.found.into_iter().collect()
}

pub type LR0Item = (usize, usize);

// A LR(0) item with 1 token lookahead.
pub type LR1Item = (usize, usize, usize);

pub trait LRItem: Ord + Copy {
    /*
     * Returns the root item (that corresponds to
     * the start state).
     */
    fn root() -> Self;

    /*
     * Returns the production the item points to and the position
     * of the bullet in the output of the production.
     */
    fn prod_and_pos(&self) -> (usize, usize);

    /*
     * Returns the item that is obtained from the given item
     * when considering the given transition.
     */
    fn neighbor_items(&self, rule_id: usize, rule: &[Production]) -> Vec<Self>;

    /*
     * Moves the bullet to the right.
     */
    fn move_bullet(&self) -> Self;

    /*
     * Should a state that contains this item reduce when
     * this terminal is encountered ?
     */
    fn reduce_on(&self, term: usize) -> bool;

    /*
     * For debugging purposes.
     */
    fn print_extra_info(&self);
}

impl LRItem for LR0Item {
    fn root() -> LR0Item {
        (0, 0)
    }

    fn prod_and_pos(&self) -> (usize, usize) {
        *self
    }

    fn neighbor_items(&self, rule_id: usize, _rule: &[Production]) -> Vec<LR0Item> {
        vec!((rule_id, 0))
    }

    fn move_bullet(&self) -> LR0Item {
        (self.0, self.1 + 1)
    }

    fn reduce_on(&self, _term: usize) -> bool {
        true
    }

    fn print_extra_info(&self) {}
}

impl LRItem for LR1Item {
    fn root() -> LR1Item {
        (0, 0, 0) // 0 is always eof.
    }

    fn prod_and_pos(&self) -> (usize, usize) {
        (self.0, self.1)
    }

    fn neighbor_items(&self, rule_id: usize, rules: &[Production]) -> Vec<LR1Item> {
        if let Some(sym) = rules.get(self.0).unwrap().expand.get(self.1 + 1) {
            first(rules, sym).into_iter().map(|s| (rule_id, 0, s)).collect()
        } else {
            vec!((rule_id, 0, self.2))
        }
    }

    fn move_bullet(&self) -> LR1Item {
        (self.0, self.1 + 1, self.2)
    }

    fn reduce_on(&self, term: usize) -> bool {
        term == self.2
    }

    fn print_extra_info(&self) {
        print!(" [{}]", self.2);
    }
}

