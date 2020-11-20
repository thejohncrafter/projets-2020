
use super::types::*;
use super::builder::Builder;
#[allow(unused)]
use super::items::{LR0Item, LR1Item};

pub fn build_pda_data(
        terms: &[&str],
        nterms: &[&str],
        prods: &[(&str, Vec<&str>)],
        start: &str
    )
    -> (Vec<Production>, MachineTable)
{
    let term_count = 1 + terms.len();
    let nterm_count = 1 + nterms.len();

    let maybe_term = |name: &str| -> Option<usize> {
        terms.iter().position(|x| *x == name).map(|i| i + 1)
    };

    let nterm_index = |name: &str| -> usize {
        if let Some(i) = nterms.iter().position(|x| *x == name) {
            i + 1
        } else {
            panic!(format!("Can't find the non-terminal \"{}\"", name))
        }
    };

    // Transform the rules to use the index-based approach
    // (instead of using plain names).
    let rules: Vec<Production> = [0].iter().map(|_| {
            Production {
                symbol: 0,
                expand: vec!(Symbol::N(nterm_index(&start)), Symbol::T(0)),
            }
        }).chain(prods.into_iter().map(|(s, t)| {
           let symbol = nterm_index(s);
           let expand = t.into_iter().map(|name| {
                if let Some(k) = maybe_term(name) {
                    Symbol::T(k)
                } else {
                    Symbol::N(nterm_index(name))
                }
       }).collect();
       Production {symbol, expand}
    })).collect();

    let mut builder = Builder::<LR1Item>::new(
        &rules,
        term_count, nterm_count,
    );

    let states = builder.build();
    (rules, states)
}

