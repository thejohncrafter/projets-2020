
use super::types::*;

/*
 * Tests if epsilon is accepted.
 */
pub fn null(exp: &IRegexp) -> bool {
    match exp {
        IRegexp::Epsilon => true,
        IRegexp::Character(_) => false,
        IRegexp::Union(l, r) => null(l) || null(r),
        IRegexp::Concat(l, r) => null(l) && null(r),
        IRegexp::Star(_) => true,
    }
}

/*
 * Computes the FIRST set.
 */
pub fn first(exp: &IRegexp) -> CSet {
    match exp {
        IRegexp::Epsilon => CSet::new(),
        IRegexp::Character(c) => {
            let mut set = CSet::new();
            set.insert(c.clone());
            set
        },
        IRegexp::Union(l, r) => CSet::union(&first(l), &first(r)).cloned().collect(),
        IRegexp::Concat(l, r) => {
            if null(l) {
                CSet::union(&first(l), &first(r)).cloned().collect()
            } else {
                first(l)
            }
        },
        IRegexp::Star(e) => first(e),
    }
}

/*
 * Coputes the LAST set.
 */
pub fn last(exp: &IRegexp) -> CSet {
    match exp {
        IRegexp::Epsilon => CSet::new(),
        IRegexp::Character(c) => {
            let mut set = CSet::new();
            set.insert(c.clone());
            set
        },
        IRegexp::Union(l, r) => CSet::union(&last(l), &last(r)).cloned().collect(),
        IRegexp::Concat(l, r) => {
            if null(r) {
                CSet::union(&last(l), &last(r)).cloned().collect()
            } else {
                last(r)
            }
        },
        IRegexp::Star(e) => last(e),
    }
}

/*
 * Computes the FOLLOW set.
 */
pub fn follow(c: &IChar, exp: &IRegexp) -> CSet {
    match exp {
        IRegexp::Epsilon | IRegexp::Character(_) => CSet::new(),
        IRegexp::Union(l, r) => CSet::union(&follow(c, l), &follow(c, r)).cloned().collect(),
        IRegexp::Concat(l, r) => {
            let follow_l = follow(c, l);
            let follow_r = follow(c, r);
            let u = CSet::union(&follow_l, &follow_r);
            
            if last(l).contains(&c) {
               u.chain(first(r).iter()).cloned().collect()
            } else {
                u.cloned().collect()
            }
        },
        IRegexp::Star(e) => {
            if last(e).contains(&c) {
                CSet::union(&follow(c, e), &first(e)).cloned().collect()
            } else {
                follow(c, e)
            }
        },
    }
}

