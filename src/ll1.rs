use std::collections::{HashMap, HashSet};

use crate::{
    grammar::{Grammar, NonTerminal, Production, Symbol, Terminal},
    parse_tree::ParseTree,
};

type FirstSet<'a> = HashMap<&'a NonTerminal, HashSet<Option<&'a Terminal>>>;
type FollowSet<'a> = HashMap<&'a NonTerminal, HashSet<&'a Terminal>>;
type Table<'a> = HashMap<(&'a NonTerminal, &'a Terminal), Vec<&'a Production>>;

fn first_rhs<'a>(rhs: &'a [Symbol], first: &'a FirstSet) -> (HashSet<&'a Terminal>, bool) {
    let mut set = HashSet::new();
    let mut nullable = true;
    for symbol in rhs.iter() {
        if !nullable {
            break;
        }
        nullable = false;
        match symbol {
            Symbol::Terminal(t) => {
                set.insert(t);
            }
            Symbol::NonTerminal(nt) => {
                let elements = first[nt].iter().copied().collect::<Vec<_>>();
                for element in elements {
                    if let Some(element) = element {
                        set.insert(element);
                    } else {
                        nullable = true;
                    }
                }
            }
        }
    }
    (set, nullable)
}

fn create_first(grammar: &Grammar) -> FirstSet {
    // None represents epsilon
    //
    // first[nt] contains None only if nt is nullable.
    // Relationship should be iff by the end of the fixed point iterations.
    // (But I should probably prove this)
    let mut first = grammar
        .nonterminals()
        .iter()
        .zip(std::iter::repeat_with(HashSet::new))
        .collect::<HashMap<_, _>>();

    let mut changed = true;
    while changed {
        changed = false;
        for production in grammar.productions() {
            let lhs = production.lhs();
            let rhs = production.rhs();
            let mut nullable = true;
            for symbol in rhs.iter() {
                if !nullable {
                    break;
                }
                nullable = false;
                match symbol {
                    Symbol::Terminal(t) => {
                        changed |= first.get_mut(lhs).unwrap().insert(Some(t));
                    }
                    Symbol::NonTerminal(nt) => {
                        let elements = first[nt]
                            .iter()
                            .copied()
                            .filter(Option::is_some) // Don't add epsilons
                            .collect::<Vec<_>>();
                        let set = first.get_mut(lhs).unwrap();
                        for element in elements {
                            if element.is_some() {
                                changed |= set.insert(element);
                            } else {
                                // nullable = true;
                            }
                        }
                    }
                }
            }
            if nullable {
                changed |= first.get_mut(lhs).unwrap().insert(None);
            }
        }
    }

    first
}

pub fn create_follow<'a>(grammar: &'a Grammar, first: &'a FirstSet) -> FollowSet<'a> {
    let mut follow = grammar
        .nonterminals()
        .iter()
        .zip(std::iter::repeat_with(HashSet::new))
        .collect::<HashMap<_, _>>();
    follow
        .get_mut(grammar.start())
        .unwrap()
        .insert(Symbol::eoim());
    let mut changed = true;
    while changed {
        changed = false;
        for production in grammar.productions() {
            let lhs = production.lhs();
            let rhs = production.rhs();

            for (i, current) in rhs.iter().enumerate() {
                let Symbol::NonTerminal(current) = current else {
                    continue;
                };
                // `nullable` is true iff everything following the current symbol up
                // till `next` is nullable
                let mut nullable = true;
                for next in rhs.iter().skip(i + 1) {
                    if !nullable {
                        break;
                    }
                    nullable = false;
                    match next {
                        Symbol::Terminal(next) => {
                            changed |= follow.get_mut(current).unwrap().insert(next);
                        }
                        Symbol::NonTerminal(next) => {
                            let set = follow.get_mut(current).unwrap();
                            for f in first[next].iter() {
                                if let Some(t) = f {
                                    changed |= set.insert(t);
                                } else {
                                    // epsilon
                                    nullable = true;
                                }
                            }
                        }
                    }
                }
                if nullable {
                    // Either nullable or current is the last symbol in the rhs
                    let elements = follow[lhs].iter().copied().collect::<Vec<_>>();
                    let set = follow.get_mut(current).unwrap();
                    for t in elements {
                        changed |= set.insert(t);
                    }
                }
            }
        }
    }
    follow
}

pub fn create_table<'a>(grammar: &'a Grammar, first: &'a FirstSet, follow: &'a FollowSet) {
    for production in grammar.productions() {
        let lhs = production.lhs();
        let rhs = production.rhs();
    }
}

pub fn parse<'a>(grammar: &'a Grammar, tokens: &[&str]) -> Option<ParseTree<'a>> {
    let first = create_first(grammar);
    let follow = create_follow(grammar, &first);

    println!("First: ");
    for (k, v) in first.iter() {
        let v = v
            .iter()
            .map(|s| {
                if let Some(s) = s {
                    s.to_string()
                } else {
                    "ϵ".to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        println!("{}: {}", k, v);
    }
    println!();
    println!("Follow: ");
    for (k, v) in follow.iter() {
        let v = v
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        println!("{}: {}", k, v);
    }

    None
}
