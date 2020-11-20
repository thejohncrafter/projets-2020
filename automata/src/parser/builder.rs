
use std::collections::{BTreeSet, BTreeMap};

use super::types::*;
use super::items::*;

pub struct Builder<'a, I> {
    rules: &'a [Production],
    term_count: usize,
    nterm_count: usize,
    // Stores the states that are built, and the transition
    // map for each state.
    states: Vec<(BTreeSet<I>, BTreeMap<Symbol, usize>)>,
}

impl<'a, I> Builder<'a, I> where
    I: LRItem
{
    pub fn new(
        rules: &'a [Production],
        term_count: usize, nterm_count: usize,
    ) -> Builder<I> {
        Builder {
            rules,
            term_count, nterm_count,
            states: Vec::new(),
        }
    }

    /*
     * Returns the token after the bullet in the given item
     * (if it exists, None otherwise).
     */
    fn next_token(&self, item: &I) -> Option<&Symbol> {
        let (prod_id, pos) = item.prod_and_pos();
        let rule = self.rules.get(prod_id).unwrap();
        rule.expand.get(pos)
    }

    /*
     * Returns the items that should be added when computing
     * a closure.
     */
    fn neighbors(&self, item: &I) -> Vec<I> {
        if let Some(Symbol::N(id)) = self.next_token(item) {
            // id is the id of the non-terminal symbol just after the bullet
            // (at this point, we know there is one such symbol).
            self.rules.iter().enumerate().filter_map(|(i, rule)| {
                if rule.symbol == *id {
                    Some(item.neighbor_items(i, self.rules))
                } else {None}
            }).flatten().collect()
        } else {vec!()}
    }

    fn closure(&self, set: BTreeSet<I>) -> BTreeSet<I> {
        let mut set = set; 

        loop {
            let mut new_set: Option<BTreeSet<I>> = None;
           
            set.iter().for_each(|item| {
               self.neighbors(item).iter().for_each(|item| {
                    if !set.contains(item) {
                        let mut s = match new_set.take() {
                            Some(s) => s,
                            None => set.clone()
                        };
                        s.insert(*item);
                        new_set = Some(s);
                    }
               })
            });

            if let Some(s) = new_set {
                set = s
            } else {
                break
            }
        }
        
        set
    }

    /*
     * Finds the id of the state that is represented by the given set
     * (if it exists).
     */
    fn state_id(&self, set: &BTreeSet<I>) -> Option<usize> {
        self.states.iter().enumerate().find_map(|(id, (state, _))| {
            if state == set {
                Some(id)
            } else {None}
        })
    }

    /*
     * Computes the transitions from the given state.
     * Returns each symbol associated with each state, to help
     * with the construction of the transitions table.
     */
    fn transitions_from(&mut self, id: usize) { 
        let term_count = self.term_count;
        let nterm_count = self.nterm_count;

        let mut next_set = |token: Symbol| {
            let set = &self.states.get(id).unwrap().0;
            let mut next_set: BTreeSet<I> = BTreeSet::new();

            set.iter().filter(|item| match self.next_token(item) {
                    Some(t) => *t == token,
                    None => false
                })
                .for_each(|item| {next_set.insert(item.move_bullet());});

            let state = self.closure(next_set);
            
            if state.len() != 0 {
                let tgt_id = match self.state_id(&state) {
                    Some(tgt_id) => tgt_id,
                    None => {
                        let tgt_id = self.states.len();
                        self.states.push((state, BTreeMap::new()));
                        self.transitions_from(tgt_id);
                        tgt_id
                    }
                };
                self.states.get_mut(id).unwrap().1.insert(token, tgt_id);
            }
        };

        for i in 0..term_count {
            next_set(Symbol::T(i)) 
        }
        for i in 0..nterm_count {
            next_set(Symbol::N(i))
        }
    }

    fn build_states(&mut self) {
        let mut set = BTreeSet::new();
        set.insert(I::root());
        set = self.closure(set);
        self.states.push((set, BTreeMap::new()));
        self.transitions_from(0);
    }

    pub fn build(&mut self) -> MachineTable {
        self.build_states();
        
        self.states.iter().map(|(items, trans)| {
            let mut actions = vec![None; self.term_count];
            let mut goto = vec![Goto::None; self.nterm_count];
            
            // Fill the actions
            trans.iter().for_each(|(sym, state)| {
                match sym {
                    Symbol::T(k) => actions[*k] = Some(Action::Shift(*state)),
                    Symbol::N(k) => goto[*k] = Goto::Some(*state),
                }
            });

            // Fill the reductions
            items.iter().for_each(|item| {
                let (prod, pos) = item.prod_and_pos();
                if self.rules.get(prod).unwrap().expand.len() == pos {
                    // We are at the end of a production.
                    // In this case we may want to reduce.
                    for i in 0..self.term_count {
                        if item.reduce_on(i) {
                            if let None = actions[i] {
                                actions[i] = Some(Action::Reduce(prod))
                            } else {
                                panic!("Ambiguous grammar !")
                            }
                        }
                    }
                }
            });

            (actions, goto)
        }).collect()
    }
}

