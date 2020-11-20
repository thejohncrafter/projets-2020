
pub use crate::TokenOrEof;
use super::types::*;

#[derive(Debug)]
pub enum StackItem<T> {
    State(usize),
    Token(Symbol, Option<T>),
}

pub struct PDA<T> {
    rules: Vec<Production>,
    table: MachineTable,
    stack: Vec<StackItem<T>>
}

impl<T> PDA<T> {
    pub fn new(rules: Vec<Production>, table: MachineTable) -> PDA<T> {
        PDA {
            rules,
            table,
            stack: vec!(StackItem::State(0)), // The first state is 0.
        }
    }

    pub fn parse<I, S, E>(
        mut self,
        tokens: &mut I,
        on_empty: &dyn Fn() -> Result<T, String>,
        builders: &[&dyn Fn(S, Vec<Option<T>>) -> Result<T, String>],
    ) -> Result<T, E>
        where I: Iterator<Item = Result<(S, TokenOrEof<(usize, T)>), E>>,
              E: std::error::Error + From<(S, String)>,
              S: Copy,
    {        
        let mut tokens = tokens.map(|r| r.map(|(span, x)| {
            if let TokenOrEof::Token((id, t)) = x {
                (span, id, Some(t))
            } else {
                (span, 0, None)
            }
        }));
        let mut token = tokens.next().unwrap()?;

        if let None = token.2 {
            return on_empty().map_err(|msg| (token.0, msg).into())
        }

        loop {
            fn expect_state<T>(x: &StackItem<T>) -> usize {
                match x {
                    StackItem::State(q) => *q,
                    _ => panic!("Malformed stack, this is a bug !")
                } 
            }

            let span = token.0;
            let id = token.1;
            let state = expect_state(self.stack.last().unwrap());

            match self.table.get(state).unwrap().0.get(id) {
                Some(Some(Action::Shift(q))) => {
                    let (_, _, x) = token;
                    self.stack.push(StackItem::Token(Symbol::T(id), x));
                    self.stack.push(StackItem::State(*q));
                    token = tokens.next().unwrap()?;
                },
                Some(Some(Action::Reduce(rule_id))) => {
                    let rule = &self.rules[*rule_id];
                    let args = self.stack.drain((self.stack.len() - 2*rule.expand.len())..)
                        .filter_map(|x| match x {
                            StackItem::Token(_, y) => Some(y),
                            _ => None
                        })
                        .collect::<Vec<Option<T>>>();
                    let builder = builders.get(*rule_id).expect(&format!("Missing builder {}.", rule_id));
                   
                    let state = expect_state(self.stack.last().unwrap());
                    let built = builder(span, args);
                    let id = rule.symbol;

                    if let Goto::Some(q) = self.table.get(state).unwrap().1.get(id).unwrap() {
                        let built = built.map_err(|msg| (span, msg).into())?;
                        self.stack.push(StackItem::Token(Symbol::N(rule.symbol), Some(built)));
                        self.stack.push(StackItem::State(*q));
                    } else {
                        if state == 0 {
                            return built.map_err(|msg| (span, msg).into());
                        } else {
                            return Err((span, "Unexpected token.".to_string()).into())
                        }
                    }
                },
                _ => {
                    return Err((span, "Unexpected token.".to_string()).into())
                }
            }
        }
    }
}

